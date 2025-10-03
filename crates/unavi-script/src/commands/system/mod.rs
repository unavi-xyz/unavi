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
use tokio::sync::{
    Notify,
    mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
};
use tracing::Instrument;
use wasmtime::{AsContextMut, Store, component::ResourceAny};

use crate::{
    api::wired::ecs::wired::ecs::types::{Param, ParamData, QueryData},
    execute::init::InitializedScript,
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

use super::{VComponent, VEntity, VOwner};

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
                let line = line.trim();
                if !line.is_empty() {
                    info!("{line}");
                }
            }
        }
        if let Some(s) = log::try_read_text_stream(&mut buf, &mut self.stderr.0).await {
            for line in s.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    error!("{line}");
                }
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

#[derive(Component)]
pub struct StartupSystems {
    complete: HashMap<u32, bool>,
    complete_send: UnboundedSender<u32>,
    complete_recv: UnboundedReceiver<u32>,
}

impl Default for StartupSystems {
    fn default() -> Self {
        let (send, recv) = unbounded_channel();
        Self {
            complete: HashMap::default(),
            complete_send: send,
            complete_recv: recv,
        }
    }
}

impl StartupSystems {
    pub fn is_complete(&self) -> bool {
        !self.complete.iter().any(|(_, x)| !x)
    }
}

const SCHEDULES: &[WSchedule] = &[WSchedule::Render, WSchedule::Startup, WSchedule::Update];

#[derive(Component)]
pub struct VSystemDependencies(pub HashMap<WSchedule, ScheduleDependencies>);

impl Default for VSystemDependencies {
    fn default() -> Self {
        let mut map = HashMap::default();
        for s in SCHEDULES {
            map.insert(*s, ScheduleDependencies::default());
        }
        Self(map)
    }
}

#[derive(Default)]
pub struct ScheduleDependencies {
    pub dependencies: HashMap<u32, Vec<u32>>,
    complete: HashMap<u32, Arc<Notify>>,
    ready: HashMap<u32, Arc<Notify>>,
    start: Arc<Notify>,
}

#[derive(Component, Default)]
pub struct ScriptCycles(HashMap<WSchedule, ScriptCycle>);

pub struct ScriptCycle {
    i: usize,
    task: Option<Task<()>>,
}

impl Default for ScriptCycle {
    fn default() -> Self {
        Self { i: 1, task: None }
    }
}

/// Increase the script cycle once all vsystems are complete.
pub fn tick_script_cycle(
    mut scripts: Query<(&VSystemDependencies, &mut ScriptCycles), With<InitializedScript>>,
) {
    for (vsystem_deps, mut cycles) in scripts.iter_mut() {
        for s in SCHEDULES {
            let cycle = cycles.0.entry(*s).or_default();

            if let Some(mut t) = cycle.task.as_mut() {
                match block_on(poll_once(&mut t)) {
                    Some(_) => {
                        cycle.i += 1;
                        cycle.task = None;
                    }
                    None => continue,
                }
            }

            if *s == WSchedule::Startup && cycle.i > 1 {
                continue;
            }

            let deps = vsystem_deps.0.get(s).unwrap();
            let waiters = deps.complete.values().cloned().collect::<Vec<_>>();
            if waiters.is_empty() {
                continue;
            }

            let pool = AsyncComputeTaskPool::get();
            cycle.task = Some(pool.spawn(async move {
                futures::future::join_all(waiters.iter().map(|n| n.notified())).await;
            }));
        }
    }
}

/// Starts script execution once all vsystems are ready.
pub fn start_script_cycle(
    mut scripts: Query<&VSystemDependencies, With<InitializedScript>>,
    mut tasks: Local<HashMap<WSchedule, Option<Task<()>>>>,
) {
    for vsystem_deps in scripts.iter_mut() {
        for s in SCHEDULES {
            let task = tasks.entry(*s).or_default();

            if let Some(mut t) = task.as_mut() {
                match block_on(poll_once(&mut t)) {
                    Some(_) => {
                        *task = None;
                    }
                    None => continue,
                }
            }

            let deps = vsystem_deps.0.get(s).unwrap();

            let waiters = deps.ready.values().cloned().collect::<Vec<_>>();
            if waiters.is_empty() {
                continue;
            }

            let start = deps.start.clone();

            let pool = AsyncComputeTaskPool::get();
            *task = Some(pool.spawn(async move {
                futures::future::join_all(waiters.iter().map(|n| n.notified())).await;
                start.notify_waiters();
            }));
        }
    }
}

