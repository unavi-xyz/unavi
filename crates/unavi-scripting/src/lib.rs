use bevy::prelude::*;
use crossbeam::channel::{Receiver, Sender};
use host::wired_gltf::WiredGltfAction;
use wasm_bridge::component::ResourceTable;
use wasm_bridge_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

use self::{asset::Wasm, load::Scripts};

pub mod asset;
mod execution;
mod host;
mod load;

mod wired_script {
    wasm_bridge::component::bindgen!({
        path: "../../wired-protocol/spatial/wit/wired-script",
        async: true,
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
                    (execution::init_scripts, execution::update_scripts).chain(),
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

pub struct StoreState {
    pub materials: Vec<u32>,
    pub meshes: Vec<u32>,
    pub name: String,
    pub nodes: Vec<u32>,
    pub primitives: Vec<u32>,
    pub sender: Sender<WiredGltfAction>,
    pub table: ResourceTable,
    pub wasi: WasiCtx,
    pub wasi_table: wasm_bridge_wasi::ResourceTable,
}

impl WasiView for StoreState {
    fn table(&mut self) -> &mut wasm_bridge_wasi::ResourceTable {
        &mut self.wasi_table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

impl StoreState {
    pub fn new(name: String) -> (Self, Receiver<WiredGltfAction>) {
        let (sender, recv) = crossbeam::channel::bounded(100);

        let wasi = WasiCtxBuilder::new().build();

        (
            Self {
                materials: Vec::default(),
                meshes: Vec::default(),
                name,
                nodes: Vec::default(),
                primitives: Vec::default(),
                sender,
                table: ResourceTable::default(),
                wasi,
                wasi_table: wasm_bridge_wasi::ResourceTable::default(),
            },
            recv,
        )
    }
}
