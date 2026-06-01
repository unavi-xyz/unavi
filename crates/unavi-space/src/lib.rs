use std::time::Duration;

use bevy::{
    prelude::*,
    time::common_conditions::on_timer,
};
use blake3::Hash;

pub mod anchor;
mod beacon;
mod connection;
mod gossip;
pub mod membership;
mod peer;
mod portal;
mod scene;
mod state;

pub struct SpacePlugin;

const TICKRATE_UPDATE_INTERVAL: Duration = Duration::from_secs(5);

impl Plugin for SpacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<anchor::SpaceGridAllocator>()
            .init_resource::<anchor::ActiveSpace>()
            .add_observer(anchor::assign_anchor)
            .add_observer(anchor::promote_active_on_teleport)
            .add_observer(anchor::reparent_doc_traveler)
            .add_observer(membership::self_own_space)
            .add_observer(membership::parent_doc_under_space)
            .add_observer(membership::register_on_owner_change)
            .add_observer(membership::deregister_doc_membership)
            .add_observer(anchor::promote_first_space)
            .add_observer(anchor::release_anchor)
            .add_observer(connection::connect_to_peer)
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
            .add_systems(Update, anchor::apply_anchor_offsets)
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
