use std::{
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::UNIX_EPOCH,
};

use async_channel::Receiver;

use crate::{
    firewall::Channel,
    runtime::shared::{
        Api,
        registry::{
            event::{
                EVENT_RECEPTOR_REGISTRY, InboundEvent, ReceptorEntry, ReceptorScope, SenderScope,
            },
            firewall::validate_firewall,
            transform::{AbsoluteNodeId, NODE_TRANSFORM_REGISTRY},
        },
        slot_map::SlotMap,
    },
};

static NEXT_RECEPTOR_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Default)]
pub struct EventFilter {
    pub documents: Option<Vec<Vec<u8>>>,
    pub scope: EventScope,
}

#[derive(Default)]
pub enum EventScope {
    #[default]
    Global,
    Spatial {
        node: u32,
        radius: f32,
    },
}

pub struct EventReceptorRes {
    rx: Receiver<InboundEvent>,
}

#[derive(Default)]
pub struct WiredEventApi {
    pub receptors: SlotMap<EventReceptorRes>,
}

const RECEPTOR_CAPACITY: usize = 16;

pub async fn emit(
    api: &Api,
    channel: String,
    payload: Vec<u8>,
    filter: EventFilter,
) -> anyhow::Result<()> {
    let time = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    let emitter_spatial: Option<(AbsoluteNodeId, Option<bevy::math::Vec3>, f32)> =
        match &filter.scope {
            EventScope::Spatial { node, radius } => {
                let scene = api.wired_scene.lock().await;
                let abs = scene
                    .prims
                    .get(*node)
                    .map(|res| AbsoluteNodeId {
                        doc: res.doc_id,
                        node: res.id,
                    })
                    .ok_or_else(|| anyhow::anyhow!("emit: node not found"))?;
                drop(scene);
                let pos = NODE_TRANSFORM_REGISTRY
                    .read()
                    .get(&abs)
                    .map(|s| s.global.translation());
                Some((abs, pos, *radius))
            }
            EventScope::Global => None,
        };

    let payload = Arc::new(payload);
    let sender_doc = api.doc_id.as_bytes().to_vec();

    let registry = EVENT_RECEPTOR_REGISTRY.read();
    for entry in registry.values() {
        if !entry.channels.iter().any(|c| c == &channel) {
            continue;
        }

        if let Some(docs) = &filter.documents
            && !docs.iter().any(|d| d.as_slice() == entry.doc_id.as_bytes())
        {
            continue;
        }

        if let Some(docs) = &entry.source_documents
            && !docs.iter().any(|d| d.as_slice() == api.doc_id.as_bytes())
        {
            continue;
        }

        if validate_firewall(&api.doc_id, &entry.doc_id, Channel::EventWrite).is_err() {
            continue;
        }

        let sender_scope = match (&emitter_spatial, &entry.scope) {
            (None, ReceptorScope::Global) => SenderScope::Global,
            (None, ReceptorScope::Spatial { .. }) => continue,
            (Some((abs, ..)), ReceptorScope::Global) => SenderScope::Spatial {
                distance: 0.0,
                node: abs.clone(),
            },
            (
                Some((emitter_abs, emitter_pos, emitter_radius)),
                ReceptorScope::Spatial {
                    node: receptor_node,
                    radius: receptor_radius,
                },
            ) => {
                let Some(e_pos) = emitter_pos else {
                    continue;
                };
                let Some(r_pos) = NODE_TRANSFORM_REGISTRY
                    .read()
                    .get(receptor_node)
                    .map(|s| s.global.translation())
                else {
                    continue;
                };
                let dist = (*e_pos - r_pos).length();
                if dist > *emitter_radius || dist > *receptor_radius {
                    continue;
                }
                SenderScope::Spatial {
                    distance: dist,
                    node: emitter_abs.clone(),
                }
            }
        };

        let _ = entry.tx.try_send(InboundEvent {
            channel: channel.clone(),
            payload: Arc::clone(&payload),
            sender_document: sender_doc.clone(),
            sender_scope,
            time,
        });
    }
    drop(registry);

    Ok(())
}

pub async fn listen(api: &Api, channels: Vec<String>, filter: EventFilter) -> anyhow::Result<u32> {
    let (tx, rx) = async_channel::bounded(RECEPTOR_CAPACITY);

    let scope = match filter.scope {
        EventScope::Global => ReceptorScope::Global,
        EventScope::Spatial { node, radius } => {
            let scene = api.wired_scene.lock().await;
            let abs = scene
                .prims
                .get(node)
                .map(|res| AbsoluteNodeId {
                    doc: res.doc_id,
                    node: res.id,
                })
                .ok_or_else(|| anyhow::anyhow!("listen: node not found"))?;
            drop(scene);
            ReceptorScope::Spatial { node: abs, radius }
        }
    };

    let id = NEXT_RECEPTOR_ID.fetch_add(1, Ordering::Relaxed);
    api.wired_event
        .lock()
        .await
        .receptors
        .items
        .insert(id, EventReceptorRes { rx });

    EVENT_RECEPTOR_REGISTRY.write().insert(
        id,
        ReceptorEntry {
            channels,
            doc_id: api.doc_id,
            scope,
            source_documents: filter.documents,
            tx,
        },
    );

    Ok(id)
}

pub async fn receptor_poll(api: &Api, rep: u32) -> anyhow::Result<Option<InboundEvent>> {
    api.wired_event
        .lock()
        .await
        .receptors
        .get(rep)
        .map(|res| {
            res.rx
                .try_recv()
                .map_or_else(|_| Ok(None), |ev| Ok(Some(ev)))
        })
        .ok_or_else(|| anyhow::anyhow!("receptor not found: {rep}"))?
}

pub async fn receptor_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    EVENT_RECEPTOR_REGISTRY.write().remove(&rep);
    api.wired_event.lock().await.receptors.remove(rep);
    Ok(())
}
