use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdChild,
    HsdRecordId,
    Prim,
    attributes::portal::PortalConfig,
};
use blake3::Hash;
use unavi_portal::{
    Portal,
    PortalDestination,
    PortalTargetReceptor,
};
use unavi_portal_protocol::{
    BACKLINK_CHANNEL,
    BacklinkPayload,
    INCOMING_CHANNEL,
    IncomingPayload,
};
use unavi_space::membership::{
    DOC_SPACE_REGISTRY,
    doc_space,
};

use crate::{
    engine::InitializedScript,
    runtime::shared::wired::event::{
        doc_has_receptor,
        emit_from_host,
    },
};

#[derive(Component)]
pub struct PendingIncoming(IncomingPayload);

#[derive(Component)]
pub struct PendingBacklink(BacklinkPayload);

pub fn enqueue_incoming_on_destination(
    trigger: On<Insert, PortalDestination>,
    portals: Query<
        (&PortalConfig, &HsdChild, &Prim),
        (With<Portal>, Without<PortalTargetReceptor>),
    >,
    docs: Query<(Entity, &HsdRecordId), With<Hsd>>,
    mut commands: Commands,
) {
    let Ok((cfg, hsd_child, prim)) = portals.get(trigger.entity) else {
        return;
    };
    let Some(dest) = cfg.0.destination.as_ref() else {
        return;
    };
    if dest.receptor.is_some() {
        return;
    }
    let Ok((_, source_record)) = docs.get(hsd_child.0) else {
        return;
    };
    let Some(source_space) = doc_space(source_record.0) else {
        return;
    };
    let target_space = Hash::from(dest.space.0);
    let payload = IncomingPayload {
        source_space: *source_space.as_bytes(),
        source_doc:   *source_record.0.as_bytes(),
        source_prim:  prim.0.to_string(),
    };
    for (target_entity, target_record) in &docs {
        if target_record.0 == source_record.0 {
            continue;
        }
        if DOC_SPACE_REGISTRY.read().get(&target_record.0).copied() != Some(target_space) {
            continue;
        }
        commands
            .entity(target_entity)
            .insert(PendingIncoming(payload.clone()));
    }
}

pub fn enqueue_incoming_on_doc_load(
    trigger: On<Insert, HsdRecordId>,
    new_docs: Query<&HsdRecordId, With<Hsd>>,
    portals: Query<
        (&PortalConfig, &HsdChild, &Prim),
        (With<Portal>, Without<PortalTargetReceptor>),
    >,
    docs: Query<&HsdRecordId, With<Hsd>>,
    mut commands: Commands,
) {
    let Ok(new_record) = new_docs.get(trigger.entity) else {
        return;
    };
    let Some(new_space) = doc_space(new_record.0) else {
        return;
    };

    for (cfg, hsd_child, prim) in &portals {
        let Some(dest) = cfg.0.destination.as_ref() else {
            continue;
        };
        if dest.receptor.is_some() {
            continue;
        }
        if Hash::from(dest.space.0) != new_space {
            continue;
        }
        let Ok(source_record) = docs.get(hsd_child.0) else {
            continue;
        };
        if new_record.0 == source_record.0 {
            continue;
        }
        let Some(source_space) = doc_space(source_record.0) else {
            continue;
        };
        commands.entity(trigger.entity).insert(PendingIncoming(IncomingPayload {
            source_space: *source_space.as_bytes(),
            source_doc:   *source_record.0.as_bytes(),
            source_prim:  prim.0.to_string(),
        }));
    }
}

pub fn enqueue_backlink_on_accept(
    trigger: On<Insert, PortalConfig>,
    portals: Query<(&PortalConfig, &HsdChild, &Prim)>,
    docs: Query<(Entity, &HsdRecordId), With<Hsd>>,
    mut commands: Commands,
) {
    let Ok((cfg, hsd_child, prim)) = portals.get(trigger.entity) else {
        return;
    };
    let Some(dest) = cfg.0.destination.as_ref() else {
        return;
    };
    let Some(receptor) = dest.receptor.as_ref() else {
        return;
    };
    let Ok((_, this_record)) = docs.get(hsd_child.0) else {
        return;
    };
    let source_doc = Hash::from(receptor.document.0);
    if source_doc == this_record.0 {
        return;
    }
    let Some((source_entity, _)) = docs.iter().find(|(_, r)| r.0 == source_doc) else {
        return;
    };
    commands.entity(source_entity).insert(PendingBacklink(BacklinkPayload {
        source_prim:   receptor.prim.clone(),
        receptor_doc:  *this_record.0.as_bytes(),
        receptor_prim: prim.0.to_string(),
    }));
}

pub fn drain_pending(
    incoming: Query<(Entity, &PendingIncoming, &HsdRecordId)>,
    backlink: Query<(Entity, &PendingBacklink, &HsdRecordId)>,
    ready: Query<&HsdChild, With<InitializedScript>>,
    mut commands: Commands,
) {
    let mut ready_docs = std::collections::HashSet::<Entity>::new();
    for c in &ready {
        ready_docs.insert(c.0);
    }

    for (entity, pending, record) in &incoming {
        if !ready_docs.contains(&entity) || !doc_has_receptor(record.0, INCOMING_CHANNEL) {
            continue;
        }
        match postcard::to_allocvec(&pending.0) {
            Ok(bytes) => emit_from_host(record.0, INCOMING_CHANNEL, bytes),
            Err(err) => warn!(?err, "encode incoming payload"),
        }
        commands.entity(entity).remove::<PendingIncoming>();
    }

    for (entity, pending, record) in &backlink {
        if !ready_docs.contains(&entity) || !doc_has_receptor(record.0, BACKLINK_CHANNEL) {
            continue;
        }
        match postcard::to_allocvec(&pending.0) {
            Ok(bytes) => emit_from_host(record.0, BACKLINK_CHANNEL, bytes),
            Err(err) => warn!(?err, "encode backlink payload"),
        }
        commands.entity(entity).remove::<PendingBacklink>();
    }
}
