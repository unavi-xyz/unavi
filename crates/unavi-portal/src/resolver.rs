use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdPrimIndex,
    HsdRecordId,
};

use crate::{
    PortalDestination,
    PortalTargetReceptor,
    PortalTargetSpace,
};

pub fn resolve_target_space(
    portals: Query<
        (Entity, &PortalTargetSpace, Option<&PortalDestination>),
        Without<PortalTargetReceptor>,
    >,
    spaces: Query<(Entity, &HsdRecordId), With<Hsd>>,
    mut commands: Commands,
) {
    for (portal, target, current) in &portals {
        let resolved = spaces
            .iter()
            .find_map(|(e, rid)| (rid.0 == target.0).then_some(e));

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
}



pub fn resolve_target_receptor(
    portals: Query<(Entity, &PortalTargetReceptor, Option<&PortalDestination>)>,
    docs: Query<(&HsdRecordId, &HsdPrimIndex), With<Hsd>>,
    mut commands: Commands,
) {
    for (portal, target, current) in &portals {
        let prim_ent = docs.iter().find_map(|(rid, idx)| {
            (rid.0 == target.document)
                .then(|| idx.0.get(&target.prim).copied())
                .flatten()
        });

        match (prim_ent, current) {
            (Some(e), Some(cur)) if cur.0 == e => {}
            (Some(e), _) => {
                commands.entity(portal).insert(PortalDestination(e));
            }
            (None, _) => {}
        }
    }
}
