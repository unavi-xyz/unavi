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
    object_receive::ObjectTransformTarget,
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
    dyn_objects: Query<(
        Entity,
        &DynObjectId,
        &Transform,
        Option<&LinearVelocity>,
        Option<&AngularVelocity>,
    )>,
    locally_owned: Query<(), With<LocallyOwned>>,
    object_targets: Query<(), With<ObjectTransformTarget>>,
    mut object_targets_mut: Query<&mut ObjectTransformTarget>,
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

                for (entity, dyn_id, transform, lin_vel, ang_vel) in dyn_objects.iter() {
                    if dyn_id.0 != object_id {
                        continue;
                    }

                    if is_local {
                        commands.entity(entity).insert(LocallyOwned);
                    } else if owner.is_some() {
                        // Remote claimed - sync target to current transform to
                        // avoid lerping from stale position.
                        let synced_target =
                            ObjectTransformTarget::from_current(transform, lin_vel, ang_vel);
                        if let Ok(mut target) = object_targets_mut.get_mut(entity) {
                            *target = synced_target;
                        } else {
                            commands.entity(entity).insert(synced_target);
                        }
                        commands.entity(entity).remove::<LocallyOwned>();
                    } else {
                        // Released (owner disconnected) - remove both.
                        commands.entity(entity).remove::<(Grabbed, LocallyOwned)>();
                    }
                }
            }
            NetworkEvent::ObjectPoseUpdate { objects, .. } => {
                for (object_id, state) in objects {
                    for (entity, dyn_id, ..) in dyn_objects.iter() {
                        if dyn_id.0 != object_id {
                            continue;
                        }

                        // Only apply remote physics to non-owned objects.
                        if locally_owned.contains(entity) {
                            continue;
                        }

                        let new_target = ObjectTransformTarget {
                            translation: state.pos.into(),
                            rotation: state.rot.into(),
                            linear_velocity: state.vel.into(),
                            angular_velocity: state.ang_vel.into(),
                        };

                        // Add or update target for lerping.
                        if object_targets.contains(entity) {
                            if let Ok(mut target) = object_targets_mut.get_mut(entity) {
                                *target = new_target;
                            }
                        } else {
                            commands.entity(entity).insert(new_target);
                        }
                    }
                }
            }
            NetworkEvent::ObjectGrabChanged { object_id, grabbed } => {
                for (entity, dyn_id, ..) in dyn_objects.iter() {
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
