use std::sync::Arc;

use bevy::prelude::*;
use wds::actor::Actor;

mod event;
mod lifecycle;
pub mod thread;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        let nt = thread::NetworkingThread::spawn();

        let thread::NetworkEvent::SetActor(actor) =
            nt.event_rx.recv().expect("recv first network event")
        else {
            panic!("invalid first network event")
        };

        app.insert_resource(nt)
            .insert_resource(WdsActor(actor))
            .add_systems(FixedUpdate, event::recv_network_event)
            .add_systems(Last, lifecycle::cleanup_connections_on_exit);
    }
}

#[derive(Resource, Deref)]
pub struct WdsActor(pub Arc<Actor>);
