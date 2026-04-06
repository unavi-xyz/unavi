use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use bevy::prelude::*;
use blake3::Hash;

pub(super) type ReceptorQueue = Arc<Mutex<VecDeque<QueuedEvent>>>;

#[derive(Clone)]
pub struct QueuedEvent {
    pub channel: String,
    pub payload: Vec<u8>,
    pub sender_node: Option<Entity>,
    pub sender_document: Hash,
    pub time: u64,
}

pub enum ReceptorFilter {
    Global {
        source_documents: Option<Vec<Hash>>,
    },
    Spatial {
        entity: Entity,
        radius: f32,
        source_documents: Option<Vec<Hash>>,
    },
}

pub(super) struct ReceptorEntry {
    pub(super) doc_id: Hash,
    pub(super) queue: ReceptorQueue,
    pub(super) filter: ReceptorFilter,
}

pub struct PendingEmission {
    pub node: Option<Entity>,
    pub channel: String,
    pub payload: Vec<u8>,
    pub radius: f32,
    pub sender_doc_id: Hash,
    pub target_documents: Option<Vec<Hash>>,
}

#[derive(Default)]
pub(super) struct InnerEventRegistry {
    pub(super) receptors: HashMap<String, Vec<ReceptorEntry>>,
    pub(super) pending: Vec<PendingEmission>,
}

impl InnerEventRegistry {
    pub(super) fn register_node(
        &mut self,
        entity: Entity,
        channels: Vec<String>,
        radius: f32,
        source_documents: Option<Vec<Hash>>,
        doc_id: Hash,
    ) -> ReceptorQueue {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        for channel in channels {
            self.receptors
                .entry(channel)
                .or_default()
                .push(ReceptorEntry {
                    doc_id,
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
        source_documents: Option<Vec<Hash>>,
        doc_id: Hash,
    ) -> ReceptorQueue {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        for channel in channels {
            self.receptors
                .entry(channel)
                .or_default()
                .push(ReceptorEntry {
                    doc_id,
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