pub fn build_system(
    world: &mut World,
    entity: Entity,
    id: u32,
    system: WSystem,
) -> anyhow::Result<()> {
    let ready = Arc::new(Notify::default());
    let complete = Arc::new(Notify::default());

    if let Some(mut vsystem_deps) = world.get_mut::<VSystemDependencies>(entity) {
        let schedule_deps = vsystem_deps.0.get_mut(&system.schedule).unwrap();
        schedule_deps.ready.insert(id, ready.clone());
        schedule_deps.complete.insert(id, complete.clone());
    } else {
        bail!("VSystemDependencies not found on script entity")
    }

    let vent_id = if let Some(vent_id) = world.component_id::<VEntity>() {
        vent_id
    } else {
        world.register_component::<VEntity>()
    };

    let mut queries = Vec::new();
    let mut component_sizes = HashMap::new();

    for param in system.params {
        match param {
            Param::Query(q) => {
                let mut vcomp_query = world.query::<(&VOwner, &VComponent)>();

                let mut find_component = |wasm_id: u32| -> Option<ComponentId> {
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
                    builder.ref_id(vent_id);

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
                let mut query_data = QueryData {
                    ents: Vec::new(),
                    data: Vec::new(),
                };

                for ent in query {
                    let Some(wasm_id) = ent.get::<VEntity>().map(|x| x.wasm_id) else {
                        continue;
                    };
                    let mut ent_data = Vec::new();

                    for access in ent
                        .access()
                        .try_iter_component_access()
                        .expect("unbounded access")
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                    {
                        match access {
                            ComponentAccessKind::Shared(id) => {
                                if id == vent_id {
                                    continue;
                                }

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

                    query_data.ents.push(wasm_id);
                    query_data.data.push(ent_data);
                }

                out.push(ParamData::Query(query_data));
            }

            out
        },
    );

    #[derive(Default)]
    struct UpstreamDeps {
        ids: Vec<u32>,
        waiters: Arc<Vec<Arc<Notify>>>,
    }

    let f = move |input: In<Vec<ParamData>>,
                  time: Res<Time>,
                  mut scripts: Query<(
        &LoadedScript,
        &ScriptRuntime,
        Option<&Name>,
        &mut StartupSystems,
        &VSystemDependencies,
        &ScriptCycles,
    )>,
                  mut executing: Local<Option<Task<anyhow::Result<()>>>>,
                  mut upstream: Local<UpstreamDeps>,
                  mut prev_i: Local<usize>| {
        if let Some(mut task) = executing.as_mut() {
            match block_on(poll_once(&mut task)) {
                Some(Ok(_)) => {
                    *executing = None;
                }
                Some(Err(e)) => {
                    error!("Script system execution error: {e:?}");
                    *executing = None;
                }
                None => return,
            }
        }

        let Ok((loaded, rt, name, mut startup, vsystem_deps, cycles)) = scripts.get_mut(entity)
        else {
            // Return early if script is not found. Eventually we will remove
            // this system from its schedule on script removal, but for now it runs indefinitely.
            // Waiting on https://github.com/bevyengine/bevy/issues/20115.
            return;
        };

        let cycle = cycles.0.get(&system.schedule).unwrap();
        if cycle.i == *prev_i {
            // Previous cycle not yet complete.
            return;
        }

        while let Ok(r) = startup.complete_recv.try_recv() {
            startup.complete.insert(r, true);
        }

        if system.schedule == WSchedule::Startup {
            if startup.complete.get(&id) == Some(&true) {
                return;
            }
        } else if !startup.is_complete() {
            // Startup not finished.
            return;
        }

        let schedule_deps = vsystem_deps.0.get(&system.schedule).unwrap();

        if let Some(deps) = schedule_deps.dependencies.get(&id)
            && *deps != *upstream.ids
        {
            let mut waiters = Vec::new();
            for d in deps {
                if let Some(notify) = schedule_deps.complete.get(d) {
                    waiters.push(notify.clone());
                }
            }

            upstream.ids = deps.clone();
            upstream.waiters = Arc::new(waiters);
        }

        let name = name
            .map(|n| n.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let complete = complete.clone();
        let ctx = rt.ctx.clone();
        let elapsed = time.elapsed();
        let guest = loaded.0.clone();
        let ready = ready.clone();
        let start = schedule_deps.start.clone();
        let startup_complete = startup.complete_send.clone();
        let waiters = upstream.waiters.clone();

        let pool = AsyncComputeTaskPool::get();
        let task = pool.spawn(
            async move {
                let start = start.notified();
                let dep_fut = futures::future::join_all(waiters.iter().map(|n| n.notified()));

                ready.notify_one();

                // Wait for all VSystems to be ready. We don't want to execute
                // and signal completion before dependants are listening.
                start.await;

                // Wait for dependencies to complete.
                dep_fut.await;

                let mut ctx = ctx.lock().await;
                ctx.store.set_epoch_deadline(1);

                // let delta = elapsed - ctx.last_tick;
                ctx.last_tick = elapsed;

                let Some(script) = ctx.script else {
                    bail!("Script resource not found")
                };

                let res = exec_system(&mut ctx.store, script, &guest, id, &input)
                    .await
                    .with_context(|| format!("exec {id}"));

                ctx.flush_logs().await;

                if system.schedule == WSchedule::Startup {
                    startup_complete.send(id)?;
                }

                complete.notify_waiters();

                res
            }
            .instrument(info_span!("", script = name)),
        );

        *executing = Some(task);
        *prev_i = cycle.i;
    };

    if system.schedule == WSchedule::Startup {
        let mut startup = world.get_mut::<StartupSystems>(entity).unwrap();
        startup.complete.insert(id, false);
    };

    let s_func = move |_: &mut World, s: &mut Schedule| {
        let sys = f_input.pipe(f);
        s.add_systems(sys);
    };

    match system.schedule {
        WSchedule::Render => world.schedule_scope(Update, s_func),
        WSchedule::Update | WSchedule::Startup => world.schedule_scope(FixedUpdate, s_func),
    };

    Ok(())
}

async fn exec_system(
    store: &mut Store<StoreState>,
    script: ResourceAny,
    guest: &Guest,
    id: u32,
    params: &[ParamData],
) -> anyhow::Result<()> {
    guest
        .wired_ecs_guest_api()
        .script()
        .call_exec_system(store.as_context_mut(), script, id, params)
        .await?;

    Ok(())
}
