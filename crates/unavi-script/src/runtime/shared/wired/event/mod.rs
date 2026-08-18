use std::{
    sync::{
        Arc,
        atomic::{
            AtomicBool,
            AtomicU32,
            Ordering,
        },
    },
    time::UNIX_EPOCH,
};

use async_channel::Receiver;
use hsd::id::DocId;
use unavi_policy::firewall::{
    Channel,
    registry::validate_firewall,
};
use unavi_quota::{
    Flow,
    QuotaError,
    Stock,
    StockGuard,
    limits::MAX_EVENT_PAYLOAD_BYTES,
};

use crate::runtime::shared::{
    Api,
    registry::{
        event::{
            EVENT_RECEPTOR_REGISTRY,
            InboundEvent,
            ReceptorEntry,
            ReceptorScope,
            SenderScope,
        },
        transform::{
            AbsoluteNodeId,
            NODE_TRANSFORM_REGISTRY,
        },
    },
    slot_map::SlotMap,
};

static NEXT_RECEPTOR_ID: AtomicU32 = AtomicU32::new(0);

pub const HOST_SENDER_DOC: [u8; 32] = [0u8; 32];

#[must_use]
pub fn doc_has_receptor(doc: DocId, channel: &str) -> bool {
    EVENT_RECEPTOR_REGISTRY
        .read()
        .values()
        .any(|e| e.doc_id == doc && e.channels.iter().any(|c| c == channel))
}

#[derive(Default)]
pub struct EventFilter {
    pub documents: Option<Vec<Vec<u8>>>,
    pub scope:     EventScope,
}

#[derive(Default)]
pub enum EventScope {
    #[default]
    Global,
    Spatial {
        node:   u32,
        radius: f32,
    },
}

pub struct EventReceptorRes {
    rx:     Receiver<InboundEvent>,
    _guard: StockGuard,
}

pub struct EventRes {
    pub inner: InboundEvent,
}

#[derive(Default)]
pub struct WiredEventApi {
    pub receptors: SlotMap<EventReceptorRes>,
    pub events:    SlotMap<EventRes>,
}

const RECEPTOR_CAPACITY: usize = 16;

pub async fn emit(
    api: &Api,
    channel: String,
    payload: Vec<u8>,
    filter: EventFilter,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        payload.len() <= MAX_EVENT_PAYLOAD_BYTES,
        "event payload too large"
    );
    api.quota.spend(Flow::Emit, 1.0)?;

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
                        doc:  res.doc_id,
                        node: res.id,
                    })
                    .ok_or_else(|| anyhow::anyhow!("emit: node not found"))?;
                drop(scene);
                let pos = NODE_TRANSFORM_REGISTRY
                    .read()
                    .get(&abs)
                    .map(|s| s.world.translation());
                Some((abs, pos, *radius))
            }
            EventScope::Global => None,
        };

    #[cfg(feature = "debug")]
    if let Some((_, Some(pos), radius)) = emitter_spatial.as_ref() {
        crate::debug::record_emit(&channel, *pos, *radius);
    }

    let payload = Arc::new(payload);
    let sender_doc = api.doc_id.0.to_vec();
    let claimed = Arc::new(AtomicBool::new(false));

    let registry = EVENT_RECEPTOR_REGISTRY.read();
    for entry in registry.values() {
        if !entry.channels.iter().any(|c| c == &channel) {
            continue;
        }

        if let Some(docs) = &filter.documents
            && !docs
                .iter()
                .any(|d| d.as_slice() == entry.doc_id.0.as_slice())
        {
            continue;
        }

        if let Some(docs) = &entry.source_documents
            && !docs.iter().any(|d| d.as_slice() == api.doc_id.0.as_slice())
        {
            continue;
        }

        if validate_firewall(&api.doc_id, &entry.doc_id, Channel::EventWrite).is_err() {
            continue;
        }

        let Some(sender_scope) = resolve_sender_scope(api, emitter_spatial.as_ref(), &entry.scope)
        else {
            continue;
        };

        let _ = entry.tx.try_send(InboundEvent {
            channel: channel.clone(),
            payload: Arc::clone(&payload),
            sender_document: sender_doc.clone(),
            sender_scope,
            time,
            claimed: Arc::clone(&claimed),
        });
    }
    drop(registry);

    Ok(())
}

