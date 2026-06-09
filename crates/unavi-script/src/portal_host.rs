use std::collections::{
    HashMap,
    HashSet,
};

use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdChild,
    HsdRecordId,
    Prim,
    attributes::portal::PortalConfig,
};
use blake3::Hash;
use unavi_manifold::Seam;
use unavi_portal_protocol::{
    BACKLINK_CHANNEL,
    BacklinkPayload,
    INCOMING_CHANNEL,
    IncomingPayload,
};
use unavi_space::membership::doc_space;

use crate::{
    engine::InitializedScript,
    runtime::shared::wired::event::{
        doc_has_receptor,
        emit_from_host,
    },
};

/// Window during which `open` awaits a matching receptor in the target space
/// before the handshake lapses and the portal stays merely space-aimed.
const WATCH_TTL_SECS: f32 = 60.0;

/// Cap on buffered notifications per document, so a target whose script never
/// listens on the portal channels cannot accumulate payloads without bound.
const PENDING_CAP: usize = 64;

/// A live handshake armed by `open`: transient by design, so a lapsed attempt
/// despawns without leaving any trace in the synced document state.
#[derive(Component)]
pub struct PortalWatch {
    source_space: Hash,
    source_doc:   Hash,
    source_prim:  String,
    target_space: Hash,
    timer:        Timer,
    emitted:      HashSet<Hash>,
}

impl PortalWatch {
    #[must_use]
    pub fn new(
        source_space: Hash,
        source_doc: Hash,
        source_prim: String,
        target_space: Hash,
    ) -> Self {
        Self {
            source_space,
            source_doc,
            source_prim,
            target_space,
            timer: Timer::from_seconds(WATCH_TTL_SECS, TimerMode::Once),
            emitted: HashSet::new(),
        }
    }
}

#[derive(Component, Default)]
pub struct PendingIncoming(Vec<IncomingPayload>);

#[derive(Component, Default)]
pub struct PendingBacklink(Vec<BacklinkPayload>);

fn stage_push<T: PartialEq>(stage: &mut HashMap<Entity, Vec<T>>, entity: Entity, payload: T) {
    let buf = stage.entry(entity).or_default();
    if !buf.contains(&payload) {
        buf.push(payload);
    }
}

fn merge_pending<T: PartialEq>(existing: &mut Vec<T>, payloads: Vec<T>) {
    for payload in payloads {
        if existing.contains(&payload) {
            continue;
        }
        if existing.len() >= PENDING_CAP {
            existing.remove(0);
        }
        existing.push(payload);
    }
}

fn flush_incoming(
    commands: &mut Commands,
    pending: &mut Query<&mut PendingIncoming>,
    stage: &mut HashMap<Entity, Vec<IncomingPayload>>,
) {
    for (entity, payloads) in stage.drain() {
        if let Ok(mut existing) = pending.get_mut(entity) {
            merge_pending(&mut existing.0, payloads);
        } else {
            commands.entity(entity).insert(PendingIncoming(payloads));
        }
    }
}

fn flush_backlink(
    commands: &mut Commands,
    pending: &mut Query<&mut PendingBacklink>,
    stage: &mut HashMap<Entity, Vec<BacklinkPayload>>,
) {
    for (entity, payloads) in stage.drain() {
        if let Ok(mut existing) = pending.get_mut(entity) {
            merge_pending(&mut existing.0, payloads);
        } else {
            commands.entity(entity).insert(PendingBacklink(payloads));
        }
    }
}

