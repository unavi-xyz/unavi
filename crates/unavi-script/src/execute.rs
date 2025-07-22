use std::{borrow::Cow, future::poll_fn, pin::Pin, sync::Arc, task::Poll, time::Duration};

use anyhow::Context;
use bevy::prelude::*;
use bevy_async_task::TaskPool;
use tokio::io::{AsyncRead, DuplexStream, ReadBuf};
use wasmtime::{AsContextMut, Store, component::ResourceAny};

use crate::{
    WasmEngine,
    load::{
        Executing, LoadedScript, StoreState,
        log::{ScriptStderr, ScriptStdout},
    },
};

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
    last_tick: Duration,
    logs_send: std::sync::mpsc::SyncSender<Log>,
}

enum Log {
    Stdout(String),
    Stderr(String),
}

#[derive(Component)]
pub struct ScriptRuntime {
    pub ctx: Arc<tokio::sync::Mutex<RuntimeCtx>>,
    logs_recv: Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Log>>>,
}

impl ScriptRuntime {
    pub fn new(store: Store<StoreState>, stdout: ScriptStdout, stderr: ScriptStderr) -> Self {
        let (logs_send, logs_recv) = std::sync::mpsc::sync_channel(20);
        Self {
            ctx: Arc::new(tokio::sync::Mutex::new(RuntimeCtx {
                store,
                stdout,
                stderr,
                script: None,
                last_tick: Duration::from_secs(0),
                logs_send,
            })),
            logs_recv: Arc::new(std::sync::Mutex::new(logs_recv)),
        }
    }
}

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

    for (ent, mut executing, guest, rt, name) in scripts.iter_mut() {
        if **executing {
            continue;
        }

        **executing = true;

        let elapsed = time.elapsed();
        let guest = guest.0.clone();
        let ctx = rt.ctx.clone();

        let name = name
            .map(|n| n.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let recv = rt.logs_recv.lock().expect("logs_recv poisioned");
        while let Ok(log) = recv.try_recv() {
            match log {
                Log::Stdout(s) => info!("[{name}] {s}"),
                Log::Stderr(s) => error!("[{name}] {s}"),
            }
        }

        pool.spawn(async move {
            let mut ctx = ctx.lock().await;
            ctx.store.set_epoch_deadline(1);

            let delta = elapsed - ctx.last_tick;
            ctx.last_tick = elapsed;

            match ctx.script {
                None => {
                    let script = guest
                        .wired_ecs_guest_api()
                        .script()
                        .call_constructor(ctx.store.as_context_mut())
                        .await
                        .context("script constructor")?;
                    ctx.script = Some(script);
                }
                Some(script) => {
                    // guest
                    //     .wired_ecs_guest_api()
                    //     .script()
                    //     .call_update(ctx.store.as_context_mut(), script, delta.as_secs_f32())
                    //     .await
                    //     .context("script update")?;
                }
            }

            let mut buf = [0; 1024];
            if let Some(s) = try_read_text_stream(&mut buf, &mut ctx.stdout.0).await {
                while ctx
                    .logs_send
                    .try_send(Log::Stdout(s.trim_end().to_string()))
                    .is_err()
                {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
            if let Some(s) = try_read_text_stream(&mut buf, &mut ctx.stderr.0).await {
                while ctx
                    .logs_send
                    .try_send(Log::Stderr(s.trim_end().to_string()))
                    .is_err()
                {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
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
