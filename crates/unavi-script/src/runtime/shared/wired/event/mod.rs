use std::sync::{
    Arc,
    atomic::{
        AtomicBool,
        Ordering,
    },
};

use async_channel::Receiver;
use hsd::bounds::MAX_EVENT_PAYLOAD_BYTES;
use unavi_policy::quota::{
    Flow,
    QuotaError,
    Stock,
    StockGuard,
};
use web_time::{
    SystemTime,
    UNIX_EPOCH,
};

use crate::runtime::shared::{
    Api,
    registry::{
        event::{
            InboundEvent,
            ReceptorScope,
            SenderScope,
            claim,
        },
        transform::AbsoluteNodeId,
    },
    slot_map::SlotMap,
};

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
    // Speaking is a write, and an unplaced document has no co-presence to
    // appeal to. Without this a document that cannot be attributed reaches
    // every receptor its owner-check happens to pass.
    api.view.placed(api.doc_id)?;
    crate::quota::acquire(&api.quota, Flow::Emit, 1.0).await?;

    let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

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
                let pos = api.transforms.node(&abs).map(|s| s.world.translation());
                Some((abs, pos, *radius))
            }
            EventScope::Global => None,
        };

    #[cfg(feature = "debug")]
    if let Some((_, Some(pos), radius)) = emitter_spatial.as_ref() {
        api.event_bus.record_emit(&channel, *pos, *radius);
    }

    let payload = Arc::new(payload);
    let sender_doc = api.doc_id.0.to_vec();
    let audience_claim = filter
        .documents
        .as_ref()
        .map(|_| Arc::new(AtomicBool::new(false)));

    api.event_bus.deliver(
        &channel,
        &payload,
        &sender_doc,
        time,
        filter.documents.as_deref(),
        audience_claim.as_ref(),
        |entry_doc, scope| {
            if api.view.write(api.doc_id, entry_doc).is_err() {
                return None;
            }
            resolve_sender_scope(api, emitter_spatial.as_ref(), scope)
        },
    );

    Ok(())
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
            let emitter_is_system = api.view.tier_of(api.doc_id).crosses_space_boundaries();
            if !emitter_is_system && !api.view.same_space(emitter_abs.doc, receptor_node.doc) {
                return None;
            }
            let r_pos = api
                .transforms
                .node(receptor_node)
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

    let (id, rx) = api
        .event_bus
        .listen(api.doc_id, channels, scope, filter.documents);
    api.wired_event.lock().await.receptors.insert_at(
        id,
        EventReceptorRes { rx, _guard: guard },
        &api.quota,
    )?;

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
    api.event_bus.drop_receptor(rep, api.doc_id);
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
    Ok(claim(&claimed))
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
