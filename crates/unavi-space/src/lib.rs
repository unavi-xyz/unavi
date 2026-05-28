use std::time::Duration;

use bevy::{
    prelude::*,
    time::common_conditions::on_timer,
};
use blake3::Hash;

mod beacon;
mod connection;
mod gossip;
mod peer;
mod portal;
mod scene;
mod state;

pub struct SpacePlugin;

const TICKRATE_UPDATE_INTERVAL: Duration = Duration::from_secs(5);

impl Plugin for SpacePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(connection::connect_to_peer)
            .add_observer(connection::disconnect_peer)
            .add_observer(connection::register_protocol)
            .add_observer(gossip::join_space_topic)
            .add_observer(gossip::leave_space_topic)
            .add_observer(gossip::spawn_gossip)
            .add_observer(peer::add_space_state_sender)
            .add_observer(portal::spawn_portal_space)
            .add_observer(scene::despawn_space_scene)
            .add_observer(scene::spawn_space_scene)
            .add_observer(state::space::add_space_state)
            .add_observer(state::space::publish_state_update)
            .add_observer(state::space::remove_space_state)
            .add_systems(
                FixedUpdate,
                (
                    beacon::publish_beacons,
                    connection::ecs::agent::send_agent_pose,
                    gossip::poll_gossip,
                    connection::ecs::agent::set_agent_tickrates
                        .run_if(on_timer(TICKRATE_UPDATE_INTERVAL)),
                    peer::presence::manage_peers,
                    scene::instantiate_pending_scenes,
                ),
            );
    }
}

#[derive(Component)]
pub struct Space(pub Hash);
