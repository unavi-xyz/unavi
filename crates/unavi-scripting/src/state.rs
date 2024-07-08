use crossbeam::channel::{Receiver, Sender};
use wasm_bridge::component::{Resource, ResourceTable};
use wasm_bridge_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

use crate::{
    actions::ScriptAction,
    api::wired_scene::gltf::{
        material::Material,
        mesh::{Mesh, Primitive},
        node::Node,
    },
};

pub struct StoreState {
    pub materials: Vec<Resource<Material>>,
    pub meshes: Vec<Resource<Mesh>>,
    pub name: String,
    pub nodes: Vec<Resource<Node>>,
    pub primitives: Vec<Resource<Primitive>>,
    pub sender: Sender<ScriptAction>,
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
    pub fn new(name: String) -> (Self, Receiver<ScriptAction>) {
        let (sender, recv) = crossbeam::channel::bounded(255);

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
