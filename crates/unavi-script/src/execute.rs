use std::{borrow::Cow, future::poll_fn, pin::Pin, sync::Arc, task::Poll, time::Duration};

use anyhow::Context;
use bevy::prelude::*;
use bevy_async_task::TaskPool;
use tokio::{
    io::{AsyncRead, DuplexStream, ReadBuf},
    sync::Mutex,
};
use wasmtime::{AsContextMut, Store, component::ResourceAny};

use crate::{
    WasmEngine,
    load::{
        Executing, LoadedScript, StoreState,
        log::{ScriptStderr, ScriptStdout},
    },
};

const TICK_DURATION: Duration = Duration::from_millis(100);

pub fn increment_epochs(engines: Query<&WasmEngine>) {
    for engine in engines {
        engine.0.increment_epoch();
    }
}

pub struct RuntimeCtx {
    pub(crate) store: Store<StoreState>,
    stdout: ScriptStdout,
    stderr: ScriptStderr,
    script: Option<ResourceAny>,
}

impl RuntimeCtx {
    pub fn new(store: Store<StoreState>, stdout: ScriptStdout, stderr: ScriptStderr) -> Self {
        Self {
            store,
            stdout,
            stderr,
            script: None,
        }
    }
}

#[derive(Component)]
pub struct ScriptRuntime(pub Arc<Mutex<RuntimeCtx>>);

pub fn execute_script_updates(
    time: Res<Time>,
    mut scripts: Query<(
        Entity,
        &mut Executing,
        &LoadedScript,
        &ScriptRuntime,
        Option<&Name>,
    )>,
    mut pool: TaskPool<anyhow::Result<Entity>>,
    mut last_tick: Local<Duration>,
) {
    for item in pool.iter_poll() {
        match item {
            Poll::Ready(Ok(ent)) => {
                for (e, mut executing, ..) in scripts.iter_mut() {
                    if e == ent {
                        **executing = false;
                    }
                }
            }
            Poll::Ready(Err(e)) => {
                error!("Script execution error: {e:?}");
            }
            Poll::Pending => {}
        }
    }

    let elapsed = time.elapsed();
    let delta = elapsed - *last_tick;

    if delta < TICK_DURATION {
        return;
    }

    *last_tick = elapsed;

    for (ent, mut executing, instance, rt, name) in scripts.iter_mut() {
        if **executing {
            continue;
        }

        **executing = true;

        let instance = instance.0.clone();
        let name = name
            .map(|n| n.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let rt = rt.0.clone();

        pool.spawn(async move {
            let mut rt = rt.lock().await;
            rt.store.set_epoch_deadline(1);

            match rt.script {
                None => {
                    let script = instance
                        .wired_script_types()
                        .script()
                        .call_constructor(rt.store.as_context_mut())
                        .await
                        .context("script constructor")?;
                    rt.script = Some(script);
                }
                Some(script) => {
                    instance
                        .wired_script_types()
                        .script()
                        .call_update(rt.store.as_context_mut(), script, delta.as_secs_f32())
                        .await
                        .context("script update")?;
                }
            }

            let mut buf = [0; 1024];
            if let Some(s) = try_read_text_stream(&mut buf, &mut rt.stdout.0).await {
                info!("[{name}] {}", s.trim_end());
            }
            if let Some(s) = try_read_text_stream(&mut buf, &mut rt.stderr.0).await {
                error!("[{name}] {}", s.trim_end());
            }

            Ok(ent)
        });
    }
}

async fn try_read_text_stream<'a>(
    buf: &'a mut [u8],
    stream: &mut DuplexStream,
) -> Option<Cow<'a, str>> {
    let mut pinned = Pin::new(stream);
    let mut read_buf = ReadBuf::new(buf);

    let n = poll_fn(|cx| match pinned.as_mut().poll_read(cx, &mut read_buf) {
        Poll::Ready(Ok(())) => Poll::Ready(read_buf.filled().len()),
        Poll::Ready(Err(e)) => Poll::Ready({
            error!("Error reading buf: {e}");
            0
        }),
        Poll::Pending => Poll::Ready(0),
    })
    .await;

    if n == 0 {
        return None;
    }

    let s = String::from_utf8_lossy(&buf[0..n]);
    Some(s)
}
