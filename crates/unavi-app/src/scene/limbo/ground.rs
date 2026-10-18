//! Looking for something to stand on under the point the agent would spawn at.

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_hsd::hierarchy::descends_from;

use crate::scene::{
    limbo::arrival::LimboArrival,
    respawn::SpawnPlaces,
};

/// How far below the spawn point ground may be and still count.
const GROUND_PROBE_DISTANCE: f32 = 64.0;
/// How far above the spawn point the ray starts, so a floor flush with it is
/// not behind the origin.
const GROUND_PROBE_LIFT: f32 = 0.5;

/// Casts straight down from a freshly picked spawn point each tick until one
/// stands on ground, and records that point.
///
/// Only solid colliders inside the arrival space count, so neither a trigger
/// volume, the limbo floor, the agent, nor a neighbouring space reads as
/// ground.
pub fn probe_ground(
    places: SpawnPlaces,
    spatial: SpatialQuery,
    sensors: Query<(), With<Sensor>>,
    parents: Query<&ChildOf>,
    mut arrival: ResMut<LimboArrival>,
) {
    let Some(space) = arrival.probing_space() else {
        return;
    };
    let Some(space_origin) = places.origin(space) else {
        return;
    };

    let spawn = places.pick(space);
    let origin = spawn + Vec3::Y * GROUND_PROBE_LIFT;
    let max_distance = GROUND_PROBE_DISTANCE + GROUND_PROBE_LIFT;
    let in_space = |entity| !sensors.contains(entity) && descends_from(entity, space, &parents);
    if spatial
        .cast_ray_predicate(
            origin,
            Dir3::NEG_Y,
            max_distance,
            true,
            &SpatialQueryFilter::default(),
            &in_space,
        )
        .is_some()
    {
        arrival.mark_grounded(spawn - space_origin);
    }
}
