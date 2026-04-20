use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use avian3d::dynamics::rigid_body::{AngularVelocity, LinearVelocity};
use bevy::prelude::*;
use bevy_wds::{LocalActor, LocalBlobs, SyncTargets};
use blake3::Hash;
use iroh::EndpointId;
use unavi_avatar::{
    Avatar, Grounded,
    animation::{defaults::default_character_animations, velocity::AverageVelocity},
};
use wds::actor::Actor;

use crate::networking::{
    AgentTickrateConfig,
    agent::receive::{RemoteAgent, TrackedBoneState, TransformTarget},
    object::publish::{DynObjectId, Grabbed, LocallyOwned},
    object::receive::ObjectTransformTarget,
    peer::{Peer, PeerKnownSpaces, PeerStateStatus},
    player::{OwnedObjectEntry, RemotePlayerState},
    thread::{InboundState, NetworkEvent, NetworkingThread},
};

/// Our local endpoint ID for ownership comparison.
#[derive(Resource)]
pub struct LocalEndpointId(pub EndpointId);

/// Record hashes received in physics gossip for documents not yet spawned locally.
/// Drained by `fetch_dynamic_docs` in the space plugin.
#[derive(Default, Resource)]
pub struct PendingDynamicDocs(pub std::collections::HashSet<blake3::Hash>);

#[derive(Component, Deref)]
pub struct AgentInboundState(pub Arc<InboundState>);

