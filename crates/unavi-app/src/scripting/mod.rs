use bevy::prelude::*;

use self::{
    asset::Wasm,
    script::{load_scripts, ScriptLoadQueue, WasmRuntime},
};

mod asset;
mod script;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(asset::WasmLoader)
            .init_asset::<Wasm>()
            .init_resource::<ScriptLoadQueue>()
            .init_resource::<WasmRuntime>()
            .add_systems(Startup, load_unavi_system)
            .add_systems(Update, load_scripts);
    }
}

fn load_unavi_system(mut load_queue: ResMut<ScriptLoadQueue>, asset_server: Res<AssetServer>) {
    let handle = asset_server.load("components/unavi_system.wasm");
    load_queue.0.push(handle);
}
