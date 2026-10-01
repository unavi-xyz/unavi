use bevy::prelude::*;

use crate::{
    document::{
        Hsd,
        Unplaced,
    },
    prim::Prims,
    reference::{
        Reference,
        ReferenceInstances,
        ReferenceStatus,
    },
};

#[derive(Component)]
pub struct HsdLoaded;

/// The document's first event batch has been drained.
#[derive(Component)]
pub(crate) struct SnapshotDrained;

pub(crate) fn evaluate_hsd_loaded(
    docs: Query<
        (Entity, &Prims),
        (
            With<Hsd>,
            Without<Unplaced>,
            With<SnapshotDrained>,
            Without<HsdLoaded>,
        ),
    >,
    prims: Query<(
        Option<&Reference>,
        Option<&ReferenceStatus>,
        Option<&ReferenceInstances>,
    )>,
    loaded_docs: Query<(), (With<Hsd>, With<HsdLoaded>)>,
    mut commands: Commands,
) {
    for (doc, children) in &docs {
        let ready = children.iter().all(|prim| {
            let Ok((reference, status, instances)) = prims.get(prim) else {
                return true;
            };
            reference.is_none()
                || matches!(status, Some(ReferenceStatus::Refused(_)))
                || instances.is_some_and(|i| i.iter().any(|e| loaded_docs.contains(e)))
        });

        if ready {
            commands.entity(doc).insert(HsdLoaded);
        }
    }
}
