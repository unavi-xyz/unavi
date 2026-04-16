use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};

use crate::networking::agent::publish::TrackedBones;

pub mod agent;
pub mod event;
mod lifecycle;
pub mod object;
pub mod peer;
pub mod player;
pub mod portal;
mod publish_utils;
pub mod thread;
pub mod tickrate;

pub use tickrate::AgentTickrateConfig;

const TICKRATE_UPDATE_RATE: Duration = Duration::from_secs(2);

pub struct NetworkingPlugin {
    pub wds_in_memory: bool,
}

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        let nt = thread::NetworkingThread::spawn(thread::NetworkingThreadOpts {
            wds_in_memory: self.wds_in_memory,
        });

        app.insert_resource(nt)
            .insert_resource(TrackedBones::desktop())
            .init_resource::<object::publish::ObjectBaselines>()
            .init_resource::<event::PendingDynamicDocs>()
            .add_systems(
                FixedUpdate,
                (
                    event::recv_network_event,
                    agent::publish::publish_agent_transforms,
                    agent::receive::receive_agent_transforms,
                    object::publish::detect_dynamic_objects,
                    object::publish::detect_removed_objects,
                    object::publish::publish_initial_object_iframe,
                    object::publish::publish_object_physics,
                    object::ownership::on_locally_claimed,
                )
                    .after(unavi_avatar::animation::weights::play_avatar_animations),
            )
            .add_systems(
                FixedUpdate,
                peer::state::request_peer_state.after(event::recv_network_event),
            )
            .add_systems(
                Update,
                (
                    agent::receive::lerp_to_target,
                    object::receive::lerp_objects_to_target,
                    tickrate::update_peer_tickrates.run_if(on_timer(TICKRATE_UPDATE_RATE)),
                    tickrate::update_object_tickrates.run_if(on_timer(TICKRATE_UPDATE_RATE)),
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    agent::receive::receive_remote_bones,
                    agent::receive::slerp_to_target,
                    object::ownership::on_locally_released,
                    object::pin::refresh_pins_on_grab,
                    object::pin::tick_pins,
                    peer::lifecycle::check_orphan_peers,
                    player::sync::detect_local_changes,
                    player::sync::broadcast_state_delta,
                )
                    .chain(),
            )
            .add_systems(Last, lifecycle::shutdown_networking_thread);
    }
}
