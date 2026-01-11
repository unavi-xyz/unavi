use std::sync::Arc;

use bevy::prelude::*;
use unavi_player::AvatarSpawner;

use crate::networking::{
    player_receive::OtherPlayer,
    thread::{InboundState, NetworkEvent, NetworkingThread},
};

#[derive(Component, Deref)]
pub struct PlayerInboundState(Arc<InboundState>);

pub fn recv_network_event(
    mut commands: Commands,
    nt: Res<NetworkingThread>,
    asset_server: Res<AssetServer>,
) {
    while let Ok(event) = nt.event_rx.try_recv() {
        match event {
            NetworkEvent::PlayerJoin { id, state } => {
                info!(%id, "spawning player");

                let entity = AvatarSpawner::new().spawn(&mut commands, &asset_server);

                commands
                    .entity(entity)
                    .insert((OtherPlayer(id), PlayerInboundState(state)));
            }
            NetworkEvent::PlayerLeave(_id) => {
                // TODO despawn player
            }
            NetworkEvent::SetActors(_) => {
                unreachable!("should only be called once on init")
            }
        }
    }
}
