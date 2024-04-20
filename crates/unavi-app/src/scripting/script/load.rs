use std::sync::{mpsc::Receiver, Arc};

use bevy::prelude::*;
use bevy_async_task::{AsyncTaskPool, AsyncTaskStatus};
use bytes::Bytes;
use thiserror::Error;
use tokio::sync::Mutex;
use wasm_bridge::{
    component::{Component, Linker, Resource},
    AsContextMut, Store,
};
use wasm_bridge_wasi::WasiCtxBuilder;

use crate::scripting::asset::Wasm;

use super::{
    commands::{ScriptCommand, ScriptResourceMap},
    host::wired_ecs::EcsWorld,
    state::ScriptState,
    stream::OutStream,
    Script, WasmEngine,
};

#[derive(Component)]
pub struct ScriptBinding(pub Arc<Mutex<Script>>);

#[derive(Component)]
pub struct ScriptOutput(pub Arc<Mutex<Receiver<Bytes>>>);

#[derive(Component)]
pub struct ScriptCommandReceiver(pub Arc<Mutex<Receiver<ScriptCommand>>>);

pub struct ScriptTaskOutput {
    pub binding: Arc<Mutex<Script>>,
    pub ecs_world: Resource<EcsWorld>,
    pub entity: Entity,
    pub store: Arc<Mutex<Store<ScriptState>>>,
}

// SAFETY: Script is used with Arc<Mutex<T>>, so uhhhh maybe this is fine?
unsafe impl Send for Script {}

#[derive(Component)]
pub struct ScriptStore(pub Arc<Mutex<Store<ScriptState>>>);

#[derive(Component)]
pub struct ScriptEcsWorld(pub Resource<EcsWorld>);

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
    #[error("Resource table error: {0}")]
    ResourceTable(anyhow::Error),
}

pub fn load_scripts(
    mut commands: Commands,
    mut pool: AsyncTaskPool<Result<ScriptTaskOutput, ScriptTaskErr>>,
    to_load: Query<(Entity, &Handle<Wasm>), Without<WasmEngine>>,
    wasm_assets: Res<Assets<Wasm>>,
) {
    for task in pool.iter_poll() {
        if let AsyncTaskStatus::Finished(res) = task {
            let output = match res {
                Ok(out) => out,
                Err(e) => {
                    error!("Error loading script: {}", e);
                    continue;
                }
            };

            commands.entity(output.entity).insert((
                ScriptBinding(output.binding),
                ScriptEcsWorld(output.ecs_world),
                ScriptStore(output.store),
            ));
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
            ScriptResourceMap::default(),
            engine.clone(),
        ));

        pool.spawn(async move {
            let wasi = WasiCtxBuilder::new().stdout(stream).build();

            let state = ScriptState {
                sender: Arc::new(Mutex::new(send_command)),
                table: Default::default(),
                wasi_table: Default::default(),
                wasi_ctx: wasi,
            };

            let ecs_world = state
                .table
                .lock()
                .await
                .push(EcsWorld)
                .map_err(|e| ScriptTaskErr::ResourceTable(anyhow::anyhow!(e)))?;

            let mut store = Store::new(&engine.0, state);

            let mut linker = Linker::new(&engine.0);

            wasm_bridge_wasi::command::add_to_linker(&mut linker)
                .map_err(ScriptTaskErr::AddWasiToLinker)?;

            super::host::add_to_linker(&mut linker).map_err(ScriptTaskErr::AddHostToLinker)?;

            let component = Component::new_safe(&engine.0, bytes)
                .await
                .map_err(ScriptTaskErr::Compile)?;

            let (script, _) =
                Script::instantiate_async(store.as_context_mut(), &component, &linker)
                    .await
                    .map_err(ScriptTaskErr::Instantiate)?;

            Ok(ScriptTaskOutput {
                binding: Arc::new(Mutex::new(script)),
                ecs_world,
                entity,
                store: Arc::new(Mutex::new(store)),
            })
        });
    }
}
