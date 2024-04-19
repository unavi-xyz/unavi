use bevy::prelude::*;

use self::{
    asset::Wasm,
    script::{
        commands::handle_script_commands, init_scripts, load::load_scripts,
        query::run_script_queries, update_scripts,
    },
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
            .add_systems(
                Update,
                (
                    load_scripts,
                    (
                        handle_script_commands,
                        run_script_queries,
                        init_scripts,
                        update_scripts,
                    )
                        .chain(),
                ),
            );
    }
}
