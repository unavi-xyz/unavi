use bevy::prelude::*;

use self::asset::Wasm;

mod asset;
mod host;
mod load;
mod script;
mod unavi_system;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(asset::WasmLoader)
            .init_asset::<Wasm>()
            .add_systems(Startup, unavi_system::spawn_unavi_system)
            .add_systems(Update, load::load_scripts);
    }
}

#[derive(Bundle)]
struct ScriptBundle {
    name: Name,
    wasm: Handle<Wasm>,
}
