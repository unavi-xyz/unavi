use bevy::prelude::*;
use blake3::Hash;

/// A queued request to travel the local agent into `target`.
///
/// Consumed by the client's travel driver, which unloads the current space and
/// routes arrival through the limbo load-gate so spawning honors the target's
/// spawn points.
#[derive(Resource, Default)]
pub struct PendingTravel(pub Option<Hash>);

pub fn request_travel(world: &mut World, target: Hash) {
    world.resource_mut::<PendingTravel>().0 = Some(target);
}
