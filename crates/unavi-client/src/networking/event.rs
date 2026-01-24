use std::sync::Arc;

use bevy::prelude::*;
use unavi_player::{
    AvatarSpawner, Grounded,
    animation::{defaults::default_character_animations, velocity::AverageVelocity},
};

use crate::networking::{
    WdsActors,
    player_receive::{BoneRotationTargets, RemotePlayer, TransformTarget},
    thread::{InboundState, NetworkEvent, NetworkingThread},
};

#[derive(Component, Deref)]
pub struct PlayerInboundState(Arc<InboundState>);

pub fn recv_network_event(
    mut commands: Commands,
    mut nt: ResMut<NetworkingThread>,
    asset_server: Res<AssetServer>,
    prev_actors: Query<Entity, With<WdsActors>>,
) {
    while let Ok(event) = nt.event_rx.try_recv() {
        match event {
            NetworkEvent::PlayerJoin { id, state } => {
                info!(%id, "spawning player");

                let entity = commands
                    .spawn((
                        RemotePlayer(id),
                        PlayerInboundState(state),
                        Grounded(true),
                        Transform::default(),
                        TransformTarget::default(),
                    ))
                    .id();

                let avatar = AvatarSpawner::new().spawn(&mut commands, &asset_server);
                let _animations = default_character_animations(&asset_server);

                commands.entity(avatar).insert((
                    AverageVelocity {
                        target: Some(entity),
                        ..Default::default()
                    },
                    BoneRotationTargets::default(),
                    // TODO fix animation application to avoid tracked bones
                    // animations,
                    Transform::default(),
                ));

                commands.entity(entity).add_child(avatar);
            }
            NetworkEvent::PlayerLeave(_id) => {
                // TODO despawn player
            }
            NetworkEvent::SetActors(actors) => {
                for ent in prev_actors {
                    commands.entity(ent).despawn();
                }

                commands.spawn(actors);
            }
        }
    }
}
