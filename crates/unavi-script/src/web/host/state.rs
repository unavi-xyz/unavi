use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;

use bevy::prelude::Entity;
use bevy_hsd::cache::{MaterialInner, MeshInner, NodeInner, SceneRegistryInner};
use bevy_hsd::hydrate::events::ScriptCommandQueue;
use smol_str::SmolStr;

use crate::core_ops::document::DocEntityRef;
use crate::event_registry::{EventRegistry, ReceptorQueue};
use crate::input_registry::{InputRegistry, ListenerQueue};

pub struct NodeEntry {
    pub inner: Arc<NodeInner>,
    pub doc_entity: Entity,
}

pub struct DocEntry {
    pub id: blake3::Hash,
    pub registry: Arc<SceneRegistryInner>,
    pub doc_entity: Entity,
    pub entity_slot: Option<Arc<Mutex<Option<Entity>>>>,
    pub is_public: bool,
    pub can_read: bool,
    pub can_write: bool,
}

impl DocEntry {
    pub fn doc_ref(&self) -> DocEntityRef {
        self.entity_slot.as_ref().map_or(
            DocEntityRef::Immediate(self.doc_entity),
            |slot| DocEntityRef::Slot(Arc::clone(slot)),
        )
    }
}

pub struct MeshEntry {
    pub inner: Arc<MeshInner>,
    pub doc_entity: Entity,
}

pub struct MatEntry {
    pub inner: Arc<MaterialInner>,
    pub doc_entity: Entity,
}

pub struct WdsQueryFuture {
    pub rx: mpsc::Receiver<anyhow::Result<Vec<blake3::Hash>>>,
}

pub struct WdsReadFuture {
    pub rx: mpsc::Receiver<anyhow::Result<WdsRecordOut>>,
}

pub struct WdsRecordOut {
    pub id: blake3::Hash,
    pub creator: String,
    pub schemas: Vec<blake3::Hash>,
    pub containers: Vec<(String, Vec<u8>)>,
}

pub struct WebScriptState {
    pub registry: Arc<SceneRegistryInner>,
    pub command_queue: ScriptCommandQueue,
    pub doc_entity: Entity,
    pub doc_id: blake3::Hash,
    pub self_node_id: SmolStr,
    pub camera_node_id: SmolStr,
    pub event_registry: EventRegistry,
    pub input_registry: InputRegistry,
    pub wds_actor: Option<wds::actor::Actor>,
    pub can_create_document: bool,
    pub next_rep: u32,
    pub nodes: HashMap<u32, NodeEntry>,
    pub docs: HashMap<u32, DocEntry>,
    pub meshes: HashMap<u32, MeshEntry>,
    pub mats: HashMap<u32, MatEntry>,
    pub receptors: HashMap<u32, ReceptorQueue>,
    pub listeners: HashMap<u32, (ListenerQueue, Option<Entity>)>,
    pub wds_instances: HashMap<u32, wds::actor::Actor>,
    pub wds_query_futures: HashMap<u32, WdsQueryFuture>,
    pub wds_read_futures: HashMap<u32, WdsReadFuture>,
}

impl WebScriptState {
    pub fn alloc(&mut self) -> u32 {
        self.next_rep += 1;
        self.next_rep
    }
}
