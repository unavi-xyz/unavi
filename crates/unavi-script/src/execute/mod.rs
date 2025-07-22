use std::{sync::Arc, task::Poll, time::Duration};

use anyhow::Context;
use bevy::prelude::*;
use bevy_async_task::TaskPool;
use wasmtime::{AsContextMut, Store, component::ResourceAny};

use crate::{
    WasmEngine,
    api::wired::ecs::wired::ecs::types::Schedule,
    load::{
        Executing, LoadedScript,
        bindings::Guest,
        log::{ScriptStderr, ScriptStdout},
        state::StoreState,
    },
};

mod log;

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
                    let ecs = &mut ctx.store.data_mut().data.wired_ecs;

                    let schedule = if ecs.initialized {
                        Schedule::Update
                    } else {
                        ecs.initialized = true;
                        Schedule::Startup
                    };

                    exec_schedule(&mut ctx.store, script, &guest, &schedule)
                        .await
                        .with_context(|| format!("exec schedule {schedule:?}"))?;
                }
            }

            let mut buf = [0; 1024];
            if let Some(s) = log::try_read_text_stream(&mut buf, &mut ctx.stdout.0).await {
                while ctx
                    .logs_send
                    .try_send(Log::Stdout(s.trim_end().to_string()))
                    .is_err()
                {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
            if let Some(s) = log::try_read_text_stream(&mut buf, &mut ctx.stderr.0).await {
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

async fn exec_schedule(
    store: &mut Store<StoreState>,
    script: ResourceAny,
    guest: &Guest,
    schedule: &Schedule,
) -> anyhow::Result<()> {
    let Some(systems) = store.data().data.wired_ecs.schedules.get(schedule) else {
        return Ok(());
    };

    for id in systems.clone() {
        exec_system(store, script, guest, id)
            .await
            .with_context(|| format!("exec system {id}"))?;
    }

    Ok(())
}

async fn exec_system(
    store: &mut Store<StoreState>,
    script: ResourceAny,
    guest: &Guest,
    id: u64,
) -> anyhow::Result<()> {
    let ecs = &store.data().data.wired_ecs;
    let Some(sys) = ecs.systems.get(id as usize) else {
        anyhow::bail!("system {id} not found")
    };

    let mut param_data = Vec::new();

    for query in &sys.queries {
        for p in &query.params {
            let Some(param) = ecs.params.get(*p as usize) else {
                anyhow::bail!("system {id} not found")
            };
        }
    }

    guest
        .wired_ecs_guest_api()
        .script()
        .call_exec_system(store.as_context_mut(), script, id, &param_data)
        .await?;

    Ok(())
}
