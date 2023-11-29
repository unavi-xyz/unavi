use bevy::prelude::*;

pub mod asset;
pub mod state;

pub struct WasmRuntimePlugin;

impl Plugin for WasmRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(asset::ScriptLoader)
            .init_asset::<asset::Script>();
    }
}
