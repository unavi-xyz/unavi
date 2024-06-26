use std::sync::Arc;

use bevy::{prelude::*, utils::HashMap};
use bevy_async_task::{AsyncTaskPool, AsyncTaskStatus};
use tokio::sync::Mutex;
use wasm_bridge::{component::Linker, Config, Engine, Store};

use crate::{
    host::{add_host_script_apis, wired_gltf::WiredGltfReceiver},
    state::StoreState,
    wired_script::Script,
};

use super::asset::Wasm;

#[derive(Component)]
pub struct LoadedScript;

/// Script was processed to begin loading.
#[derive(Component)]
pub struct ProcessedScript;

#[derive(Default, Deref)]
pub struct Scripts(pub Arc<Mutex<HashMap<Entity, (Script, Store<StoreState>)>>>);

pub fn load_scripts(
    assets: Res<Assets<Wasm>>,
    mut commands: Commands,
    mut pool: AsyncTaskPool<anyhow::Result<Entity>>,
    scripts: NonSendMut<Scripts>,
    to_load: Query<(Entity, &Name, &Handle<Wasm>), Without<ProcessedScript>>,
) {
    for (entity, name, handle) in to_load.iter() {
        let wasm = match assets.get(handle) {
            Some(a) => a,
            None => continue,
        };

        info!("Loading script: {}", name);
        commands.entity(entity).insert(ProcessedScript);

        let mut config = Config::new();
        config.async_support(true);
        config.wasm_component_model(true);

        let engine = match Engine::new(&config) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to create engine: {}", e);
                continue;
            }
        };

        let (state, recv) = StoreState::new(name.to_string());
        let mut store = Store::new(&engine, state);
        let mut linker = Linker::new(store.engine());

        let bytes = wasm.0.clone();
        let scripts = scripts.clone();

        pool.spawn(async move {
            wasm_bridge_wasi::add_to_linker_async(&mut linker)?;
            add_host_script_apis(&mut linker)?;

            let component =
                wasm_bridge::component::Component::new_safe(store.engine(), &bytes).await?;

            let (script, _) =
                Script::instantiate_async(&mut store, &component, &mut linker).await?;

            let mut scripts = scripts.lock().await;
            scripts.insert(entity, (script, store));

            Ok(entity)
        });

        commands.entity(entity).insert(WiredGltfReceiver(recv));
    }

    for task in pool.iter_poll() {
        if let AsyncTaskStatus::Finished(res) = task {
            let entity = match res {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to load script: {}", e);
                    continue;
                }
            };

            info!("Loaded script!");

            commands.entity(entity).insert(LoadedScript);
        }
    }
}
