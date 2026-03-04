use std::{collections::HashMap, sync::Arc};

use bevy_vrm::BoneName;
use loro::{LoroDoc, TreeID};
use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

use crate::api::wired::{agent::WiredAgentRt, scene::WiredSceneRt};

pub struct StoreState {
    wasi: WasiCtx,
    resource_table: ResourceTable,
    pub rt: RuntimeData,
}

pub struct RuntimeData {
    pub wired_agent: WiredAgentRt,
    pub wired_scene: WiredSceneRt,
}

impl RuntimeData {
    #[must_use]
    pub fn new(
        actor: Option<wds::actor::Actor>,
        blobs: Option<wds::Blobs>,
        doc: Arc<LoroDoc>,
        self_node_id: TreeID,
        agent: Option<Arc<HashMap<BoneName, TreeID>>>,
    ) -> Self {
        Self {
            wired_agent: WiredAgentRt {
                bone_nodes: agent.unwrap_or_else(|| Arc::new(HashMap::new())),
                table: ResourceTable::default(),
            },
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
    #[must_use]
    pub fn new(wasi: WasiCtx, rt: RuntimeData) -> Self {
        Self {
            wasi,
            resource_table: ResourceTable::default(),
            rt,
        }
    }
}
