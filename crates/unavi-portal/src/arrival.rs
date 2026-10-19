//! Where a one-way portal lands a body in its destination space.

use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_hsd::{
    attributes::spawn::SpawnPoint,
    document::HsdDocId,
    hierarchy::descends_from,
    prim::{
        Prim,
        PrimOf,
    },
};

use crate::{
    destination::Destination,
    portal::Portal,
};

#[cfg(test)] mod tests;

/// The pose a one-way portal's crossing lands at, and its still is taken
/// from. Only portals glued to a document root carry one.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Arrival {
    pub translation: Vec3,
    /// Yaw only.
    pub facing:      Quat,
}

impl Arrival {
    /// Carries a direction or rotation through `portal` onto this arrival,
    /// as a virtual seam at the arrival pose facing its forward. `from_front`
    /// is whether the crossing starts on the side the portal's normal points
    /// to. Only the portal's yaw is undone, so a tilted portal never tilts the
    /// body that crosses it.
    #[must_use]
    pub fn rotation_from(&self, portal: &GlobalTransform, from_front: bool) -> Quat {
        let turn = if from_front {
            Quat::IDENTITY
        } else {
            Quat::from_rotation_y(PI)
        };
        self.facing * turn * yaw_of(portal.rotation()).inverse()
    }
}

fn yaw_of(rotation: Quat) -> Quat {
    Quat::from_rotation_y(rotation.to_euler(EulerRot::YXZ).0)
}

/// Sets each one-way portal's [`Arrival`] to the lowest-keyed spawn point
/// under its destination root, so every peer lands at the same one. A
/// destination with no spawn point arrives at the root itself.
pub fn update_arrivals(
    portals: Query<(Entity, Option<&Destination>, Option<&Arrival>), With<Portal>>,
    roots: Query<&GlobalTransform>,
    spawn_points: Query<(&GlobalTransform, &Prim, &PrimOf, &ChildOf), With<SpawnPoint>>,
    doc_ids: Query<&HsdDocId>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    for (portal, destination, current) in &portals {
        let root = destination.map(|d| d.0).filter(|d| !portals.contains(*d));
        let next = root.and_then(|root| {
            let lowest = spawn_points
                .iter()
                .filter(|(.., child_of)| descends_from(child_of.parent(), root, &parents))
                .filter_map(|(transform, prim, prim_of, _)| {
                    let doc = doc_ids.get(prim_of.0).ok()?;
                    Some(((doc.0, prim.0), transform))
                })
                .min_by_key(|(key, _)| *key)
                .map(|(_, transform)| transform);
            let pose = lowest.or_else(|| roots.get(root).ok())?;
            Some(Arrival {
                translation: pose.translation(),
                facing:      yaw_of(pose.rotation()),
            })
        });

        match (next, current) {
            (Some(next), Some(current)) if next == *current => {}
            (Some(next), _) => {
                commands.entity(portal).insert(next);
            }
            (None, Some(_)) => {
                commands.entity(portal).remove::<Arrival>();
            }
            (None, None) => {}
        }
    }
}