#[expect(clippy::too_many_lines)]
pub fn recv_network_event(
    mut commands: Commands,
    mut nt: ResMut<NetworkingThread>,
    asset_server: Res<AssetServer>,
    local_actors: Query<Entity, With<LocalActor>>,
    mut sync_targets: Query<&mut SyncTargets>,
    local_endpoint: Option<Res<LocalEndpointId>>,
    peers: Query<(Entity, &Peer)>,
    mut peer_known_spaces: Query<(&Peer, &mut PeerKnownSpaces)>,
    mut peer_state_status: Query<&mut PeerStateStatus>,
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
    mut pending_remote_actors: Local<Vec<Actor>>,
    mut pending_docs: ResMut<PendingDynamicDocs>,
    // Spaces seen from peers before the peer entity existed.
    mut pending_spaces: Local<HashMap<EndpointId, HashSet<Hash>>>,
) {
    if !pending_remote_actors.is_empty()
        && let Ok(mut targets) = sync_targets.single_mut()
    {
        targets.0.append(&mut pending_remote_actors);
    }

    while let Ok(event) = nt.event_rx.try_recv() {
        match event {
            NetworkEvent::SetLocalEndpoint(id) => {
                commands.insert_resource(LocalEndpointId(id));
            }
            NetworkEvent::PeerJoin { id, state } => {
                // Deduplicate: skip if already tracked.
                if peers.iter().any(|(_, p)| p.0 == id) {
                    continue;
                }

                info!(%id, "spawning peer");

                let known_spaces = pending_spaces.remove(&id).unwrap_or_default();

                let entity = commands
                    .spawn((
                        Peer(id),
                        PeerKnownSpaces(known_spaces),
                        PeerStateStatus::default(),
                        RemotePlayerState,
                        RemoteAgent(id),
                        AgentInboundState(state),
                        AgentTickrateConfig::default(),
                        Grounded(true),
                        Transform::default(),
                        Visibility::default(),
                        TransformTarget::default(),
                    ))
                    .id();

                let avatar = commands
                    .spawn((
                        Avatar,
                        AverageVelocity {
                            target: Some(entity),
                            ..Default::default()
                        },
                        TrackedBoneState::default(),
                        default_character_animations(&asset_server),
                        Transform::default(),
                    ))
                    .id();

                commands.entity(entity).add_child(avatar);
            }
            NetworkEvent::PeerLeft(id) => {
                if let Some((entity, _)) = peers.iter().find(|(_, p)| p.0 == id) {
                    info!(%id, "despawning peer");
                    commands.entity(entity).despawn();
                }
            }
            NetworkEvent::PeerJoinedSpace { peer, space } => {
                if let Some((entity, _)) = peers.iter().find(|(_, p)| p.0 == peer) {
                    if let Ok((_, mut known)) = peer_known_spaces.get_mut(entity) {
                        known.0.insert(space);
                    }
                } else {
                    pending_spaces.entry(peer).or_default().insert(space);
                }
            }
            NetworkEvent::PeerStateReceived { peer, state } => {
                let Some((entity, _)) = peers.iter().find(|(_, p)| p.0 == peer) else {
                    continue;
                };

                // Spawn OwnedObjectEntry children from peer state.
                for obj in &state.objects {
                    let child = commands
                        .spawn(OwnedObjectEntry {
                            record_id: Hash::from_bytes(obj.record_id),
                            node_id: obj.node_id.clone(),
                        })
                        .id();
                    commands.entity(entity).add_child(child);
                }

                if let Ok(mut status) = peer_state_status.get_mut(entity) {
                    *status = PeerStateStatus::Synced;
                }
            }
            NetworkEvent::PeerStateDelta { peer: _, delta: _ } => {
                // Delta handling deferred — full state sync covers initial state.
                // Real-time object ownership changes arrive via ObjectClaim gossip.
            }
            NetworkEvent::SetLocalWds { actor, blobs } => {
                for ent in local_actors.iter() {
                    commands.entity(ent).despawn();
                }
                commands.spawn((LocalActor(actor), LocalBlobs(blobs)));
            }
            NetworkEvent::SetRemoteActor(actor) => {
                if let Ok(mut targets) = sync_targets.single_mut() {
                    targets.0.push(actor);
                } else {
                    pending_remote_actors.push(actor);
                }
            }
            NetworkEvent::ObjectOwnershipChanged { object_id, owner } => {
                let is_local = local_endpoint.as_ref().is_some_and(|e| owner == Some(e.0));

                if is_local {
                    info!(object = %object_id.node, "claimed object (local)");
                } else if let Some(remote) = owner {
                    info!(object = %object_id.node, owner = %remote, "object claimed by remote");
                } else {
                    info!(object = %object_id.node, "object released");
                }

                for (entity, dyn_id, transform, lin_vel, ang_vel) in dyn_objects.iter() {
                    if dyn_id.0 != object_id {
                        continue;
                    }

                    if is_local {
                        commands.entity(entity).insert(LocallyOwned);
                    } else if owner.is_some() {
                        let synced_target =
                            ObjectTransformTarget::from_current(transform, lin_vel, ang_vel);
                        if let Ok(mut target) = object_targets_mut.get_mut(entity) {
                            *target = synced_target;
                        } else {
                            commands.entity(entity).insert(synced_target);
                        }
                        commands.entity(entity).remove::<LocallyOwned>();
                    } else {
                        commands.entity(entity).remove::<(Grabbed, LocallyOwned)>();
                    }
                }
            }
            NetworkEvent::ObjectPoseUpdate { objects, .. } => {
                for (object_id, state) in objects {
                    let mut matched = false;
                    for (entity, dyn_id, ..) in dyn_objects.iter() {
                        if dyn_id.0 != object_id {
                            continue;
                        }
                        matched = true;

                        if locally_owned.contains(entity) {
                            continue;
                        }

                        let new_target = ObjectTransformTarget {
                            translation: state.pos.into(),
                            rotation: state.rot.into(),
                            linear_velocity: state.vel.into(),
                            angular_velocity: state.ang_vel.into(),
                        };

                        if object_targets.contains(entity) {
                            if let Ok(mut target) = object_targets_mut.get_mut(entity) {
                                *target = new_target;
                            }
                        } else {
                            commands.entity(entity).insert(new_target);
                        }
                    }
                    if !matched {
                        pending_docs.0.insert(object_id.record);
                    }
                }
            }
            NetworkEvent::ObjectGrabChanged { object_id, grabbed } => {
                for (entity, dyn_id, ..) in dyn_objects.iter() {
                    if dyn_id.0 != object_id {
                        continue;
                    }

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
