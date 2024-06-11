use bevy::{prelude::*, utils::HashMap};
use wasm_component_layer::{AsContextMut, Component, Linker, Store};
use wasm_runtime_layer::Engine;

use crate::{host::add_host_script_apis, script::get_script_interface, StoreData};

use super::asset::Wasm;

#[derive(Component)]
pub struct LoadedScript;

#[cfg(not(target_family = "wasm"))]
pub type EngineBackend = wasmtime::Engine;
#[cfg(target_family = "wasm")]
pub type EngineBackend = wasmi::Engine;

/// Store is `!Send` for the web backend.
/// Because of this [WasmStores] is a [NonSend] resource, but we could change
/// this when targeting native to allow for parallel script execution.
#[derive(Default)]
pub struct WasmStores(pub HashMap<Entity, Store<StoreData, EngineBackend>>);

pub fn load_scripts(
    assets: Res<Assets<Wasm>>,
    mut commands: Commands,
    mut stores: NonSendMut<WasmStores>,
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
        let mut store = Store::new(&engine, StoreData::default());
        let mut linker = Linker::default();

        let host = match add_host_script_apis(&mut store, &mut linker) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to add host APIs: {}", e);
                continue;
            }
        };

        let component = match Component::new(&engine, &wasm.0) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to create component: {}", e);
                continue;
            }
        };

        let instance = match linker.instantiate(store.as_context_mut(), &component) {
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

        stores.0.insert(entity, store);

        commands
            .entity(entity)
            .insert((host.wired_gltf_components, script));
    }
}
