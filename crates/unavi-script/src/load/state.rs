use std::sync::{Arc, Mutex};

use bevy::prelude::Entity;
use bevy_hsd::{cache::SceneRegistryInner, hydrate::events::DocChange};
use loro::LoroDoc;
use smol_str::SmolStr;
use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

use crate::{
    agent::AgentDocEntry,
    api::wired::{agent::WiredAgentRt, scene::WiredSceneRt},
    permissions::ScriptPermissions,
};

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
        self_node_id: SmolStr,
        registry: Arc<SceneRegistryInner>,
        events: Arc<Mutex<Vec<DocChange>>>,
        perms: ScriptPermissions,
        agent_entry: Option<Arc<AgentDocEntry>>,
        doc_id: blake3::Hash,
        doc_entity: Entity,
    ) -> Self {
        Self {
            wired_agent: WiredAgentRt {
                local_agent: agent_entry,
                table: ResourceTable::default(),
            },
            wired_scene: WiredSceneRt {
                actor,
                blobs,
                doc,
                doc_entity,
                doc_id,
                self_node_id,
                table: ResourceTable::default(),
                registry,
                events,
                perms,
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
