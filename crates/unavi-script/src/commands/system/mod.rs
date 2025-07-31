use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::{Context, bail};
use bevy::{
    ecs::{
        component::ComponentId, query::ComponentAccessKind, system::QueryParamBuilder,
        world::FilteredEntityRef,
    },
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, poll_once},
};
use tracing::Instrument;
use wasmtime::{AsContextMut, Store, component::ResourceAny};

use crate::{
    api::wired::ecs::wired::ecs::types::{Param, ParamData},
    load::{
        LoadedScript,
        bindings::{
            Guest,
            wired::ecs::types::{Constraint, Schedule as WSchedule, System as WSystem},
        },
        log::{ScriptStderr, ScriptStdout},
        state::StoreState,
    },
};

use super::{VComponent, VOwner};

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

pub fn build_system(
    world: &mut World,
    entity: Entity,
    id: u64,
    system: WSystem,
) -> anyhow::Result<()> {
    let mut queries = Vec::new();
    let mut component_sizes = HashMap::new();

    for param in system.params {
        match param {
            Param::Query(q) => {
                let mut vcomp_query = world.query::<(&VOwner, &VComponent)>();

                let mut find_component = |wasm_id: u64| -> Option<ComponentId> {
                    vcomp_query.iter(world).find_map(|(owner, vcomp)| {
                        if owner.0 == entity && vcomp.wasm_id == wasm_id {
                            Some(vcomp.bevy_id)
                        } else {
                            None
                        }
                    })
                };

                let mut components = Vec::new();
                let mut with = Vec::new();
                let mut without = Vec::new();

                for wasm_id in q.components {
                    let Some(found) = find_component(wasm_id) else {
                        bail!("Query::components virtual component not found");
                    };

                    components.push(found);
                }

                for constraint in q.constraints {
                    match constraint {
                        Constraint::WithComponent(wasm_id) => {
                            let Some(found) = find_component(wasm_id) else {
                                bail!("Query::with virtual component not found");
                            };

                            with.push(found);
                        }
                        Constraint::WithoutComponent(wasm_id) => {
                            let Some(found) = find_component(wasm_id) else {
                                bail!("Query::without virtual component not found");
                            };

                            without.push(found);
                        }
                    }
                }

                for id in components.iter().chain(with.iter()).chain(without.iter()) {
                    if component_sizes.contains_key(id) {
                        continue;
                    }
                    let Some(info) = world.components().get_info(*id) else {
                        bail!("component info not found")
                    };
                    let size = info.layout().size() / size_of::<u8>();
                    component_sizes.insert(*id, size);
                }

                let builder = QueryParamBuilder::new_box(|builder| {
                    for id in components {
                        builder.ref_id(id);
                    }
                    for id in with {
                        builder.with_id(id);
                    }
                    for id in without {
                        builder.without_id(id);
                    }
                });

                queries.push(builder);
            }
        }
    }

    let f_input = (queries,).build_state(world).build_system(
        move |queries: Vec<Query<FilteredEntityRef>>| -> Vec<ParamData> {
            let mut out = Vec::new();

            for query in queries {
                let mut query_data = Vec::new();

                for ent in query {
                    let mut ent_data = Vec::new();

                    for access in ent
                        .access()
                        .try_iter_component_access()
                        .expect("unbounded access")
                    {
                        match access {
                            ComponentAccessKind::Shared(id) => {
                                let ptr = ent.get_by_id(id).unwrap();
                                let size = component_sizes.get(&id).copied().unwrap();

                                // SAFETY:
                                // - All virtual components are created with layout [u8]
                                // - len is calculated from the component descriptor
                                let data = unsafe {
                                    std::slice::from_raw_parts(
                                        ptr.assert_unique().as_ptr().cast::<u8>(),
                                        size,
                                    )
                                };

                                ent_data.extend(&data[0..size]);
                            }
                            ComponentAccessKind::Exclusive(_id) => {}
                            ComponentAccessKind::Archetypal(_id) => {}
                        }
                    }

                    query_data.push(ent_data)
                }

                out.push(ParamData::Query(query_data));
            }

            out
        },
    );

    let f = move |input: In<Vec<ParamData>>,
                  time: Res<Time>,
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

            if system.schedule == WSchedule::Update {
                // Startup has not yet run.
                return;
            }
        } else if system.schedule == WSchedule::Startup {
            // Startup already ran.
            return;
        }

        let name = name
            .map(|n| n.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let ctx = rt.ctx.clone();
        let elapsed = time.elapsed();
        let guest = loaded.0.clone();

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

                exec_system(&mut ctx.store, script, &guest, id, &input)
                    .await
                    .with_context(|| format!("exec {id}"))?;

                ctx.flush_logs().await;

                Ok(())
            }
            .instrument(info_span!("", script = name)),
        );

        *executing = Some(task);
    };

    match system.schedule {
        WSchedule::Render => world.schedule_scope(Update, |_, s| {
            s.add_systems(f_input.pipe(f));
        }),
        WSchedule::Startup | WSchedule::Update => world.schedule_scope(FixedUpdate, |_, s| {
            s.add_systems(f_input.pipe(f));
        }),
    };

    Ok(())
}

async fn exec_system(
    store: &mut Store<StoreState>,
    script: ResourceAny,
    guest: &Guest,
    id: u64,
    params: &[ParamData],
) -> anyhow::Result<()> {
    guest
        .wired_ecs_guest_api()
        .script()
        .call_exec_system(store.as_context_mut(), script, id, params)
        .await?;

    Ok(())
}
