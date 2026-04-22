use bevy::{platform::collections::HashMap, prelude::*};
use blake3::Hash;
use iroh::EndpointId;

mod gossip;
mod presence;

pub struct SpacePlugin;

impl Plugin for SpacePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(gossip::spawn_gossip)
            .add_observer(gossip::on_space_add)
            .add_observer(gossip::on_space_remove)
            .add_systems(FixedUpdate, presence::manage_peers);
    }
}

#[derive(Component)]
struct Space(pub Hash);

#[derive(Component)]
#[require(ActiveSpaces)]
pub struct Peer(pub EndpointId);

#[derive(Component, Default)]
pub struct ActiveSpaces(pub HashMap<Hash, f32>);
