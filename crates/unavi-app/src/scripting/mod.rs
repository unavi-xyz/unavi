use crate::state::AppState;
use bevy::prelude::*;

pub mod asset;
mod lifecycle;
pub mod scripts;
pub mod state;

use scripts::WasmScript;

use self::scripts::ScriptRuntimeBundle;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(asset::WasmLoader)
            .init_asset::<asset::Wasm>()
            .add_systems(OnEnter(AppState::InWorld), setup_runtimes)
            .add_systems(
                Update,
                (
                    scripts::instantiate_scripts,
                    lifecycle::init_scripts,
                    lifecycle::update_scripts,
                )
                    .chain(),
            );
    }
}

pub fn setup_runtimes(mut commands: Commands, asset_server: Res<AssetServer>) {
    let unavi_system = commands
        .spawn(WasmScript::load(
            asset_server,
            "scripts/unavi_system.wasm".to_string(),
        ))
        .id();

    commands
        .spawn((Name::new("system"), ScriptRuntimeBundle::default()))
        .add_child(unavi_system);

    commands.spawn((Name::new("world"), ScriptRuntimeBundle::default()));
}
