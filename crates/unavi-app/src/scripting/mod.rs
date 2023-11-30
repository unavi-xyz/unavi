use bevy::prelude::*;

use crate::state::AppState;

pub mod asset;
pub mod state;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(asset::WasmLoader)
            .init_asset::<asset::Wasm>()
            .add_systems(OnEnter(AppState::InWorld), setup_runtimes);
    }
}

#[derive(Component)]
pub struct WasmRuntime;

fn setup_runtimes(mut commands: Commands) {
    commands.spawn((Name::new("system"), WasmRuntime));
    commands.spawn((Name::new("world"), WasmRuntime));
}
