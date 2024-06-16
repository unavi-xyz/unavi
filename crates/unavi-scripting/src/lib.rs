use bevy::prelude::*;
use wasm_bridge::component::ResourceTable;

use self::{asset::Wasm, load::Scripts};

pub mod asset;
mod execution;
mod host;
mod load;

mod wired_script {
    wasm_bridge::component::bindgen!({
        path: "../../wired-protocol/spatial/wit/wired-script"
    });
}

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(asset::WasmLoader)
            .init_asset::<Wasm>()
            .init_non_send_resource::<Scripts>()
            .add_systems(
                FixedUpdate,
                (
                    (
                        // host::wired_gltf::query::query_node_data,
                        execution::init_scripts,
                        execution::update_scripts,
                    )
                        .chain(),
                    // host::wired_gltf::handler::handle_wired_gltf_actions,
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

impl ScriptBundle {
    /// Loads a give WASM script from the assets folder, in the `namespace:package` format.
    pub fn load(name: &str, asset_server: &AssetServer) -> Self {
        let (namespace, package) = name.split_once(':').unwrap();

        let path = format!(
            "components/{}/{}/{}.wasm",
            env!("CARGO_PKG_VERSION"),
            namespace,
            package
        );
        let wasm = asset_server.load(path);

        Self {
            name: name.into(),
            wasm,
        }
    }
}

#[derive(Default)]
pub struct State {
    pub name: String,
    pub table: ResourceTable,
    pub materials: Vec<u32>,
    pub meshes: Vec<u32>,
    pub nodes: Vec<u32>,
}
