use std::sync::{Arc, Mutex};

use bevy::{
    ecs::system::CommandQueue,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use wasm_bridge::{
    component::{new_component_async, Linker},
    AsContextMut, Config, Engine, Store,
};
use wasm_bridge_wasi::preview2::{command, ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

use super::{asset::Wasm, stream::OutStream};

wasm_bridge::component::bindgen!({
    async: true,
    path: "../../wired-protocol/spatial/wired-script/world.wit",
    world: "script",
});

#[derive(Resource, Default)]
pub struct ScriptLoadQueue(pub Vec<Handle<Wasm>>);

#[derive(Resource)]
pub struct WasmRuntime {
    pub store: Arc<Mutex<Store<RuntimeState>>>,
    pub out_bytes: Arc<Mutex<Vec<u8>>>,
}

pub struct RuntimeState {
    table: ResourceTable,
    wasi: WasiCtx,
}

impl WasiView for RuntimeState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

impl Default for WasmRuntime {
    fn default() -> Self {
        let mut config = Config::default();
        config.async_support(true);
        config.wasm_component_model(true);

        let engine = Engine::new(&config).expect("Failed to create engine");

        let out_bytes = Arc::new(Mutex::new(Vec::<u8>::new()));
        let out_stream = OutStream {
            data: out_bytes.clone(),
            max: 512,
        };

        let wasi = WasiCtxBuilder::new().stdout(out_stream).build();

        let table = ResourceTable::new();
        let state = RuntimeState { table, wasi };

        let store = Store::new(&engine, state);

        Self {
            store: Arc::new(Mutex::new(store)),
            out_bytes,
        }
    }
}

#[derive(Component)]
struct LoadScript(Task<CommandQueue>);

pub fn load_scripts(
    mut load_queue: ResMut<ScriptLoadQueue>,
    runtime: Res<WasmRuntime>,
    wasm_assets: Res<Assets<Wasm>>,
) {
    if !runtime.out_bytes.lock().unwrap().is_empty() {
        info!(
            "Script output: {}",
            String::from_utf8_lossy(&runtime.out_bytes.lock().unwrap())
        );
        runtime.out_bytes.lock().unwrap().clear();
    }

    if load_queue.0.is_empty() {
        return;
    }

    let mut to_remove = Vec::new();

    let pool = AsyncComputeTaskPool::get();

    for handle in &load_queue.0 {
        if let Some(wasm) = wasm_assets.get(handle) {
            to_remove.push(handle.clone());

            let store = runtime.store.clone();
            let bytes = wasm.bytes.clone();

            let mut linker = Linker::new(store.lock().unwrap().engine());
            if let Err(e) = command::add_to_linker(&mut linker) {
                error!("Failed to add wasi to linker: {}", e);
                continue;
            }

            #[allow(clippy::await_holding_lock)]
            pool.spawn_local(async move {
                let mut store = store.lock().unwrap();

                let component = new_component_async(store.engine(), &bytes).await.unwrap();

                let (script, _) =
                    Script::instantiate_async(&mut store.as_context_mut(), &component, &linker)
                        .await
                        .unwrap();

                script
                    .interface0
                    .call_init(&mut store.as_context_mut())
                    .await
                    .unwrap();

                info!("Script initialized");
            })
            .detach();
        }
    }

    for handle in to_remove {
        load_queue.0.retain(|h| h != &handle);
    }
}
