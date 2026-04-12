use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use bevy::prelude::{Command, Entity, World};
use bevy_hsd::{
    HsdDoc, HsdRecordId, cache::SceneRegistryInner, hydrate::events::ScriptCommandQueue,
};
use loro::LoroDoc;
use smol_str::SmolStr;
use tracing::warn;
use wasmtime::bail;
use wasmtime_wasi::ResourceTable;
use wired_schemas::schemas::SCHEMA_HSD;

use crate::firewall::{HsdFirewall, HsdFirewallInner};

macro_rules! mesh_attr {
    ($get:ident, $set:ident, $field:ident, $ty:ty) => {
        async fn $get(
            &mut self,
            self_: wasmtime::component::Resource<HostMesh>,
        ) -> wasmtime::Result<$ty> {
            let inner = std::sync::Arc::clone(&self.table.get(&self_)?.inner);
            Ok(inner.state.lock().expect("mesh state lock").$field.clone())
        }
        async fn $set(
            &mut self,
            self_: wasmtime::component::Resource<HostMesh>,
            values: $ty,
        ) -> wasmtime::Result<()> {
            let inner = std::sync::Arc::clone(&self.table.get(&self_)?.inner);
            let mut queue = self.command_queue.lock().expect("cmd queue lock");
            crate::core_ops::mesh::$set(&inner, self.doc_entity, values, &mut queue);
            Ok(())
        }
    };
}

macro_rules! material_setter {
    ($set:ident, $ty:ty) => {
        async fn $set(
            &mut self,
            self_: wasmtime::component::Resource<HostMaterial>,
            value: $ty,
        ) -> wasmtime::Result<()> {
            let inner = std::sync::Arc::clone(&self.table.get(&self_)?.inner);
            let mut queue = self.command_queue.lock().expect("cmd queue lock");
            crate::core_ops::material::$set(&inner, self.doc_entity, value, &mut queue);
            Ok(())
        }
    };
}

pub mod document;
mod material;
mod mesh;
pub mod node;

/// All data needed to operate on a specific HSD document from a script host call.
#[derive(Clone)]
pub struct DocHandle {
    pub registry: Arc<SceneRegistryInner>,
    pub doc_entity: Entity,
    pub firewall: Arc<RwLock<HsdFirewallInner>>,
}

/// Index of all live documents, keyed by their blake3 content ID.
///
/// Scripts use this to cross-reference foreign documents via `get_document()`.
/// Firewall checks (`foreign_perms`) are enforced before a handle is returned,
/// so entries here do not imply unconditional access.
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
    pub can_create_document: bool,
    pub doc: Arc<LoroDoc>,
    pub doc_entity: Entity,
    pub doc_id: blake3::Hash,
    pub command_queue: Arc<Mutex<ScriptCommandQueue>>,
    pub registry: Arc<SceneRegistryInner>,
    pub registry_map: GlobalRegistryMap,
    pub self_node_id: SmolStr,
    pub table: ResourceTable,
}

impl WiredSceneRt {
    pub(super) fn push_command<C: Command>(&self, cmd: C) {
        self.command_queue.lock().expect("cmd queue lock").push(cmd);
    }

    pub(super) fn get_doc_read(
        &self,
        res: &wasmtime::component::Resource<document::HostDocument>,
    ) -> wasmtime::Result<(Arc<SceneRegistryInner>, bool, bool, Entity)> {
        let d = self.table.get(res)?;
        if !d.can_read {
            bail!("hsd read permission required")
        }
        Ok((
            Arc::clone(&d.registry),
            d.can_read,
            d.can_write,
            d.doc_entity,
        ))
    }

