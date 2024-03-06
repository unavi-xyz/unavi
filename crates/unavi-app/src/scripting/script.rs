use std::sync::Arc;

use bevy::{
    prelude::*,
    tasks::{block_on, poll_once, AsyncComputeTaskPool, Task},
};
use wasm_bridge::{
    component::{new_component_async, Linker},
    Config, Engine, Store,
};

use super::asset::Wasm;

wasm_bridge::component::bindgen!({
    path: "../../wired-protocol/spatial/wired-script/world.wit",
    world: "script",
});

#[derive(Resource)]
pub struct WasmRuntime {
    engine: Arc<Engine>,
    store: Store<()>,
}

impl Default for WasmRuntime {
    fn default() -> Self {
        let mut config = Config::new();
        config.wasm_component_model(true);

        let engine = Engine::new(&config).expect("Failed to create WASM engine");
        let store = Store::new(&engine, ());

        Self {
            engine: Arc::new(engine),
            store,
        }
    }
}

#[derive(Resource, Default)]
pub struct ScriptsLoadQueue(pub Vec<Handle<Wasm>>);

#[derive(Component)]
pub struct LoadComponentTask(Task<wasm_bridge::Result<wasm_bridge::component::Component>>);

pub fn load_scripts(
    mut commands: Commands,
    runtime: Res<WasmRuntime>,
    mut load_queue: ResMut<ScriptsLoadQueue>,
    wasm_assets: Res<Assets<Wasm>>,
) {
    if load_queue.0.is_empty() {
        return;
    }

    let mut to_remove = Vec::new();

    let thread_pool = AsyncComputeTaskPool::get();

    for handle in &load_queue.0 {
        if let Some(wasm) = wasm_assets.get(handle) {
            info!("Wasm asset loaded, spawning task");

            let bytes = wasm.bytes.clone();
            let engine = runtime.engine.clone();

            let task = thread_pool.spawn(async move { new_component_async(&engine, &bytes).await });

            commands.spawn(LoadComponentTask(task));

            to_remove.push(handle.clone());
        }
    }

    for handle in to_remove {
        load_queue.0.retain(|h| h != &handle);
    }
}

pub fn load_components(
    mut commands: Commands,
    mut runtime: ResMut<WasmRuntime>,
    mut tasks: Query<(Entity, &mut LoadComponentTask)>,
) {
    for (entity, mut task) in tasks.iter_mut() {
        if let Some(res) = block_on(poll_once(&mut task.0)) {
            let component = match res {
                Ok(component) => component,
                Err(e) => {
                    error!("Failed to load component: {}", e);
                    commands.entity(entity).despawn();
                    continue;
                }
            };

            let linker = Linker::new(&runtime.engine);
            let (script, _) = Script::instantiate(&mut runtime.store, &component, &linker).unwrap();

            info!("Instantiated script");

            let res = script.interface0.call_init(&mut runtime.store);

            info!("Called script init: {:?}", res);
        }
    }
}
