use bevy::prelude::*;
use iroh_docs::NamespaceId;

/// A queued request to travel the local agent into `target`.
///
/// Consumed by the client's travel driver, which unloads the current space and
/// routes arrival through the limbo load-gate so spawning honors the target's
/// spawn points.
#[derive(Resource, Default)]
pub struct PendingTravel(pub Option<NamespaceId>);

pub fn request_travel(world: &mut World, target: NamespaceId) {
    world.resource_mut::<PendingTravel>().0 = Some(target);
}
