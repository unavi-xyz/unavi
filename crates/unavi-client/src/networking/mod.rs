use std::sync::Arc;

use bevy::prelude::*;
use wired_data_store::Actor;

mod event;
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
            .add_systems(FixedUpdate, event::recv_network_event);
    }
}

#[derive(Resource, Deref)]
pub struct WdsActor(pub Arc<Actor>);
