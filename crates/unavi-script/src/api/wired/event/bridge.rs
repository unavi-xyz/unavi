use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::prelude::*;
use bevy_hsd::HsdRecordId;

use crate::firewall::HsdFirewall;

use super::registry::{EventRegistry, QueuedEvent, ReceptorFilter};

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
                        if !fw.read.iter().any(|h| *h == emission.sender_doc_id) {
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
