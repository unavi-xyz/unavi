use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use bevy_hsd::{
    Hsd,
    HsdPrimIndex,
    HsdRecordId,
};
use blake3::Hash;

use crate::{
    PortalDestination,
    PortalTargetDoc,
    PortalTargetReceptor,
};

pub fn resolve_target_doc(
    portals: Query<
        (Entity, &PortalTargetDoc, Option<&PortalDestination>),
        Without<PortalTargetReceptor>,
    >,
    spaces: Query<(Entity, &HsdRecordId), With<Hsd>>,
    mut commands: Commands,
) {
    let index: HashMap<Hash, Entity> = spaces.iter().map(|(e, rid)| (rid.0, e)).collect();

    for (portal, target, current) in &portals {
        let resolved = index.get(&target.0).copied();
        reconcile(portal, resolved, current, &mut commands);
    }
}

pub fn resolve_target_receptor(
    portals: Query<(Entity, &PortalTargetReceptor, Option<&PortalDestination>)>,
    docs: Query<(&HsdRecordId, &HsdPrimIndex), With<Hsd>>,
    mut commands: Commands,
) {
    let index: HashMap<Hash, &HsdPrimIndex> = docs.iter().map(|(rid, idx)| (rid.0, idx)).collect();

    for (portal, target, current) in &portals {
        let resolved = index
            .get(&target.document)
            .and_then(|idx| idx.0.get(&target.prim).copied());
        reconcile(portal, resolved, current, &mut commands);
    }
}

fn reconcile(
    portal: Entity,
    resolved: Option<Entity>,
    current: Option<&PortalDestination>,
    commands: &mut Commands,
) {
    match (resolved, current) {
        (Some(e), Some(cur)) if cur.0 == e => {}
        (Some(e), _) => {
            commands.entity(portal).insert(PortalDestination(e));
        }
        (None, Some(_)) => {
            commands.entity(portal).remove::<PortalDestination>();
        }
        (None, None) => {}
    }
}
