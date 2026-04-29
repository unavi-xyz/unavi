use bevy::prelude::*;

pub mod asset;
mod hsd;
pub mod local;

pub struct LoadPlugin;

impl Plugin for LoadPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(asset::WasmLoader)
            .init_asset::<asset::Wasm>()
            .add_observer(hsd::spawn_hsd_scripts)
            .add_observer(hsd::despawn_hsd_scripts)
            .add_observer(local::load_local_script);
    }
}
