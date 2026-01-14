use bevy::prelude::*;
use wds::actor::Actor;

mod event;
mod lifecycle;
mod player_publish;
mod player_receive;
pub mod thread;

pub use player_publish::TrackedBones;

pub struct NetworkingPlugin {
    pub wds_in_memory: bool,
}

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        let nt = thread::NetworkingThread::spawn(thread::NetworkingThreadOpts {
            wds_in_memory: self.wds_in_memory,
        });

        app.insert_resource(nt)
            .insert_resource(TrackedBones(TrackedBones::full()))
            .add_systems(
                FixedUpdate,
                (
                    event::recv_network_event,
                    player_publish::publish_player_transforms,
                    player_receive::receive_player_transforms,
                ),
            )
            .add_systems(Update, player_receive::lerp_to_target)
            .add_systems(PostUpdate, player_receive::apply_remote_bones)
            .add_systems(Last, lifecycle::shutdown_networking_thread);
    }
}

#[derive(Component, Clone)]
pub struct WdsActors {
    pub local: Actor,
    pub remote: Option<Actor>,
}
