use std::{
    sync::{mpsc::Receiver, Arc, Mutex},
    time::Duration,
};

use bevy::{
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use bytes::Bytes;
use thiserror::Error;
use wasm_bridge::{
    component::{Component, Linker, Resource},
    AsContextMut, Config, Engine, Store,
};
use wasm_bridge_wasi::WasiCtxBuilder;

use self::{
    state::{ScriptCommand, ScriptState},
    stream::OutStream,
};

use super::asset::Wasm;

mod host;
mod state;
mod stream;

wasm_bridge::component::bindgen!({
    async: true,
    path: "../../wired-protocol/spatial/wit/wired-script",
    with: {
        "wired:ecs/types": host::wired_ecs,
        "wired:ecs/types/component-instance": host::wired_ecs::ComponentInstance,
        "wired:ecs/types/component": host::wired_ecs::Component,
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
pub struct InstantiatedScript(pub Script);

#[derive(Component)]
pub struct ScriptOutput(pub Arc<Mutex<Receiver<Bytes>>>);

#[derive(Component)]
pub struct ScriptCommandReceiver(pub Arc<Mutex<Receiver<ScriptCommand>>>);

pub struct ScriptTaskOutput {
    pub script: Script,
    pub store: Store<ScriptState>,
}

#[derive(Error, Debug)]
pub enum ScriptTaskErr {
    #[error("Failed to add WASI to linker: {0}")]
    AddWasiToLinker(anyhow::Error),
    #[error("Failed to add host scripts to linker: {0}")]
    AddHostToLinker(anyhow::Error),
    #[error("Failed to compile WASM component: {0}")]
    Compile(anyhow::Error),
    #[error("Failed to instantiate script: {0}")]
    Instantiate(anyhow::Error),
    #[error("Failed to initialize script: {0}")]
    Init(anyhow::Error),
}

pub fn load_scripts(
    mut commands: Commands,
    mut tasks: Local<Vec<Task<Result<ScriptTaskOutput, ScriptTaskErr>>>>,
    to_load: Query<(Entity, &Handle<Wasm>), Without<WasmEngine>>,
    wasm_assets: Res<Assets<Wasm>>,
) {
    let pool = AsyncComputeTaskPool::get();

    for (i, task) in tasks.iter_mut().enumerate() {
        if let Some(res) = block_on(future::poll_once(task)) {
            let _output = match res {
                Ok(out) => out,
                Err(e) => {
                    error!("Error loading script: {}", e);
                    continue;
                }
            };

            info!("Loaded script!");

            let _t = tasks.remove(i);
            break;
        }
    }

    for (entity, wasm) in to_load.iter() {
        let bytes = match wasm_assets.get(wasm) {
            Some(wasm) => wasm.0.clone(),
            None => continue,
        };

        let (stream, recv) = OutStream::new();
        let (send_command, recv_command) = std::sync::mpsc::sync_channel(100);

        // Create a new engine for each script.
        // In the future an engine could be shared across many scripts.
        let engine = WasmEngine::default();

        commands.entity(entity).insert((
            ScriptCommandReceiver(Arc::new(Mutex::new(recv_command))),
            ScriptOutput(Arc::new(Mutex::new(recv))),
            engine.clone(),
        ));

        let task = pool.spawn(async move {
            let wasi = WasiCtxBuilder::new().stdout(stream).build();

            let state = ScriptState {
                sender: send_command,
                table: Default::default(),
                wasi,
            };

            let mut store = Store::new(&engine.0, state);

            let mut linker = Linker::new(&engine.0);

            wasm_bridge_wasi::command::add_to_linker(&mut linker)
                .map_err(ScriptTaskErr::AddWasiToLinker)?;

            host::add_to_linker(&mut linker).map_err(ScriptTaskErr::AddHostToLinker)?;

            let component = Component::new_safe(&engine.0, bytes)
                .await
                .map_err(ScriptTaskErr::Compile)?;

            let (script, _) =
                Script::instantiate_async(store.as_context_mut(), &component, &linker)
                    .await
                    .map_err(ScriptTaskErr::Instantiate)?;

            let ecs_world_id = 0;
            let ecs_world = Resource::new_borrow(ecs_world_id);

            let res_script = match script
                .interface0
                .call_init(store.as_context_mut(), ecs_world)
                .await
            {
                Ok(r) => r,
                Err(e) => return Err(ScriptTaskErr::Init(e)),
            };

            for _ in 0..10 {
                std::thread::sleep(Duration::from_secs_f32(0.1));

                let ecs_world = Resource::new_borrow(ecs_world_id);

                if let Err(e) = script
                    .interface0
                    .call_update(store.as_context_mut(), ecs_world, res_script)
                    .await
                {
                    return Err(ScriptTaskErr::Init(e));
                }
            }

            Ok(ScriptTaskOutput { script, store })
        });

        tasks.push(task);
    }
}

pub fn log_script_output(outputs: Query<(&Name, &ScriptOutput)>) {
    for (name, output) in outputs.iter() {
        let recv = output.0.lock().unwrap();

        let mut out = Vec::new();

        while let Ok(bytes) = recv.try_recv() {
            out.extend(&bytes);
        }

        if out.is_empty() {
            continue;
        }

        let out_str = String::from_utf8_lossy(&out);
        info!("{}: {}", name, out_str);
    }
}
