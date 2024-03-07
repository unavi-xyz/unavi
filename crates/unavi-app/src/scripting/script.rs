use bevy::{prelude::*, tasks::Task};
use wasm_bridge::{component::Linker, Config, Engine, Store};

use super::asset::Wasm;

#[cfg(target_family = "windows")]
wasm_bridge::component::bindgen!({
    path: "..\\..\\wired-protocol\\spatial\\wired-script\\world.wit",
    world: "script",
});

#[cfg(not(target_family = "windows"))]
wasm_bridge::component::bindgen!({
    path: "../../wired-protocol/spatial/wired-script/world.wit",
    world: "script",
});

#[derive(Resource, Default)]
pub struct ScriptLoadQueue(pub Vec<Handle<Wasm>>);

#[derive(Component)]
pub struct LoadComponentTask(Task<()>);

pub fn load_scripts(mut load_queue: ResMut<ScriptLoadQueue>, wasm_assets: Res<Assets<Wasm>>) {
    if load_queue.0.is_empty() {
        return;
    }

    let mut to_remove = Vec::new();

    for handle in &load_queue.0 {
        if let Some(wasm) = wasm_assets.get(handle) {
            to_remove.push(handle.clone());

            let mut config = Config::default();
            config.wasm_component_model(true);

            let engine = match Engine::new(&config) {
                Ok(e) => e,
                Err(e) => {
                    error!("Failed to create engine: {:?}", e);
                    continue;
                }
            };

            let mut store = Store::new(&engine, ());

            let component = match wasm_bridge::component::Component::new(&engine, &wasm.bytes) {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to load component: {:?}", e);
                    continue;
                }
            };

            let linker = Linker::new(&engine);

            let (script, _) = match Script::instantiate(&mut store, &component, &linker) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to instantiate component: {:?}", e);
                    continue;
                }
            };

            script.interface0.call_init(&mut store).unwrap();

            info!("Script initialized");
        }
    }

    for handle in to_remove {
        load_queue.0.retain(|h| h != &handle);
    }
}
