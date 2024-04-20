use bevy::prelude::*;
use wasm_component_layer::{Component, Linker, Store};
use wasm_runtime_layer::Engine;

use crate::scripting::{host::add_host_script_apis, script::get_script_interface};

use super::asset::Wasm;

#[derive(Component)]
pub struct LoadedScript;

#[cfg(not(target_family = "wasm"))]
pub type EngineBackend = wasmtime::Engine;
#[cfg(target_family = "wasm")]
pub type EngineBackend = wasm_runtime_layer::web::Engine;

pub fn load_scripts(
    assets: Res<Assets<Wasm>>,
    mut commands: Commands,
    to_load: Query<(Entity, &Name, &Handle<Wasm>), Without<LoadedScript>>,
) {
    for (entity, name, handle) in to_load.iter() {
        let wasm = match assets.get(handle) {
            Some(a) => a,
            None => continue,
        };

        info!("Loading script: {}", name);

        commands.entity(entity).insert(LoadedScript);

        let engine = Engine::new(EngineBackend::default());
        let mut store = Store::new(&engine, ScriptData {});
        let mut linker = Linker::default();

        if let Err(e) = add_host_script_apis(&mut store, &mut linker) {
            error!("Failed to add host APIs: {}", e);
            continue;
        }

        let component = match Component::new(&engine, &wasm.0) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to create component: {}", e);
                continue;
            }
        };

        let instance = match linker.instantiate(&mut store, &component) {
            Ok(i) => i,
            Err(e) => {
                error!("Failed to instantiate component: {}", e);
                continue;
            }
        };

        let script = match get_script_interface(&instance) {
            Ok(i) => i,
            Err(e) => {
                error!("Failed to get script interface: {}", e);
                continue;
            }
        };
    }
}

pub struct ScriptData {}
