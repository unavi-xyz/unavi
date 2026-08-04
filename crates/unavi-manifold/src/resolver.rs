use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use bevy_hsd::{
    Hsd,
    HsdDocId,
    HsdPrimIndex,
};
use hsd::id::DocId;

use crate::{
    GluedTo,
    SeamTargetDoc,
    SeamTargetReceptor,
};

pub fn resolve_target_doc(
    seams: Query<(Entity, &SeamTargetDoc, Option<&GluedTo>), Without<SeamTargetReceptor>>,
    spaces: Query<(Entity, &HsdDocId), With<Hsd>>,
    mut commands: Commands,
) {
    let index: HashMap<DocId, Entity> = spaces.iter().map(|(e, rid)| (rid.0, e)).collect();

    for (seam, target, current) in &seams {
        let resolved = index.get(&target.0).copied();
        reconcile(seam, resolved, current, &mut commands);
    }
}

pub fn resolve_target_receptor(
    seams: Query<(Entity, &SeamTargetReceptor, Option<&GluedTo>)>,
    docs: Query<(&HsdDocId, &HsdPrimIndex), With<Hsd>>,
    mut commands: Commands,
) {
    let index: HashMap<DocId, &HsdPrimIndex> = docs.iter().map(|(rid, idx)| (rid.0, idx)).collect();

    for (seam, target, current) in &seams {
        let resolved = index
            .get(&target.document)
            .and_then(|idx| idx.0.get(&target.prim).copied());
        reconcile(seam, resolved, current, &mut commands);
    }
}

fn reconcile(
    seam: Entity,
    resolved: Option<Entity>,
    current: Option<&GluedTo>,
    commands: &mut Commands,
) {
    match (resolved, current) {
        (Some(e), Some(cur)) if cur.0 == e => {}
        (Some(e), _) => {
            commands.entity(seam).insert(GluedTo(e));
        }
        (None, Some(_)) => {
            commands.entity(seam).remove::<GluedTo>();
        }
        (None, None) => {}
    }
}
