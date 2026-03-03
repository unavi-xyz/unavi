use std::sync::Arc;

use loro::LoroDoc;
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

impl RuntimeData {
    pub fn new(
        actor: Option<wds::actor::Actor>,
        blobs: Option<wds::Blobs>,
        doc: Arc<LoroDoc>,
        self_node_id: loro::TreeID,
    ) -> Self {
        Self {
            wired_scene: WiredSceneRt {
                actor,
                blobs,
                doc,
                self_node_id,
                table: ResourceTable::default(),
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
