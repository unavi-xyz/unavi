use bevy::{
    asset::{Assets, Handle},
    core::Name,
    ecs::{
        component::Component,
        entity::Entity,
        query::Without,
        system::{Commands, NonSendMut, Query, Res},
    },
    log::{error, info},
    utils::HashMap,
};
use wasm_bridge::{component::Linker, Config, Engine, Store};

use crate::{
    host::{add_host_script_apis, wired_gltf::WiredGltfReceiver},
    wired_script::Script,
    StoreState,
};

use super::asset::Wasm;

#[derive(Component)]
pub struct LoadedScript;

/// This is `!Send` for the web backend.
/// Because of this we use a [NonSend] resource, but we could change
/// this when targeting native to allow for parallel script execution.
#[derive(Default)]
pub struct Scripts(pub HashMap<Entity, (Script, Store<StoreState>)>);

pub fn load_scripts(
    assets: Res<Assets<Wasm>>,
    mut commands: Commands,
    mut stores: NonSendMut<Scripts>,
    to_load: Query<(Entity, &Name, &Handle<Wasm>), Without<LoadedScript>>,
) {
    for (entity, name, handle) in to_load.iter() {
        let wasm = match assets.get(handle) {
            Some(a) => a,
            None => continue,
        };

        info!("Loading script: {}", name);

        let config = Config::new();

        let engine = match Engine::new(&config) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to create engine: {}", e);
                continue;
            }
        };

        let mut linker = Linker::new(&engine);

        let (state, recv) = StoreState::new(name.to_string());
        let mut store = Store::new(&engine, state);

        match add_host_script_apis(&mut linker) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to add host APIs: {}", e);
                continue;
            }
        };

        // TODO: Move to async loading + instantiation
        #[allow(deprecated)]
        let component = match wasm_bridge::component::Component::new(&engine, &wasm.0) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to create component: {}", e);
                continue;
            }
        };

        #[allow(deprecated)]
        let instance = match linker.instantiate(&mut store, &component) {
            Ok(i) => i,
            Err(e) => {
                error!("Failed to instantiate component: {}", e);
                continue;
            }
        };

        let script = match super::wired_script::Script::new(&mut store, &instance) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to construct script resource: {}", e);
                continue;
            }
        };

        stores.0.insert(entity, (script, store));
        commands
            .entity(entity)
            .insert((LoadedScript, WiredGltfReceiver(recv)));
    }
}
