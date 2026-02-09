use std::sync::Arc;

use avian3d::dynamics::rigid_body::{AngularVelocity, LinearVelocity};
use bevy::prelude::*;
use bevy_wds::{LocalActor, LocalBlobs, RemoteActor};
use iroh::EndpointId;
use unavi_avatar::{AvatarSpawner, AverageVelocity, Grounded, default_character_animations};

use crate::networking::{
    AgentTickrateConfig,
    agent_receive::{RemoteAgent, TrackedBoneState, TransformTarget},
    object_publish::{DynObjectId, Grabbed, LocallyOwned},
    thread::{InboundState, NetworkEvent, NetworkingThread},
};

/// Our local endpoint ID for ownership comparison.
#[derive(Resource)]
pub struct LocalEndpointId(pub EndpointId);

#[derive(Component, Deref)]
pub struct AgentInboundState(Arc<InboundState>);

pub fn recv_network_event(
    mut commands: Commands,
    mut nt: ResMut<NetworkingThread>,
    asset_server: Res<AssetServer>,
    local_actors: Query<Entity, With<LocalActor>>,
    local_blobs: Query<Entity, With<LocalBlobs>>,
    local_endpoint: Option<Res<LocalEndpointId>>,
    dyn_objects: Query<(Entity, &DynObjectId)>,
    locally_owned: Query<(), With<LocallyOwned>>,
    mut object_transforms: Query<&mut Transform, With<DynObjectId>>,
    mut velocities: Query<(&mut LinearVelocity, &mut AngularVelocity), With<DynObjectId>>,
) {
    while let Ok(event) = nt.event_rx.try_recv() {
        match event {
            NetworkEvent::SetLocalEndpoint(id) => {
                commands.insert_resource(LocalEndpointId(id));
            }
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
                for ent in local_blobs.iter() {
                    commands.entity(ent).despawn();
                }
                commands.spawn(LocalBlobs(blobs));
            }
            NetworkEvent::SetLocalActor(actor) => {
                for ent in local_actors.iter() {
                    commands.entity(ent).despawn();
                }
                commands.spawn(LocalActor(actor));
            }
            NetworkEvent::AddRemoteActor(actor) => {
                // TODO: Handle disconnects / multiple adds.
                commands.spawn(RemoteActor(actor));
            }
            NetworkEvent::ObjectOwnershipChanged { object_id, owner } => {
                let is_local = local_endpoint.as_ref().is_some_and(|e| owner == Some(e.0));

                if is_local {
                    info!(object = %object_id.index, "claimed object (local)");
                } else if let Some(remote) = owner {
                    info!(object = %object_id.index, owner = %remote, "object claimed by remote");
                } else {
                    info!(object = %object_id.index, "object released");
                }

                for (entity, dyn_id) in dyn_objects.iter() {
                    if dyn_id.0 != object_id {
                        continue;
                    }

                    if is_local {
                        commands.entity(entity).insert(LocallyOwned);
                    } else if owner.is_some() {
                        // Remote claimed - don't add Grabbed here, wait for
                        // ObjectGrabChanged.
                        commands.entity(entity).remove::<LocallyOwned>();
                    } else {
                        // Released (owner disconnected) - remove both.
                        commands.entity(entity).remove::<(Grabbed, LocallyOwned)>();
                    }
                }
            }
            NetworkEvent::ObjectPoseUpdate { objects, .. } => {
                for (object_id, state) in objects {
                    for (entity, dyn_id) in dyn_objects.iter() {
                        if dyn_id.0 != object_id {
                            continue;
                        }

                        // Only apply remote physics to non-owned objects.
                        if locally_owned.contains(entity) {
                            continue;
                        }

                        if let Ok(mut transform) = object_transforms.get_mut(entity) {
                            transform.translation = state.pos.into();
                            transform.rotation = state.rot.into();
                        }

                        // Apply velocities for smooth prediction/interpolation.
                        if let Ok((mut lin_vel, mut ang_vel)) = velocities.get_mut(entity) {
                            lin_vel.0 = state.vel.into();
                            ang_vel.0 = state.ang_vel.into();
                        }
                    }
                }
            }
            NetworkEvent::ObjectGrabChanged { object_id, grabbed } => {
                for (entity, dyn_id) in dyn_objects.iter() {
                    if dyn_id.0 != object_id {
                        continue;
                    }

                    // Don't modify grab state of locally owned objects.
                    if locally_owned.contains(entity) {
                        continue;
                    }

                    if grabbed {
                        commands.entity(entity).insert(Grabbed);
                    } else {
                        commands.entity(entity).remove::<Grabbed>();
                    }
                }
            }
        }
    }
}
