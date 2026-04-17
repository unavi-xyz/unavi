use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use bevy_hsd::HsdRecordId;
use blake3::Hash;

use crate::firewall::HsdFirewall;

pub type ReceptorQueue = Arc<Mutex<VecDeque<QueuedEvent>>>;

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

pub struct ReceptorEntry {
    pub doc_id: Hash,
    pub queue: ReceptorQueue,
    pub filter: ReceptorFilter,
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
pub struct InnerEventRegistry {
    pub receptors: HashMap<String, Vec<ReceptorEntry>>,
    pub pending: Vec<PendingEmission>,
}

impl InnerEventRegistry {
    pub fn register_node(
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

    pub fn register_global(
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

    pub fn remove_receptor(&mut self, queue: &ReceptorQueue) {
        for entries in self.receptors.values_mut() {
            entries.retain(|e| !Arc::ptr_eq(&e.queue, queue));
        }
        self.receptors.retain(|_, v| !v.is_empty());
    }

    pub fn push_emission(&mut self, emission: PendingEmission) {
        self.pending.push(emission);
    }

    pub fn drain_pending(&mut self) -> Vec<PendingEmission> {
        std::mem::take(&mut self.pending)
    }
}

#[derive(Resource, Clone, Default)]
pub struct EventRegistry(pub Arc<Mutex<InnerEventRegistry>>);

// Events are buffered during script tick, then dispatched in a single pass:
// 1. target_documents filter (emitter-side allowlist)
// 2. firewall check (receiver document must allow sender)
// 3. source_documents filter (receptor-side allowlist)
// 4. spatial distance check (both emitter and receptor radii must contain each other)
// Running after tick ensures same-frame emissions arrive next frame.
pub fn process_event_emissions(
    registry: Res<EventRegistry>,
    transforms: Query<&GlobalTransform>,
    firewalls: Query<(&HsdRecordId, &HsdFirewall)>,
) {
    let pending = {
        let mut inner = registry.0.lock().expect("registry lock");
        inner.drain_pending()
    };

    if pending.is_empty() {
        return;
    }

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as u64);

    for emission in &pending {
        let origin: Option<Vec3> = emission
            .node
            .and_then(|e| transforms.get(e).ok().map(GlobalTransform::translation));

        let matched: Vec<_> = {
            let inner = registry.0.lock().expect("registry lock");
            let Some(entries) = inner.receptors.get(&emission.channel) else {
                continue;
            };
            let result: Vec<_> = entries
                .iter()
                .filter_map(|entry| {
                    if let Some(docs) = &emission.target_documents
                        && !docs.contains(&entry.doc_id)
                    {
                        return None;
                    }

                    if let Some((_, fw)) = firewalls.iter().find(|(id, _)| id.0 == entry.doc_id) {
                        let Ok(fw) = fw.0.read() else {
                            error!("firewall poisoned");
                            return None;
                        };
                        if !fw.event_receive.permits(&emission.sender_doc_id) {
                            return None;
                        }
                    }

                    match &entry.filter {
                        ReceptorFilter::Global { source_documents } => {
                            if let Some(docs) = source_documents
                                && !docs.contains(&emission.sender_doc_id)
                            {
                                return None;
                            }
                            Some(Arc::clone(&entry.queue))
                        }
                        ReceptorFilter::Spatial {
                            entity,
                            radius,
                            source_documents,
                        } => {
                            if let Some(docs) = source_documents
                                && !docs.contains(&emission.sender_doc_id)
                            {
                                return None;
                            }
                            let origin = origin?;
                            let t = transforms.get(*entity).ok()?;
                            let dist = origin.distance(t.translation());
                            (dist <= emission.radius && dist <= *radius)
                                .then(|| Arc::clone(&entry.queue))
                        }
                    }
                })
                .collect();
            drop(inner);
            result
        };

        for queue in matched {
            queue.lock().expect("queue lock").push_back(QueuedEvent {
                channel: emission.channel.clone(),
                payload: emission.payload.clone(),
                sender_node: emission.node,
                sender_document: emission.sender_doc_id,
                time,
            });
        }
    }
}
