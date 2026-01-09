use bevy::prelude::*;
use wds::actor::Actor;

mod event;
mod lifecycle;
mod player_publish;
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

        let thread::NetworkEvent::SetActors(actors) =
            nt.event_rx.recv().expect("recv first network event")
        else {
            panic!("invalid first network event")
        };

        app.insert_resource(nt)
            .insert_resource(actors)
            .init_resource::<TrackedBones>()
            .add_systems(
                FixedUpdate,
                (
                    event::recv_network_event,
                    player_publish::publish_player_transforms,
                ),
            )
            .add_systems(Last, lifecycle::shutdown_networking_thread);
    }
}

#[derive(Resource, Clone)]
pub struct WdsActors {
    pub local: Actor,
    pub remote: Option<Actor>,
}
