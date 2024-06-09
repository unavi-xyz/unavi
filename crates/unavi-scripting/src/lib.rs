use std::sync::{Arc, RwLock};

use bevy::prelude::*;

use self::{asset::Wasm, load::WasmStores, resource_table::ResourceTable};

pub mod asset;
mod execution;
mod host;
mod load;
mod resource_table;
mod script;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(asset::WasmLoader)
            .init_asset::<Wasm>()
            .init_non_send_resource::<WasmStores>()
            .add_systems(
                FixedUpdate,
                (
                    (
                        host::wired_gltf::query::query_node_data,
                        execution::init_scripts,
                        execution::update_scripts,
                    )
                        .chain(),
                    host::wired_gltf::handler::handle_wired_gltf_actions,
                    load::load_scripts,
                ),
            );
    }
}

#[derive(Bundle)]
pub struct ScriptBundle {
    pub name: Name,
    pub wasm: Handle<Wasm>,
}

#[derive(Default)]
pub struct StoreData {
    pub resource_table: Arc<RwLock<ResourceTable>>,
}

/// Marks an entity as being "owned" by another entity.
/// For example, entities spawned in by a script are owned by that script.
#[derive(Component, Deref)]
pub struct Owner(pub Entity);
