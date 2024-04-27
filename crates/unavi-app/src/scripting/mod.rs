use std::sync::Arc;

use bevy::{prelude::*, utils::HashMap};
use tokio::sync::Mutex;

use self::{
    asset::Wasm, host::wired_ecs::QueriedEntity, load::WasmStores, resource_table::ResourceTable,
};

mod asset;
mod execution;
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
            .init_non_send_resource::<WasmStores>()
            .add_systems(Startup, unavi_system::spawn_unavi_system)
            .add_systems(
                Update,
                (
                    execution::init_scripts,
                    host::wired_ecs::add_wired_ecs_map,
                    host::wired_ecs::handle_wired_ecs_command,
                    load::load_scripts,
                    (
                        host::wired_ecs::run_script_queries,
                        execution::update_scripts,
                    )
                        .chain(),
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
    pub query_results: Arc<Mutex<HashMap<u32, Vec<QueriedEntity>>>>,
    pub resource_table: Arc<Mutex<ResourceTable>>,
}
