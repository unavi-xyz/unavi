use avian3d::prelude::Collider;
use bevy::prelude::*;

use crate::{
    Hsd,
    HsdChild,
    attributes::{
        collider::{
            DisabledCollider,
            HsdCollider,
        },
        image::HsdImage,
        subdocument::HsdSubdocument,
    },
};

#[derive(Component)]
pub struct HsdLoaded;

/// Set after the initial snapshot diff batch has been drained, so readiness is
/// only evaluated once every prim and its pending-asset markers exist.
#[derive(Component)]
pub struct HsdSnapshotDrained;

pub fn evaluate_hsd_loaded(
    docs: Query<Entity, (With<Hsd>, With<HsdSnapshotDrained>, Without<HsdLoaded>)>,
    prims: Query<(
        &HsdChild,
        Option<&HsdCollider>,
        Option<&Collider>,
        Option<&DisabledCollider>,
        Option<&Mesh3d>,
        Option<&HsdImage>,
        Option<&HsdSubdocument>,
        Option<&Children>,
    )>,
    loaded_docs: Query<(), (With<Hsd>, With<HsdLoaded>)>,
    mut commands: Commands,
) {
    for doc in &docs {
        let ready = prims.iter().filter(|(child, ..)| child.0 == doc).all(
            |(_, hsd_collider, collider, disabled, mesh, image, subdoc, children)| {
                let collider_ready =
                    hsd_collider.is_none() || collider.is_some() || disabled.is_some();
                let mesh_ready = mesh.is_none_or(|m| m.0 != Handle::default());
                let image_ready = image.is_none_or(|i| i.0 != Handle::default());
                let subdoc_ready = subdoc.is_none()
                    || children.is_some_and(|c| c.iter().any(|e| loaded_docs.contains(e)));
                collider_ready && mesh_ready && image_ready && subdoc_ready
            },
        );

        if ready {
            commands.entity(doc).insert(HsdLoaded);
        }
    }
}
