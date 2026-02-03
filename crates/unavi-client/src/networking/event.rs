use std::sync::Arc;

use bevy::prelude::*;
use bevy_wds::{LocalActor, LocalBlobs, RemoteActor};
use unavi_avatar::{AvatarSpawner, AverageVelocity, Grounded, default_character_animations};

use crate::networking::{
    AgentTickrateConfig,
    agent_receive::{RemoteAgent, TrackedBoneState, TransformTarget},
    thread::{InboundState, NetworkEvent, NetworkingThread},
};

#[derive(Component, Deref)]
pub struct AgentInboundState(Arc<InboundState>);

pub fn recv_network_event(
    mut commands: Commands,
    mut nt: ResMut<NetworkingThread>,
    asset_server: Res<AssetServer>,
    local_actors: Query<Entity, With<LocalActor>>,
    local_blobs: Query<Entity, With<LocalBlobs>>,
) {
    while let Ok(event) = nt.event_rx.try_recv() {
        match event {
            NetworkEvent::AgentJoin { id, state } => {
                info!(%id, "spawning agent");

                let entity = commands
                    .spawn((
                        RemoteAgent(id),
                        AgentInboundState(state),
                        AgentTickrateConfig::default(),
                        Grounded(true),
                        Transform::default(),
                        Visibility::default(),
                        TransformTarget::default(),
                    ))
                    .id();

                let avatar = AvatarSpawner::new().spawn(&mut commands, &asset_server);
                let animations = default_character_animations(&asset_server);

                commands.entity(avatar).insert((
                    AverageVelocity {
                        target: Some(entity),
                        ..Default::default()
                    },
                    TrackedBoneState::default(),
                    animations,
                    Transform::default(),
                ));

                commands.entity(entity).add_child(avatar);
            }
            NetworkEvent::AgentLeave(_id) => {
                // TODO: Despawn agent.
            }
            NetworkEvent::SetLocalBlobs(blobs) => {
                for ent in local_blobs {
                    commands.entity(ent).despawn();
                }
                commands.spawn(LocalBlobs(blobs));
            }
            NetworkEvent::SetLocalActor(actor) => {
                for ent in local_actors {
                    commands.entity(ent).despawn();
                }
                commands.spawn(LocalActor(actor));
            }
            NetworkEvent::AddRemoteActor(actor) => {
                // TODO handle disconnects / multiple adds

                commands.spawn(RemoteActor(actor));
            }
        }
    }
}
