use bevy::prelude::*;

pub mod asset;
pub mod state;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(asset::WasmLoader)
            .init_asset::<asset::Wasm>();
    }
}

#[derive(Component)]
pub struct WasmRuntime;
