use bevy::prelude::*;

use crate::{
    Hsd,
    HsdChildren,
    attributes::{
        Pending,
        collider::HsdCollider,
        image::HsdImage,
        reference::HsdRef,
    },
    load::RefRefused,
};

#[derive(Component)]
pub struct HsdLoaded;

/// Set after the first event batch has been drained, so readiness is only
/// evaluated once every prim and its pending-asset markers exist.
#[derive(Component)]
pub(crate) struct HsdSnapshotDrained;

pub(crate) fn evaluate_hsd_loaded(
    docs: Query<(Entity, &HsdChildren), (With<Hsd>, With<HsdSnapshotDrained>, Without<HsdLoaded>)>,
    prims: Query<(
        Option<&Pending<Mesh3d>>,
        Option<&Pending<HsdImage>>,
        Option<&Pending<HsdCollider>>,
        Option<&HsdRef>,
        Option<&RefRefused>,
        Option<&Children>,
    )>,
    loaded_docs: Query<(), (With<Hsd>, With<HsdLoaded>)>,
    mut commands: Commands,
) {
    for (doc, children) in &docs {
        let ready = children.iter().all(|prim| {
            let Ok((mesh, image, collider, reference, refused, prim_children)) = prims.get(prim)
            else {
                return true;
            };
            // A refused reference never resolves, so it must not block the
            // rest of the document from ever loading.
            let reference_ready = reference.is_none()
                || refused.is_some()
                || prim_children.is_some_and(|c| c.iter().any(|e| loaded_docs.contains(e)));
            mesh.is_none() && image.is_none() && collider.is_none() && reference_ready
        });

        if ready {
            commands.entity(doc).insert(HsdLoaded);
        }
    }
}
