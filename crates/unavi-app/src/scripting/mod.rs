use bevy::prelude::*;

use self::{
    asset::Wasm,
    script::{load_scripts, log_script_output},
    unavi_system::spawn_unavi_system,
};

mod asset;
mod script;
mod unavi_system;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(asset::WasmLoader)
            .init_asset::<Wasm>()
            .add_systems(Startup, spawn_unavi_system)
            .add_systems(Update, (load_scripts, log_script_output));
    }
}
