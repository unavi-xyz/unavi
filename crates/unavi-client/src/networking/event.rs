use bevy::prelude::*;

use crate::networking::thread::{NetworkEvent, NetworkingThread};

pub fn recv_network_event(nt: Res<NetworkingThread>) {
    while let Ok(event) = nt.event_rx.try_recv() {
        match event {
            NetworkEvent::Connected { id, .. } => {
                info!("connected to {id}");
            }
            NetworkEvent::ConnectionClosed { id, message } => {
                info!("connection {id} closed: {message}");
            }
            NetworkEvent::SetActor(_) => {
                unreachable!("should only be called once on init")
            }
        }
    }
}
