use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};

mod agent_publish;
mod agent_receive;
mod event;
mod lifecycle;
pub mod thread;
mod tickrate;

pub use agent_publish::TrackedBones;
pub use tickrate::AgentTickrateConfig;

pub struct NetworkingPlugin {
    pub wds_in_memory: bool,
}

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        let nt = thread::NetworkingThread::spawn(thread::NetworkingThreadOpts {
            wds_in_memory: self.wds_in_memory,
        });

        app.insert_resource(nt)
            .insert_resource(TrackedBones::default())
            .add_systems(
                FixedUpdate,
                (
                    event::recv_network_event,
                    agent_publish::publish_agent_transforms,
                    agent_receive::receive_agent_transforms,
                )
                    .after(unavi_avatar::animation::weights::play_avatar_animations),
            )
            .add_systems(
                Update,
                (
                    agent_receive::lerp_to_target,
                    tickrate::update_peer_tickrates.run_if(on_timer(Duration::from_secs(2))),
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    agent_receive::receive_remote_bones,
                    agent_receive::slerp_to_target,
                )
                    .chain(),
            )
            .add_systems(Last, lifecycle::shutdown_networking_thread);
    }
}
