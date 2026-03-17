use std::sync::{Arc, Mutex};

use bevy::prelude::Entity;
use bevy_hsd::{cache::SceneRegistryInner, hydrate::events::ScriptQueuedEvent};
use loro::LoroDoc;
use smol_str::SmolStr;
use wasmtime_wasi::ResourceTable;

use crate::permissions::{HsdPermissions, ScriptPermissions};

pub mod document;
mod material;
mod mesh;
pub mod node;

pub mod bindings {
    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-scene",
        with: {
            "wired:scene/types.node": super::node::HostNode,
            "wired:scene/types.material": super::material::HostMaterial,
            "wired:scene/types.mesh": super::mesh::HostMesh,
            "wired:scene/types.document": super::document::HostDocument,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

pub struct WiredSceneRt {
    pub actor: Option<wds::actor::Actor>,
    pub blobs: Option<wds::Blobs>,
    pub doc: Arc<LoroDoc>,
    pub doc_entity: Entity,
    pub doc_id: blake3::Hash,
    pub events: Arc<Mutex<Vec<ScriptQueuedEvent>>>,
    pub perms: ScriptPermissions,
    pub registry: Arc<SceneRegistryInner>,
    pub self_node_id: SmolStr,
    pub table: ResourceTable,
}

impl WiredSceneRt {
    pub(super) fn push_script_event(&self, ev: ScriptQueuedEvent) {
        self.events.lock().expect("events lock").push(ev);
    }

    pub(super) fn check_hsd_write(&self) -> wasmtime::Result<()> {
        if self
            .perms
            .hsd
            .get(&self.doc_id)
            .is_some_and(|p| p.contains(&HsdPermissions::Write))
        {
            Ok(())
        } else {
            Err(anyhow::anyhow!("hsd write permission required"))
        }
    }
}

impl bindings::wired::scene::context::Host for WiredSceneRt {
    async fn self_node(
        &mut self,
    ) -> wasmtime::Result<wasmtime::component::Resource<bindings::wired::scene::context::Node>>
    {
        let inner = {
            self.registry
                .node_map
                .lock()
                .expect("node_map lock")
                .get(&self.self_node_id)
                .cloned()
        };
        if let Some(inner) = inner {
            let res = self.table.push(node::HostNode { inner })?;
            return Ok(res);
        }
        Err(anyhow::anyhow!("self_node not found in registry"))
    }

    async fn self_document(
        &mut self,
    ) -> wasmtime::Result<wasmtime::component::Resource<bindings::wired::scene::context::Document>>
    {
        let res = self
            .table
            .push(document::HostDocument { id: self.doc_id })?;
        Ok(res)
    }
}

impl bindings::wired::scene::types::Host for WiredSceneRt {}