    pub(super) fn foreign_perms(&self, foreign_id: blake3::Hash) -> (bool, bool) {
        let Some(hsd_fw) = self
            .registry_map
            .read()
            .expect("registry_map read")
            .get(&foreign_id)
            .map(|h| Arc::clone(&h.firewall))
        else {
            return (false, false);
        };
        let fw = hsd_fw.read().expect("hsd_fw read");
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
                doc_entity: self.doc_entity,
            })?;
            return Ok(res);
        }
        bail!("self_node not found in registry")
    }

    async fn self_document(
        &mut self,
    ) -> wasmtime::Result<wasmtime::component::Resource<bindings::wired::scene::context::Document>>
    {
        let res = self.table.push(document::HostDocument {
            id: self.doc_id,
            registry: Arc::clone(&self.registry),
            doc_entity: self.doc_entity,
            lor_doc: Some(Arc::clone(&self.doc)),
            entity_slot: None,
            is_public: false,
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
            warn!("script cannot read document {foreign_id}");
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
            warn!("document {foreign_id} not found");
            return Ok(None);
        };
        Ok(Some(self.table.push(document::HostDocument {
            id: foreign_id,
            registry: h.registry,
            doc_entity: h.doc_entity,
            lor_doc: None,
            entity_slot: None,
            is_public: false,
            can_read,
            can_write,
        })?))
    }

    async fn create_document(
        &mut self,
    ) -> wasmtime::Result<Result<wasmtime::component::Resource<document::HostDocument>, String>>
    {
        if !self.can_create_document {
            return Ok(Err("create-document permission required".into()));
        }
        let Some(actor) = self.actor.clone() else {
            return Ok(Err("create-document requires a WDS actor".into()));
        };

        // Build the new registry and firewall before the async work.
        let new_registry = SceneRegistryInner::new();

        // Inherit creator's firewall, then grant creator read+write on new doc.
        let new_firewall_inner = {
            let (mut read_set, mut write_set) = {
                let map = self.registry_map.read().expect("registry_map read");
                map.get(&self.doc_id).map_or_else(Default::default, |h| {
                    let fw = h.firewall.read().expect("firewall read");
                    (fw.read.clone(), fw.write.clone())
                })
            };
            read_set.insert(self.doc_id);
            write_set.insert(self.doc_id);
            Arc::new(RwLock::new(HsdFirewallInner {
                read: read_set,
                write: write_set,
            }))
        };

        // Create the WDS record with an empty HSD document.
        let result = actor
            .create_record()
            .add_schema("hsd", &*SCHEMA_HSD, |_doc| Ok(()))
            .map_err(|e| wasmtime::Error::msg(e.to_string()))?
            .ttl(Duration::from_hours(24))
            .send()
            .await
            .map_err(|e| wasmtime::Error::msg(format!("create record failed: {e}")))?;

        let record_id = result.id;
        let lor_doc = Arc::new(result.doc);

        // Entity slot: filled when the spawn command flushes, so that node/mesh
        // commands queued in the same tick can resolve the real entity.
        let entity_slot: Arc<Mutex<Option<Entity>>> = Arc::new(Mutex::new(None));
        let slot_clone = Arc::clone(&entity_slot);
        let lor_doc_spawn = Arc::clone(&lor_doc);
        let registry_spawn = Arc::clone(&new_registry);
        let firewall_spawn = Arc::clone(&new_firewall_inner);

        self.command_queue
            .lock()
            .expect("cmd queue lock")
            .push(move |world: &mut World| {
                let entity = world
                    .spawn((
                        HsdDoc(lor_doc_spawn),
                        HsdRecordId(record_id),
                        bevy_hsd::cache::SceneRegistry(registry_spawn),
                        HsdFirewall(firewall_spawn),
                    ))
                    .id();
                *slot_clone.lock().expect("entity slot lock") = Some(entity);
            });

        // Register in the global map so other scripts can find this doc.
        self.registry_map
            .write()
            .expect("registry_map write")
            .insert(
                record_id,
                DocHandle {
                    registry: Arc::clone(&new_registry),
                    doc_entity: Entity::PLACEHOLDER,
                    firewall: Arc::clone(&new_firewall_inner),
                },
            );

        let res = self.table.push(document::HostDocument {
            id: record_id,
            registry: new_registry,
            doc_entity: Entity::PLACEHOLDER,
            lor_doc: Some(lor_doc),
            entity_slot: Some(entity_slot),
            is_public: false,
            can_read: true,
            can_write: true,
        })?;

        Ok(Ok(res))
    }

    async fn remove_document(&mut self, id: Vec<u8>) -> wasmtime::Result<()> {
        let Ok(arr): Result<[u8; 32], _> = id.try_into() else {
            return Ok(());
        };
        let foreign_id = blake3::Hash::from(arr);

        // Only allow removal if the caller has write access or it's their own doc.
        let can_write = foreign_id == self.doc_id || self.foreign_perms(foreign_id).1;
        if !can_write {
            warn!("script cannot remove document {foreign_id}");
            return Ok(());
        }

        self.registry_map
            .write()
            .expect("registry_map write")
            .remove(&foreign_id);

        self.command_queue
            .lock()
            .expect("cmd queue lock")
            .push(move |world: &mut World| {
                // Query by HsdRecordId and despawn the matching entity.
                let entity = world
                    .query::<(Entity, &HsdRecordId)>()
                    .iter(world)
                    .find(|(_, r)| r.0 == foreign_id)
                    .map(|(e, _)| e);
                if let Some(e) = entity {
                    world.entity_mut(e).despawn();
                }
            });

        Ok(())
    }
}

impl bindings::wired::scene::types::Host for WiredSceneRt {}
