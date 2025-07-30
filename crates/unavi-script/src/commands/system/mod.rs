use std::{sync::Arc, time::Duration};

use anyhow::{Context, bail};
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, poll_once},
};
use tracing::Instrument;
use wasmtime::{AsContextMut, Store, component::ResourceAny};

use crate::{
    api::wired::ecs::wired::ecs::types::{ComponentType, Param, ParamData, Primitive},
    load::{
        LoadedScript,
        bindings::{Guest, wired::ecs::types::Schedule as BSchedule},
        log::{ScriptStderr, ScriptStdout},
        state::StoreState,
    },
};

mod log;

pub struct RuntimeCtx {
    pub(crate) store: Store<StoreState>,
    stdout: ScriptStdout,
    stderr: ScriptStderr,
    pub(crate) script: Option<ResourceAny>,
    last_tick: Duration,
}

impl RuntimeCtx {
    pub async fn flush_logs(&mut self) {
        let mut buf = [0; 1024];
        if let Some(s) = log::try_read_text_stream(&mut buf, &mut self.stdout.0).await {
            for line in s.lines() {
                info!("{}", line.trim());
            }
        }
        if let Some(s) = log::try_read_text_stream(&mut buf, &mut self.stderr.0).await {
            for line in s.lines() {
                error!("{}", line.trim());
            }
        }
    }
}

#[derive(Component)]
pub struct ScriptRuntime {
    pub ctx: Arc<tokio::sync::Mutex<RuntimeCtx>>,
}

impl ScriptRuntime {
    pub fn new(store: Store<StoreState>, stdout: ScriptStdout, stderr: ScriptStderr) -> Self {
        Self {
            ctx: Arc::new(tokio::sync::Mutex::new(RuntimeCtx {
                store,
                stdout,
                stderr,
                script: None,
                last_tick: Duration::from_secs(0),
            })),
        }
    }
}

pub fn build_system(world: &mut World, entity: Entity, id: u64, schedule: BSchedule) {
    let f = move |time: Res<Time>,
                  scripts: Query<(&LoadedScript, &ScriptRuntime, Option<&Name>)>,
                  mut started: Local<bool>,
                  mut executing: Local<Option<Task<anyhow::Result<()>>>>| {
        if let Some(mut task) = executing.as_mut() {
            match block_on(poll_once(&mut task)) {
                Some(Ok(_)) => *executing = None,
                Some(Err(e)) => {
                    error!("Script system execution error: {e:?}");
                    *executing = None;
                }
                None => return,
            }
        }

        let Ok((loaded, rt, name)) = scripts.get(entity) else {
            // Return early if script is not found. Eventually we will remove
            // this system from its schedule on script removal, but for now it runs indefinitely.
            // Waiting on https://github.com/bevyengine/bevy/issues/20115.
            return;
        };

        if !*started {
            *started = true;

            if schedule == BSchedule::Update {
                // Startup has not yet run.
                return;
            }
        } else if schedule == BSchedule::Startup {
            // Startup already ran.
            return;
        }

        let name = name
            .map(|n| n.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let ctx = rt.ctx.clone();
        let elapsed = time.elapsed();
        let guest = loaded.0.clone();

        info!("Spawning system task: {id} ({schedule:?})");

        let pool = AsyncComputeTaskPool::get();
        let task = pool.spawn(
            async move {
                let mut ctx = ctx.lock().await;
                ctx.store.set_epoch_deadline(1);

                // let delta = elapsed - ctx.last_tick;
                ctx.last_tick = elapsed;

                let Some(script) = ctx.script else {
                    bail!("Script resource not found")
                };

                exec_system(&mut ctx.store, script, &guest, id)
                    .await
                    .with_context(|| format!("exec {id}"))?;

                ctx.flush_logs().await;

                Ok(())
            }
            .instrument(info_span!("", script = name)),
        );

        *executing = Some(task);
    };

    match schedule {
        BSchedule::Render => world.schedule_scope(Update, |_, s| {
            s.add_systems(f);
        }),
        BSchedule::Startup | BSchedule::Update => world.schedule_scope(FixedUpdate, |_, s| {
            s.add_systems(f);
        }),
    };
}

async fn exec_system(
    store: &mut Store<StoreState>,
    script: ResourceAny,
    guest: &Guest,
    id: u64,
) -> anyhow::Result<()> {
    let ecs = &store.data().rt.wired_ecs;
    let Some(sys) = ecs.systems.get(id as usize) else {
        bail!("system {id} not found")
    };

    let mut param_data = Vec::new();

    for param in sys.params.iter() {
        match param {
            Param::Query(query) => {
                let mut query_data = Vec::new();

                let mut ent_data = Vec::new();

                for c in &query.components {
                    let Some(comp) = ecs.components.get(*c as usize) else {
                        bail!("system {id} not found")
                    };

                    for (i, ty) in comp.types.iter().enumerate() {
                        match ty {
                            ComponentType::Primitive(p) => match p {
                                Primitive::F32 => {
                                    let val = (i + 1) as f32;
                                    ent_data.extend(bytemuck::bytes_of(&val).to_vec());
                                }
                                _ => todo!("primitive type unsupported"),
                            },
                            _ => todo!("component type unsupported"),
                        }
                    }
                }
                query_data.push(ent_data);

                param_data.push(ParamData::Query(query_data));
            }
        }
    }

    guest
        .wired_ecs_guest_api()
        .script()
        .call_exec_system(store.as_context_mut(), script, id, &param_data)
        .await?;

    Ok(())
}
