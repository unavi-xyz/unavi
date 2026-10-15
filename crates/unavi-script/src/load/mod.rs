//! Loading script components, from `.wasm` assets or a prim's script
//! attribute.

use bevy::prelude::*;

pub mod asset;
mod hsd;

pub struct LoadPlugin;

impl Plugin for LoadPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(asset::WasmLoader)
            .init_asset::<asset::Wasm>()
            .add_observer(hsd::load_hsd_scripts);
    }
}
