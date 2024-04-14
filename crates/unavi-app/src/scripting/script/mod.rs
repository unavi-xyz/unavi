use std::sync::{mpsc::Receiver, Arc, Mutex};

use bevy::{
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use bytes::Bytes;
use wasm_bridge::{
    component::{Component, Linker, ResourceTable},
    AsContextMut, Config, Engine, Store,
};
use wasm_bridge_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

use self::stream::OutStream;

use super::asset::Wasm;

mod stream;

wasm_bridge::component::bindgen!({
    async: true,
    path: "../../wired-protocol/spatial/wired-script/world.wit",
});

#[derive(Bundle, Clone)]
pub struct ScriptBundle {
    pub wasm: Handle<Wasm>,
}

impl ScriptBundle {
    pub fn new(wasm: Handle<Wasm>) -> Self {
        ScriptBundle { wasm }
    }
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
pub struct InstantiatedScript(pub Script);

#[derive(Component)]
pub struct ScriptOutput(pub Arc<Mutex<Receiver<Bytes>>>);

pub struct ScriptTaskOutput {
    pub script: Script,
    pub store: Store<RuntimeState>,
}

pub fn load_scripts(
    mut commands: Commands,
    mut tasks: Local<Vec<Task<ScriptTaskOutput>>>,
    to_load: Query<(Entity, &Handle<Wasm>), Without<WasmEngine>>,
    wasm_assets: Res<Assets<Wasm>>,
) {
    let pool = AsyncComputeTaskPool::get();

    for (i, task) in tasks.iter_mut().enumerate() {
        if let Some(_output) = block_on(future::poll_once(task)) {
            info!("Loaded script!");

            let _task = tasks.remove(i);
            break;
        }
    }

    for (entity, wasm) in to_load.iter() {
        let bytes = match wasm_assets.get(wasm) {
            Some(wasm) => wasm.0.clone(),
            None => continue,
        };

        let (stream, recv) = OutStream::new();

        // Create a new engine for each script.
        // In the future an engine could be shared across many scripts.
        let engine = WasmEngine::default();

        commands
            .entity(entity)
            .insert(engine.clone())
            .insert(ScriptOutput(Arc::new(Mutex::new(recv))));

        let task = pool.spawn(async move {
            let wasi = WasiCtxBuilder::new().stdout(stream).build();

            let mut store = Store::new(&engine.0, RuntimeState::new(wasi));

            let mut linker = Linker::new(&engine.0);
            wasm_bridge_wasi::command::add_to_linker(&mut linker).unwrap();

            let component = Component::new_safe(&engine.0, bytes).await.unwrap();

            let (script, _) =
                Script::instantiate_async(store.as_context_mut(), &component, &linker)
                    .await
                    .unwrap();

            script
                .interface0
                .call_init(store.as_context_mut())
                .await
                .unwrap();

            ScriptTaskOutput { script, store }
        });

        tasks.push(task);
    }
}

pub fn log_script_output(outputs: Query<&ScriptOutput>) {
    for output in outputs.iter() {
        let recv = output.0.lock().unwrap();

        let mut out = Vec::new();

        while let Ok(bytes) = recv.try_recv() {
            out.extend(&bytes);
        }

        if out.is_empty() {
            continue;
        }

        let out_str = String::from_utf8_lossy(&out);
        info!("{}", out_str);
    }
}

pub struct RuntimeState {
    table: ResourceTable,
    wasi: WasiCtx,
}

impl RuntimeState {
    pub fn new(wasi: WasiCtx) -> Self {
        Self {
            table: Default::default(),
            wasi,
        }
    }
}

impl WasiView for RuntimeState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}
