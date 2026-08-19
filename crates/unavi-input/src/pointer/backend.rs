use avian3d::prelude::*;
use bevy::{
    picking::backend::{
        HitData,
        PointerHits,
    },
    prelude::*,
};

use crate::{
    capture::Captured,
    pointer::{
        PointerAnchor,
        PointerReach,
        ray_of,
    },
};

/// Colliders a pointer's ray passes through rather than lands on. The agent's
/// own body is the reason this exists — a ray cast from inside the head would
/// otherwise hit it every frame.
#[derive(Resource, Default)]
pub struct PointerFilter(pub SpatialQueryFilter);

/// Casts each pointer's ray at the physics world.
///
/// Avian ships a picking backend of its own, and it casts without limit.
/// Reach is what stops a press landing on something across the map, so the
/// cast is ours.
pub fn update_hits(
    pointers: Query<(&PointerAnchor, &PointerReach, &GlobalTransform)>,
    cameras: Query<(Entity, &Camera), With<Camera3d>>,
    filter: Res<PointerFilter>,
    captured: Res<Captured>,
    query: SpatialQuery,
    mut hits: MessageWriter<PointerHits>,
) {
    // Reported as misses rather than skipped, so a prim hovered when something
    // else took the input hears that it was left.
    if captured.0 {
        for (anchor, ..) in pointers {
            hits.write(PointerHits::new(anchor.0.id(), Vec::new(), 0.0));
        }
        return;
    }

    // A hit names the camera it was seen through, and picking sorts by that
    // camera's order. Which of a headset's two eyes reported it is arbitrary,
    // so the ray is cast once against the one drawn last.
    let Some(camera) = cameras
        .iter()
        .filter(|(_, camera)| camera.is_active)
        .max_by_key(|(_, camera)| camera.order)
        .map(|(entity, _)| entity)
    else {
        return;
    };

    for (anchor, reach, transform) in pointers {
        let ray = ray_of(transform);

        let Some(hit) = query.cast_ray(ray.origin, ray.direction, reach.0, true, &filter.0) else {
            hits.write(PointerHits::new(anchor.0.id(), Vec::new(), 0.0));
            continue;
        };

        let data = HitData::new(
            camera,
            hit.distance,
            Some(ray.get_point(hit.distance)),
            Some(hit.normal),
        );
        hits.write(PointerHits::new(
            anchor.0.id(),
            vec![(hit.entity, data)],
            0.0,
        ));
    }
}
