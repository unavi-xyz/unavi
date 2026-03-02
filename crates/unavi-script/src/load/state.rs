use std::sync::Arc;

use loro::{LoroDoc, LoroTree, TreeParentId};
use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

use crate::api::wired::scene::WiredSceneRt;

pub struct StoreState {
    wasi: WasiCtx,
    resource_table: ResourceTable,
    pub rt: RuntimeData,
}

pub struct RuntimeData {
    pub wired_scene: WiredSceneRt,
}

pub struct RuntimeDataResult {
    pub rt: RuntimeData,
}

impl RuntimeData {
    pub fn spawn(actor: Option<wds::actor::Actor>, blobs: Option<wds::Blobs>) -> RuntimeDataResult {
        let doc = Arc::new(LoroDoc::new()); // TODO

        let self_node_id = doc
            .get_map("hsd")
            .get_or_create_container("nodes", LoroTree::new())
            .expect("node tree")
            .create(TreeParentId::Root)
            .expect("create self node");

        RuntimeDataResult {
            rt: Self {
                wired_scene: WiredSceneRt {
                    actor,
                    blobs,
                    doc,
                    self_node_id,
                    table: ResourceTable::default(),
                },
            },
        }
    }
}

impl WasiView for StoreState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.resource_table,
        }
    }
}

impl StoreState {
    pub fn new(wasi: WasiCtx, rt: RuntimeData) -> Self {
        Self {
            wasi,
            resource_table: ResourceTable::default(),
            rt,
        }
    }
}
