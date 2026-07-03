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
    GluedTo,
    SeamTargetDoc,
    SeamTargetReceptor,
};

pub fn resolve_target_doc(
    seams: Query<(Entity, &SeamTargetDoc, Option<&GluedTo>), Without<SeamTargetReceptor>>,
    spaces: Query<(Entity, &HsdRecordId), With<Hsd>>,
    mut commands: Commands,
) {
    let index: HashMap<Hash, Entity> = spaces.iter().map(|(e, rid)| (rid.0, e)).collect();

    for (seam, target, current) in &seams {
        let resolved = index.get(&target.0).copied();
        reconcile(seam, resolved, current, &mut commands);
    }
}

pub fn resolve_target_receptor(
    seams: Query<(Entity, &SeamTargetReceptor, Option<&GluedTo>)>,
    docs: Query<(&HsdRecordId, &HsdPrimIndex), With<Hsd>>,
    mut commands: Commands,
    mut unresolved: Local<HashMap<Entity, bool>>,
) {
    let index: HashMap<Hash, &HsdPrimIndex> = docs.iter().map(|(rid, idx)| (rid.0, idx)).collect();

    for (seam, target, current) in &seams {
        let doc = index.get(&target.document);
        let resolved = doc.and_then(|idx| idx.0.get(&target.prim).copied());
        if resolved.is_none() {
            let doc_loaded = doc.is_some();
            if unresolved.insert(seam, doc_loaded) != Some(doc_loaded) {
                warn!(
                    ?seam,
                    document = %target.document,
                    prim = %target.prim,
                    doc_loaded,
                    "seam receptor unresolved"
                );
            }
        } else if unresolved.remove(&seam).is_some() {
            info!(?seam, "seam receptor resolved");
        }
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