pub fn emit_from_host(target_doc: DocId, channel: &str, payload: Vec<u8>) {
    let payload = Arc::new(payload);
    let claimed = Arc::new(AtomicBool::new(false));
    let time = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();

    let registry = EVENT_RECEPTOR_REGISTRY.read();
    for entry in registry.values() {
        if entry.doc_id != target_doc {
            continue;
        }
        if !entry.channels.iter().any(|c| c == channel) {
            continue;
        }
        let _ = entry.tx.try_send(InboundEvent {
            channel: channel.into(),
            payload: Arc::clone(&payload),
            sender_document: HOST_SENDER_DOC.to_vec(),
            sender_scope: SenderScope::Global,
            time,
            claimed: Arc::clone(&claimed),
        });
    }
}

fn resolve_sender_scope(
    api: &Api,
    emitter_spatial: Option<&(AbsoluteNodeId, Option<bevy::math::Vec3>, f32)>,
    receptor_scope: &ReceptorScope,
) -> Option<SenderScope> {
    match (emitter_spatial, receptor_scope) {
        (None, ReceptorScope::Global) => Some(SenderScope::Global),
        (None, ReceptorScope::Spatial { .. }) => None,
        (Some((abs, ..)), ReceptorScope::Global) => Some(SenderScope::Spatial {
            distance: 0.0,
            node:     *abs,
        }),
        (
            Some((emitter_abs, emitter_pos, emitter_radius)),
            ReceptorScope::Spatial {
                node: receptor_node,
                radius: receptor_radius,
            },
        ) => {
            let e_pos = (*emitter_pos)?;
            let emitter_is_system = api.policy.trust.crosses_space_boundaries();
            if !emitter_is_system
                && !unavi_space::membership::same_space(emitter_abs.doc, receptor_node.doc)
            {
                return None;
            }
            let r_pos = NODE_TRANSFORM_REGISTRY
                .read()
                .get(receptor_node)
                .map(|s| s.world.translation())?;
            let dist = (e_pos - r_pos).length();
            if dist > *emitter_radius + *receptor_radius {
                return None;
            }
            Some(SenderScope::Spatial {
                distance: dist,
                node:     *emitter_abs,
            })
        }
    }
}

pub async fn listen(api: &Api, channels: Vec<String>, filter: EventFilter) -> anyhow::Result<u32> {
    let guard = api.quota.charge(Stock::Receptors, 1)?;
    let (tx, rx) = async_channel::bounded(RECEPTOR_CAPACITY);

    let scope = match filter.scope {
        EventScope::Global => ReceptorScope::Global,
        EventScope::Spatial { node, radius } => {
            let scene = api.wired_scene.lock().await;
            let abs = scene
                .prims
                .get(node)
                .map(|res| AbsoluteNodeId {
                    doc:  res.doc_id,
                    node: res.id,
                })
                .ok_or_else(|| anyhow::anyhow!("listen: node not found"))?;
            drop(scene);
            ReceptorScope::Spatial { node: abs, radius }
        }
    };

    let id = NEXT_RECEPTOR_ID.fetch_add(1, Ordering::Relaxed);
    api.wired_event.lock().await.receptors.insert_at(
        id,
        EventReceptorRes { rx, _guard: guard },
        &api.quota,
    )?;

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
    let rx = {
        let slots = api.wired_event.lock().await;
        slots
            .receptors
            .get(rep)
            .map(|res| res.rx.clone())
            .ok_or_else(|| anyhow::anyhow!("receptor not found: {rep}"))?
    };
    loop {
        match rx.try_recv() {
            Ok(ev) => {
                if ev.claimed.load(Ordering::Acquire) {
                    continue;
                }
                return Ok(Some(ev));
            }
            Err(_) => return Ok(None),
        }
    }
}

pub async fn receptor_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    EVENT_RECEPTOR_REGISTRY.write().remove(&rep);
    api.wired_event.lock().await.receptors.remove(rep);
    Ok(())
}

pub async fn insert_event(api: &Api, event: InboundEvent) -> Result<u32, QuotaError> {
    api.wired_event
        .lock()
        .await
        .events
        .insert(EventRes { inner: event }, &api.quota)
}

pub async fn event_consume(api: &Api, rep: u32) -> anyhow::Result<bool> {
    let claimed = {
        let slots = api.wired_event.lock().await;
        slots
            .events
            .get(rep)
            .map(|res| Arc::clone(&res.inner.claimed))
            .ok_or_else(|| anyhow::anyhow!("event not found: {rep}"))?
    };
    Ok(claimed
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok())
}

pub async fn event_clone_inner(api: &Api, rep: u32) -> anyhow::Result<InboundEvent> {
    let slots = api.wired_event.lock().await;
    let inner = slots
        .events
        .get(rep)
        .map(|res| res.inner.clone())
        .ok_or_else(|| anyhow::anyhow!("event not found: {rep}"))?;
    drop(slots);
    Ok(inner)
}

pub async fn event_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_event.lock().await.events.remove(rep);
    Ok(())
}
