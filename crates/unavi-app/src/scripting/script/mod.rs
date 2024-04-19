use bevy::{
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};

use wasm_bridge::{
    component::{Resource, ResourceAny},
    AsContextMut, Config, Engine,
};

use self::load::{ScriptBinding, ScriptEcsWorld, ScriptOutput, ScriptStore};

use super::asset::Wasm;

pub mod commands;
mod host;
pub mod load;
pub mod query;
mod state;
mod stream;

wasm_bridge::component::bindgen!({
    async: true,
    path: "../../wired-protocol/spatial/wit/wired-script",
    with: {
        "wired:ecs/types": host::wired_ecs,
        "wired:ecs/types/component": host::wired_ecs::Component,
        "wired:ecs/types/component-instance": host::wired_ecs::ComponentInstance,
        "wired:ecs/types/ecs-world": host::wired_ecs::EcsWorld,
        "wired:ecs/types/entity": host::wired_ecs::Entity,
        "wired:ecs/types/query": host::wired_ecs::Query,
    }
});

#[derive(Bundle, Clone)]
pub struct ScriptBundle {
    pub name: Name,
    pub wasm: Handle<Wasm>,
}

#[derive(Component, Clone)]
pub struct WasmEngine(pub Engine);

impl Default for WasmEngine {
    fn default() -> Self {
        let mut config = Config::default();
        config.async_support(true);
        config.wasm_component_model(true);

        let engine = Engine::new(&config).expect("Failed to create engine");

        Self(engine)
    }
}

#[derive(Component)]
pub struct ScriptInitialized;

#[derive(Component)]
pub struct ScriptInitializing(Task<wasm_bridge::Result<ResourceAny>>);

#[derive(Component)]
pub struct ScriptResource(ResourceAny);

pub fn init_scripts(
    mut commands: Commands,
    scripts: Query<
        (Entity, &ScriptBinding, &ScriptStore, &ScriptEcsWorld),
        (Without<ScriptInitialized>, Without<ScriptInitializing>),
    >,
    mut initializing: Query<(Entity, &mut ScriptInitializing)>,
) {
    for (entity, mut task) in initializing.iter_mut() {
        if let Some(res) = block_on(future::poll_once(&mut task.0)) {
            commands
                .entity(entity)
                .insert(ScriptInitialized)
                .remove::<ScriptInitializing>();

            let script = match res {
                Ok(r) => r,
                Err(e) => {
                    error!("Error initializing script: {}", e);
                    continue;
                }
            };

            commands.entity(entity).insert(ScriptResource(script));
        }
    }

    let pool = AsyncComputeTaskPool::get();

    for (entity, binding, store, ecs_world) in scripts.iter() {
        let binding = binding.0.clone();
        let store = store.0.clone();

        let ecs_world = Resource::new_borrow(ecs_world.0.rep());

        let task = pool.spawn(async move {
            let binding = binding.lock().await;
            let mut store = store.lock().await;

            binding
                .interface0
                .call_init(store.as_context_mut(), ecs_world)
                .await
        });

        commands.entity(entity).insert(ScriptInitializing(task));
    }
}

const SCRIPT_UPDATE_HZ: f32 = 2.0;
const SCRIPT_DELTA: f32 = 1.0 / SCRIPT_UPDATE_HZ;

pub fn update_scripts(
    last_update: Local<f32>,
    time: Res<Time>,
    scripts: Query<
        (
            &Name,
            &ScriptBinding,
            &ScriptStore,
            &ScriptEcsWorld,
            &ScriptResource,
            &ScriptOutput,
        ),
        With<ScriptInitialized>,
    >,
    mut tasks: Local<Vec<Task<wasm_bridge::Result<()>>>>,
) {
    let mut completed_tasks = Vec::new();

    for (i, task) in tasks.iter_mut().enumerate() {
        if let Some(res) = block_on(future::poll_once(task)) {
            if let Err(e) = res {
                error!("Script update errror: {}", e);
            }

            completed_tasks.push(i);
        }
    }

    for i in completed_tasks {
        let _task = tasks.remove(i);
    }

    let current_time = time.elapsed_seconds();
    let delta = current_time - *last_update;

    if delta < SCRIPT_DELTA {
        return;
    }

    let pool = AsyncComputeTaskPool::get();

    for (name, binding, store, ecs_world, script_resource, output) in scripts.iter() {
        let binding = binding.0.clone();
        let ecs_world = Resource::new_borrow(ecs_world.0.rep());
        let name = name.to_string();
        let output = output.0.clone();
        let script_resource = script_resource.0;
        let store = store.0.clone();

        let task = pool.spawn(async move {
            let binding = binding.lock().await;
            let mut store = store.lock().await;

            let resource = binding
                .interface0
                .call_update(store.as_context_mut(), ecs_world, script_resource)
                .await;

            let output = output.lock().await;
            let mut out_bytes = Vec::new();

            while let Ok(bytes) = output.try_recv() {
                out_bytes.extend(&bytes);
            }

            if !out_bytes.is_empty() {
                let out_str = String::from_utf8_lossy(&out_bytes);
                info!("{}: {}", name, out_str)
            }

            resource
        });

        tasks.push(task);
    }
}
