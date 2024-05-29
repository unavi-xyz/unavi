use std::sync::{Arc, RwLock};

use bevy::prelude::*;

use self::{asset::Wasm, load::WasmStores, resource_table::ResourceTable};

mod asset;
mod execution;
mod host;
mod load;
mod resource_table;
mod script;
mod unavi_system;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(asset::WasmLoader)
            .init_asset::<Wasm>()
            .init_non_send_resource::<WasmStores>()
            .add_systems(Startup, unavi_system::spawn_unavi_system)
            .add_systems(
                Update,
                (
                    load::load_scripts,
                    (execution::init_scripts, execution::update_scripts).chain(),
                ),
            );
    }
}

#[derive(Bundle)]
struct ScriptBundle {
    name: Name,
    wasm: Handle<Wasm>,
}

#[derive(Default)]
pub struct StoreData {
    pub resource_table: Arc<RwLock<ResourceTable>>,
}
