use bevy::{platform::collections::HashMap, prelude::*};
use blake3::Hash;
use iroh::EndpointId;

mod gossip;
mod presence;
mod scene;

pub struct SpacePlugin;

impl Plugin for SpacePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(gossip::spawn_gossip)
            .add_observer(gossip::join_space_topic)
            .add_observer(gossip::leave_space_topic)
            .add_observer(scene::spawn_space_scene)
            .add_observer(scene::despawn_space_scene)
            .add_systems(
                FixedUpdate,
                (presence::manage_peers, scene::instantiate_pending_scenes),
            );
    }
}

#[derive(Component)]
struct Space(pub Hash);

#[derive(Component)]
#[require(ActiveSpaces)]
pub struct Peer(pub EndpointId);

#[derive(Component, Default)]
pub struct ActiveSpaces(pub HashMap<Hash, f32>);
