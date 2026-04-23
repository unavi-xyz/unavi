use std::time::Duration;

use bevy::{platform::collections::HashMap, prelude::*, time::common_conditions::on_timer};
use blake3::Hash;
use iroh::EndpointAddr;

mod beacon;
mod connection;
mod gossip;
mod presence;
mod scene;

pub struct SpacePlugin;

impl Plugin for SpacePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(connection::connect_to_peer)
            .add_observer(connection::disconnect_peer)
            .add_observer(connection::register_protocol)
            .add_observer(gossip::spawn_gossip)
            .add_observer(gossip::join_space_topic)
            .add_observer(gossip::leave_space_topic)
            .add_observer(scene::spawn_space_scene)
            .add_observer(scene::despawn_space_scene)
            .add_systems(
                FixedUpdate,
                (
                    beacon::publish_beacons,
                    presence::manage_peers,
                    scene::instantiate_pending_scenes,
                    connection::ecs::agent::send_agent_pose,
                    connection::ecs::agent::set_agent_tickrates
                        .run_if(on_timer(Duration::from_secs(5))),
                ),
            );
    }
}

#[derive(Component)]
pub struct Space(pub Hash);

#[derive(Component)]
#[require(ActiveSpaces, Transform)]
pub struct Peer(pub EndpointAddr);

#[derive(Component, Default)]
pub struct ActiveSpaces(pub HashMap<Hash, f32>);
