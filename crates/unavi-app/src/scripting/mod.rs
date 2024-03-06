use bevy::prelude::*;

use self::script::{load_components, load_scripts, ScriptsLoadQueue, WasmRuntime};

mod asset;
mod script;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(asset::WasmLoader)
            .init_asset::<asset::Wasm>()
            .insert_resource(ScriptsLoadQueue::default())
            .insert_resource(WasmRuntime::default())
            .add_systems(Startup, load_unavi_system)
            .add_systems(Update, (load_scripts, load_components));
    }
}

fn load_unavi_system(mut load_queue: ResMut<ScriptsLoadQueue>, asset_server: Res<AssetServer>) {
    info!("Loading unavi-system.wasm");
    let handle = asset_server.load("unavi-system.wasm");
    load_queue.0.push(handle);
}