/// Drives every armed watch each frame: expiring the stale, confirming the
/// answered, and announcing the rest into their target space.
pub fn service_portal_watches(
    time: Res<Time>,
    mut watches: Query<(Entity, &mut PortalWatch)>,
    docs: Query<(Entity, &HsdRecordId), With<Hsd>>,
    receptors: Query<(&PortalConfig, &HsdChild, &Prim), With<Seam>>,
    mut incoming: Query<&mut PendingIncoming>,
    mut backlink: Query<&mut PendingBacklink>,
    mut commands: Commands,
    mut stage_incoming: Local<HashMap<Entity, Vec<IncomingPayload>>>,
    mut stage_backlink: Local<HashMap<Entity, Vec<BacklinkPayload>>>,
) {
    for (entity, mut watch) in &mut watches {
        if watch.timer.tick(time.delta()).is_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let found_receptor = receptors.iter().find_map(|(cfg, hsd_child, prim)| {
            let receptor = cfg.0.destination.as_ref()?.receptor.as_ref()?;
            if Hash::from(receptor.document.0) != watch.source_doc
                || receptor.prim != watch.source_prim
            {
                return None;
            }
            let (_, recp_record) = docs.get(hsd_child.0).ok()?;
            (doc_space(recp_record.0) == Some(watch.target_space))
                .then(|| (recp_record.0, prim.0.to_string()))
        });
        if let Some((receptor_doc, receptor_prim)) = found_receptor {
            if let Some((source_entity, _)) = docs.iter().find(|(_, r)| r.0 == watch.source_doc) {
                stage_push(
                    &mut stage_backlink,
                    source_entity,
                    BacklinkPayload {
                        source_prim: watch.source_prim.clone(),
                        receptor_doc: *receptor_doc.as_bytes(),
                        receptor_prim,
                    },
                );
            }
            commands.entity(entity).despawn();
            continue;
        }

        let payload = IncomingPayload {
            source_space: *watch.source_space.as_bytes(),
            source_doc:   *watch.source_doc.as_bytes(),
            source_prim:  watch.source_prim.clone(),
        };
        for (doc_entity, record) in &docs {
            if record.0 == watch.source_doc {
                continue;
            }
            if doc_space(record.0) != Some(watch.target_space) {
                continue;
            }
            if !watch.emitted.insert(record.0) {
                continue;
            }
            stage_push(&mut stage_incoming, doc_entity, payload.clone());
        }
    }

    if !stage_incoming.is_empty() {
        flush_incoming(&mut commands, &mut incoming, &mut stage_incoming);
        stage_incoming.shrink_to(64);
    }
    if !stage_backlink.is_empty() {
        flush_backlink(&mut commands, &mut backlink, &mut stage_backlink);
        stage_backlink.shrink_to(64);
    }
}

pub fn drain_pending(
    mut incoming: Query<(Entity, &mut PendingIncoming, &HsdRecordId)>,
    mut backlink: Query<(Entity, &mut PendingBacklink, &HsdRecordId)>,
    ready: Query<&HsdChild, With<InitializedScript>>,
    mut commands: Commands,
) {
    let mut ready_docs = HashSet::<Entity>::new();
    for c in &ready {
        ready_docs.insert(c.0);
    }

    for (entity, mut pending, record) in &mut incoming {
        if !ready_docs.contains(&entity) || !doc_has_receptor(record.0, INCOMING_CHANNEL) {
            continue;
        }
        for payload in pending.0.drain(..) {
            match postcard::to_allocvec(&payload) {
                Ok(bytes) => emit_from_host(record.0, INCOMING_CHANNEL, bytes),
                Err(err) => warn!(?err, "encode incoming payload"),
            }
        }
        commands.entity(entity).remove::<PendingIncoming>();
    }

    for (entity, mut pending, record) in &mut backlink {
        if !ready_docs.contains(&entity) || !doc_has_receptor(record.0, BACKLINK_CHANNEL) {
            continue;
        }
        for payload in pending.0.drain(..) {
            match postcard::to_allocvec(&payload) {
                Ok(bytes) => emit_from_host(record.0, BACKLINK_CHANNEL, bytes),
                Err(err) => warn!(?err, "encode backlink payload"),
            }
        }
        commands.entity(entity).remove::<PendingBacklink>();
    }
}
