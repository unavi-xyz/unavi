use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use bevy::prelude::Entity;
use bevy_hsd::{cache::SceneRegistryInner, hydrate::events::ScriptQueuedEvent};
use loro::LoroDoc;
use smol_str::SmolStr;
use wasmtime_wasi::ResourceTable;

pub mod document;
mod hsd_firewall;
mod material;
mod mesh;
pub mod node;

pub use hsd_firewall::{HsdFirewall, HsdFirewallInner};

/// All data needed to operate on a specific HSD document from a script host call.
#[derive(Clone)]
pub struct DocHandle {
    pub registry: Arc<SceneRegistryInner>,
    pub events: Arc<Mutex<Vec<ScriptQueuedEvent>>>,
    pub hsd_fw: Arc<RwLock<HsdFirewallInner>>,
}

/// Shared map from `doc_id` → `DocHandle`, accessible from script host calls.
pub type GlobalRegistryMap = Arc<RwLock<HashMap<blake3::Hash, DocHandle>>>;

/// Bevy resource wrapping the shared registry map.
#[derive(bevy::prelude::Resource, Clone, Default)]
pub struct GlobalRegistryMapRes(pub GlobalRegistryMap);

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
    pub registry: Arc<SceneRegistryInner>,
    pub registry_map: GlobalRegistryMap,
    pub self_node_id: SmolStr,
    pub table: ResourceTable,
}

impl WiredSceneRt {
    pub(super) fn push_script_event(&self, ev: ScriptQueuedEvent) {
        self.events.lock().expect("events lock").push(ev);
    }

    /// Returns (`can_read`, `can_write`) for a foreign document by checking its
    /// `HsdFirewall` for this script's `doc_id`.
    pub(super) fn foreign_perms(&self, foreign_id: blake3::Hash) -> (bool, bool) {
        let map = self.registry_map.read().expect("registry_map read");
        let Some(h) = map.get(&foreign_id) else {
            return (false, false);
        };
        let fw = h.hsd_fw.read().expect("hsd_fw read");
        (
            fw.read.contains(&self.doc_id),
            fw.write.contains(&self.doc_id),
        )
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
            let res = self.table.push(node::HostNode {
                inner,
                can_read: true,
                can_write: true,
            })?;
            return Ok(res);
        }
        Err(anyhow::anyhow!("self_node not found in registry"))
    }

    async fn self_document(
        &mut self,
    ) -> wasmtime::Result<wasmtime::component::Resource<bindings::wired::scene::context::Document>>
    {
        let res = self.table.push(document::HostDocument {
            id: self.doc_id,
            registry: Arc::clone(&self.registry),
            events: Arc::clone(&self.events),
            can_read: true,
            can_write: true,
        })?;
        Ok(res)
    }

    async fn get_document(
        &mut self,
        id: Vec<u8>,
    ) -> wasmtime::Result<
        Option<wasmtime::component::Resource<bindings::wired::scene::context::Document>>,
    > {
        let Ok(arr): Result<[u8; 32], _> = id.try_into() else {
            return Ok(None);
        };
        let foreign_id = blake3::Hash::from(arr);
        // Own doc is always accessible.
        let (can_read, can_write) = if foreign_id == self.doc_id {
            (true, true)
        } else {
            self.foreign_perms(foreign_id)
        };
        if !can_read {
            return Ok(None);
        }
        let handle = {
            self.registry_map
                .read()
                .expect("registry_map read")
                .get(&foreign_id)
                .cloned()
        };
        let Some(h) = handle else {
            return Ok(None);
        };
        Ok(Some(self.table.push(document::HostDocument {
            id: foreign_id,
            registry: h.registry,
            events: h.events,
            can_read,
            can_write,
        })?))
    }
}

impl bindings::wired::scene::types::Host for WiredSceneRt {}
