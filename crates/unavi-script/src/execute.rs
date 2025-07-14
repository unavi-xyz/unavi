use std::{borrow::Cow, future::poll_fn, pin::Pin, sync::Arc, task::Poll, time::Duration};

use anyhow::Context;
use bevy::{ecs::entity::EntityHashMap, prelude::*, utils::synccell::SyncCell};
use bevy_async_task::TaskPool;
use tokio::{
    io::{AsyncRead, DuplexStream, ReadBuf},
    sync::Mutex,
};
use wasmtime::{AsContextMut, Store, component::ResourceAny};

use crate::load::{
    LoadedScript, StoreState,
    log::{ScriptStderr, ScriptStdout},
};

const TICK_DURATION: Duration = Duration::from_millis(200);
const YIELD_DURATION: Duration = Duration::from_millis(20);

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

#[derive(Resource)]
pub struct Runtimes(pub SyncCell<EntityHashMap<Arc<Mutex<RuntimeCtx>>>>);

impl Default for Runtimes {
    fn default() -> Self {
        Self(SyncCell::new(EntityHashMap::default()))
    }
}

pub fn execute_script_updates(
    time: Res<Time>,
    scripts: Query<(Entity, &LoadedScript, Option<&Name>)>,
    mut runtimes: NonSendMut<Runtimes>,
    mut pool: TaskPool<anyhow::Result<()>>,
    mut last_tick: Local<Duration>,
) {
    for item in pool.iter_poll() {
        match item {
            Poll::Ready(Ok(_)) => {}
            Poll::Ready(Err(e)) => {
                error!("Script execution error: {e}");
            }
            Poll::Pending => {}
        }
    }

    if !pool.is_idle() {
        return;
    }

    let elapsed = time.elapsed();
    let delta = elapsed - *last_tick;

    if delta < TICK_DURATION {
        return;
    }

    *last_tick = elapsed;

    for (ent, instance, name) in scripts {
        let Some(rt) = runtimes.0.get().get(&ent).cloned() else {
            warn!("Script {ent} has no store");
            continue;
        };

        let instance = instance.0.clone();
        let name = name
            .map(|n| n.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        pool.spawn(async move {
            let mut rt = rt.lock().await;

            match rt.script {
                None => {
                    let script = instance
                        .wired_script_types()
                        .script()
                        .call_constructor(rt.store.as_context_mut())
                        .await
                        .context("construct script resource")?;

                    rt.script = Some(script);
                }
                Some(script) => {
                    instance
                        .wired_script_types()
                        .script()
                        .call_update(rt.store.as_context_mut(), script, delta.as_secs_f32())
                        .await?;
                }
            }

            let mut buf = [0; 1024];
            if let Some(s) = try_read_text_stream(&mut buf, &mut rt.stdout.0).await {
                info!("[{name}] {}", s.trim_end());
            }
            if let Some(s) = try_read_text_stream(&mut buf, &mut rt.stderr.0).await {
                error!("[{name}] {}", s.trim_end());
            }

            Ok(())
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
