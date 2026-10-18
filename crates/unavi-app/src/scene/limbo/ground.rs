//! Looking for something to stand on under the point the agent would spawn at.

use avian3d::prelude::*;
use bevy::prelude::*;
use unavi_agent::{
    LocalAgent,
    LocalAgentEntities,
};

use crate::scene::{
    limbo::{
        Limbo,
        arrival::LimboArrival,
    },
    respawn::SpawnPlaces,
};

/// How far below the spawn point ground may be and still count.
const GROUND_PROBE_DISTANCE: f32 = 64.0;
/// How far above the spawn point the ray starts, so a floor flush with it is
/// not behind the origin.
const GROUND_PROBE_LIFT: f32 = 0.5;

/// Casts straight down from the point the arrival space would put the agent
/// at, and records that ground was found.
///
/// The limbo floor and the agent's own colliders are skipped, since neither
/// is part of the space.
pub fn probe_ground(
    places: SpawnPlaces,
    spatial: SpatialQuery,
    agents: Query<&LocalAgentEntities, With<LocalAgent>>,
    floor: Query<Entity, With<Limbo>>,
    children: Query<&Children>,
    mut arrival: ResMut<LimboArrival>,
) {
    let Some(space) = arrival.ready_space() else {
        return;
    };

    let mut excluded: Vec<Entity> = floor.iter().collect();
    if let Ok(ents) = agents.single() {
        excluded.push(ents.body);
        excluded.extend(children.iter_descendants(ents.body));
    }
    let filter = SpatialQueryFilter::from_excluded_entities(excluded);

    let Some(space_origin) = places.origin(space) else {
        return;
    };
    let offset = arrival.spawn_offset().unwrap_or_else(|| {
        let offset = places.pick(space) - space_origin;
        arrival.set_spawn_offset(offset);
        offset
    });

    let origin = space_origin + offset + Vec3::Y * GROUND_PROBE_LIFT;
    let max_distance = GROUND_PROBE_DISTANCE + GROUND_PROBE_LIFT;
    if spatial
        .cast_ray(origin, Dir3::NEG_Y, max_distance, true, &filter)
        .is_some()
    {
        arrival.mark_grounded();
    }
}
