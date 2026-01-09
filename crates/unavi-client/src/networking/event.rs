use std::sync::Arc;

use bevy::prelude::*;

use crate::networking::{
    player_receive::OtherPlayer,
    thread::{InboundState, NetworkEvent, NetworkingThread},
};

#[derive(Component, Deref)]
pub struct PlayerInboundState(Arc<InboundState>);

pub fn recv_network_event(mut commands: Commands, nt: Res<NetworkingThread>) {
    while let Ok(event) = nt.event_rx.try_recv() {
        match event {
            NetworkEvent::PlayerJoin { id, state } => {
                info!(%id, "spawning player");
                commands.spawn((
                    OtherPlayer(id),
                    PlayerInboundState(state),
                    Transform::default(),
                ));
            }
            NetworkEvent::PlayerLeave(_id) => {
                todo!()
            }
            NetworkEvent::SetActors(_) => {
                unreachable!("should only be called once on init")
            }
        }
    }
}
