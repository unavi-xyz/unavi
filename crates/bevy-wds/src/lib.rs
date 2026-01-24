use bevy::prelude::*;
use wds::actor::Actor;

pub struct WdsPlugin;

impl Plugin for WdsPlugin {
    fn build(&self, _app: &mut App) {}
}

/// Singleton actor of the local WDS.
#[derive(Component)]
pub struct LocalActor(pub Actor);

/// Actor for a remote WDS.
/// Used for syncing, data fetching, and discovery.
#[derive(Component)]
pub struct RemoteActor(pub Actor);
