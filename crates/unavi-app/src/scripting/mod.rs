use std::sync::Arc;

use bevy::prelude::*;
use tokio::sync::Mutex;

use self::{asset::Wasm, resource_table::ResourceTable};

mod asset;
mod host;
mod load;
mod resource_table;
mod script;
mod unavi_system;
mod util;

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

#[derive(Default)]
pub struct StoreData {
    pub resource_table: Arc<Mutex<ResourceTable>>,
}
