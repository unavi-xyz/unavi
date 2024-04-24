use bevy::prelude::*;
use wasm_component_layer::{AsContextMut, Component, Linker, Store, Value};
use wasm_runtime_layer::Engine;

use crate::scripting::{host::add_host_script_apis, script::get_script_interface, StoreData};

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
        let mut store = Store::new(&engine, StoreData::default());
        let mut linker = Linker::default();

        let receiver = match add_host_script_apis(&mut store, &mut linker) {
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

        let script = match get_script_interface(&mut store, &linker, &instance) {
            Ok(i) => i,
            Err(e) => {
                error!("Failed to get script interface: {}", e);
                continue;
            }
        };

        let ecs_world = script.ecs_world.borrow(store.as_context_mut()).unwrap();

        let mut results = vec![Value::U8(0)];

        if let Err(e) = script.init.call(
            store.as_context_mut(),
            &[Value::Borrow(ecs_world.clone())],
            &mut results,
        ) {
            error!("Failed to call script init: {}", e);
            continue;
        }

        info!("Script initialized!!!");

        let script_data = match &results[0] {
            Value::Own(own) => own,
            _ => {
                error!("Wrong script data value");
                continue;
            }
        };

        let script_data_borrow = match script_data.borrow(store.as_context_mut()) {
            Ok(s) => Value::Borrow(s),
            Err(e) => {
                error!("Failed to borrow script data: {}", e);
                continue;
            }
        };

        if let Err(e) = script.update.call(
            store.as_context_mut(),
            &[Value::Borrow(ecs_world), script_data_borrow],
            &mut [],
        ) {
            error!("Failed to call script update: {}", e);
            continue;
        }

        info!("Script updated!!!");

        commands.entity(entity).insert(receiver);
    }
}
