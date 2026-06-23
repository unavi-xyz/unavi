use std::time::Duration;

use bevy::{
    prelude::*,
    time::common_conditions::on_timer,
};
use blake3::Hash;
use unavi_manifold::{
    echo::maintain_seam_echoes,
    transition::apply_seam_crossings,
};

pub mod anchor;
mod beacon;
mod connection;
mod gossip;
pub mod membership;
pub mod peer;
mod portal;
mod portal_bridge;
pub mod quota;
mod scene;
pub mod spawn;
pub mod state;

pub struct SpacePlugin;

const TICKRATE_UPDATE_INTERVAL: Duration = Duration::from_secs(5);

impl Plugin for SpacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<anchor::SpaceGridAllocator>()
            .init_resource::<anchor::ActiveSpace>()
            .add_observer(anchor::assign_anchor)
            .add_observer(anchor::reparent_doc_traveler)
            .add_observer(membership::self_own_space)
            .add_observer(membership::parent_doc_under_space)
            .add_observer(membership::register_on_owner_change)
            .add_observer(membership::deregister_doc_membership)
            .add_observer(anchor::promote_first_space)
            .add_observer(anchor::release_anchor)
            .add_observer(connection::connect_to_peer)
            .add_observer(connection::disconnect_peer)
            .add_observer(connection::ecs::remote::despawn_remote_agent)
            .add_observer(connection::register_protocol)
            .add_observer(gossip::join_space_topic)
            .add_observer(gossip::leave_space_topic)
            .add_observer(gossip::spawn_gossip)
            .add_observer(portal::spawn_portal_space)
            .add_observer(portal_bridge::sync_portal_config)
            .add_observer(portal_bridge::clear_portal_config)
            .add_observer(scene::despawn_space_scene)
            .add_observer(scene::spawn_space_scene)
            .add_systems(
                PostUpdate,
                (anchor::recenter_active_space, anchor::apply_anchor_offsets)
                    .chain()
                    .after(apply_seam_crossings)
                    // Recenter the world before the seam echoes are posed, so a
                    // crossing's echo clones aren't placed from stale positions
                    // for a frame.
                    .before(maintain_seam_echoes)
                    .before(TransformSystems::Propagate),
            )
            .add_systems(
                FixedUpdate,
                (
                    beacon::publish_beacons,
                    connection::ecs::agent::send_agent_pose,
                    connection::ecs::object::send_object_poses,
                    connection::ecs::object::reconcile_object_authority,
                    (
                        connection::ecs::object::apply_remote_objects,
                        connection::ecs::object::advance_object_interp,
                    )
                        .chain(),
                    gossip::poll_gossip,
                    connection::ecs::agent::set_agent_tickrates
                        .run_if(on_timer(TICKRATE_UPDATE_INTERVAL)),
                    peer::presence::manage_peers,
                    scene::instantiate_pending_scenes,
                    state::pin::spawn_pinned_docs,
                    state::pin::instantiate_pinned_docs,
                    state::pin::despawn_unpinned_docs,
                ),
            )
            .add_systems(
                Update,
                (
                    gossip::publish_active_space,
                    connection::ecs::remote::apply_remote_poses,
                    connection::ecs::remote::advance_remote_lerp,
                )
                    .chain(),
            );
    }
}

#[derive(Component)]
pub struct Space(pub Hash);
