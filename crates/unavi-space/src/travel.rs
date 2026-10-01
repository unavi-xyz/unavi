//! Requests to move the local agent into another space.

use bevy::prelude::*;

use crate::membership::SpaceId;

/// A queued request to travel the local agent into a space.
///
/// Consumed by the client's travel driver, which unloads the current space and
/// routes arrival through the limbo load-gate so spawning honors the target's
/// spawn points.
#[derive(Resource, Default)]
pub struct PendingTravel(pub Option<SpaceId>);

pub fn request_travel(world: &mut World, target: SpaceId) {
    world.resource_mut::<PendingTravel>().0 = Some(target);
}
