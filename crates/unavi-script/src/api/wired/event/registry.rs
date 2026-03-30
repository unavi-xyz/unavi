use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use bevy::prelude::*;

pub(super) type ReceptorQueue = Arc<Mutex<VecDeque<QueuedEvent>>>;

#[derive(Clone)]
pub struct QueuedEvent {
    pub channel: String,
    pub payload: Vec<u8>,
    #[expect(dead_code, reason = "reserved for event tracing")]
    pub sender_node: Option<Entity>,
    pub sender_document: Vec<u8>,
}

pub enum ReceptorFilter {
    Global {
        source_documents: Vec<Vec<u8>>,
    },
    Spatial {
        entity: Entity,
        radius: f32,
        source_documents: Vec<Vec<u8>>,
    },
}

pub(super) struct ReceptorEntry {
    pub(super) doc_id: Vec<u8>,
    pub(super) queue: ReceptorQueue,
    pub(super) filter: ReceptorFilter,
}

pub struct PendingEmission {
    /// `None` for global emitter (no spatial origin).
    pub node: Option<Entity>,
    pub channel: String,
    pub payload: Vec<u8>,
    /// Emit radius in world units.
    pub radius: f32,
    pub sender_doc_id: Vec<u8>,
    /// Empty = broadcast to all matching receptors.
    pub target_documents: Vec<Vec<u8>>,
}

#[derive(Default)]
pub(super) struct InnerEventRegistry {
    /// channel → list of receptors
    pub(super) receptors: HashMap<String, Vec<ReceptorEntry>>,
    pub(super) pending: Vec<PendingEmission>,
}

impl InnerEventRegistry {
    pub(super) fn register_node(
        &mut self,
        entity: Entity,
        channels: Vec<String>,
        radius: f32,
        source_documents: Vec<Vec<u8>>,
        doc_id: Vec<u8>,
    ) -> ReceptorQueue {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        for channel in channels {
            self.receptors
                .entry(channel)
                .or_default()
                .push(ReceptorEntry {
                    doc_id: doc_id.clone(),
                    queue: Arc::clone(&queue),
                    filter: ReceptorFilter::Spatial {
                        entity,
                        radius,
                        source_documents: source_documents.clone(),
                    },
                });
        }
        queue
    }

    pub(super) fn register_global(
        &mut self,
        channels: Vec<String>,
        source_documents: Vec<Vec<u8>>,
        doc_id: Vec<u8>,
    ) -> ReceptorQueue {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        for channel in channels {
            self.receptors
                .entry(channel)
                .or_default()
                .push(ReceptorEntry {
                    doc_id: doc_id.clone(),
                    queue: Arc::clone(&queue),
                    filter: ReceptorFilter::Global {
                        source_documents: source_documents.clone(),
                    },
                });
        }
        queue
    }

    pub(super) fn remove_receptor(&mut self, queue: &ReceptorQueue) {
        for entries in self.receptors.values_mut() {
            entries.retain(|e| !Arc::ptr_eq(&e.queue, queue));
        }
        self.receptors.retain(|_, v| !v.is_empty());
    }

    pub(super) fn push_emission(&mut self, emission: PendingEmission) {
        self.pending.push(emission);
    }

    pub(super) fn drain_pending(&mut self) -> Vec<PendingEmission> {
        std::mem::take(&mut self.pending)
    }
}

#[derive(Resource, Clone, Default)]
pub struct EventRegistry(pub(super) Arc<Mutex<InnerEventRegistry>>);
