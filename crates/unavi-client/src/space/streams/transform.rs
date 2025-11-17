use std::{collections::HashMap, sync::mpsc::SyncSender};

use bevy::{prelude::*, tasks::futures_lite::StreamExt};
use bevy_vrm::BoneName;
use tarpc::tokio_util::codec::LengthDelimitedCodec;
use unavi_player::{AvatarBones, AvatarSpawner};
use unavi_server_service::{
    TRANSFORM_LENGTH_FIELD_LENGTH, TRANSFORM_MAX_FRAME_LENGTH, TrackingIFrame, TrackingPFrame,
    TrackingUpdate, from_server::TransformMeta,
};
use wtransport::RecvStream;

use crate::space::{Host, HostTransformChannel, PlayerHost, RemotePlayer, RemotePlayerState};

use super::{PFRAME_ROTATION_SCALE, PFRAME_TRANSLATION_SCALE};

#[derive(Clone)]
pub struct RecievedTransform {
    player_id: u64,
    update: TrackingUpdate,
}

pub async fn recv_transform_stream(
    stream: RecvStream,
    transform_tx: SyncSender<RecievedTransform>,
) -> anyhow::Result<()> {
    let mut framed = LengthDelimitedCodec::builder()
        .little_endian()
        .length_field_length(TRANSFORM_LENGTH_FIELD_LENGTH)
        .max_frame_length(TRANSFORM_MAX_FRAME_LENGTH)
        .new_read(stream);

    let Some(meta_bytes) = framed.next().await else {
        return Ok(());
    };

    let (meta, _) =
        bincode::decode_from_slice::<TransformMeta, _>(&meta_bytes?, bincode::config::standard())?;

    while let Some(bytes) = framed.next().await {
        let (update, _) = bincode::serde::decode_from_slice::<TrackingUpdate, _>(
            &bytes?,
            bincode::config::standard(),
        )?;

        if let Err(e) = transform_tx.try_send(RecievedTransform {
            player_id: meta.player,
            update,
        }) {
            debug!(
                "Dropped transform update for player {}: {:?}",
                meta.player, e
            );
        }
    }

    Ok(())
}

pub fn apply_player_transforms(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    hosts: Query<(Entity, &Host, &HostTransformChannel)>,
    mut remote_players: Query<(
        Entity,
        &RemotePlayer,
        &PlayerHost,
        &mut RemotePlayerState,
        &AvatarBones,
    )>,
    mut bone_transforms: Query<&mut Transform, With<BoneName>>,
    mut pending_spawns: Local<HashMap<(Entity, u64), Entity>>,
) {
    // Clean up pending spawns once they become visible in queries.
    pending_spawns.retain(|_, entity| !remote_players.iter().any(|(e, ..)| e == *entity));

    for (host_entity, _, channel) in hosts.iter() {
        let Ok(rx) = channel.rx.lock() else {
            continue;
        };

        // Drain all pending messages, keeping only latest per player.
        let mut latest_updates: HashMap<u64, RecievedTransform> = HashMap::new();

        while let Ok(received) = rx.try_recv() {
            latest_updates
                .entry(received.player_id)
                .and_modify(|existing| {
                    // If new message is IFrame, always replace.
                    // If new message is PFrame and existing is PFrame, replace.
                    // If new message is PFrame and existing is IFrame, keep IFrame.
                    match (&existing.update, &received.update) {
                        (_, TrackingUpdate::IFrame(_)) => *existing = received.clone(),
                        (TrackingUpdate::PFrame(_), TrackingUpdate::PFrame(_)) => {
                            *existing = received.clone()
                        }
                        (TrackingUpdate::IFrame(_), TrackingUpdate::PFrame(_)) => {
                            // Keep the IFrame, discard the PFrame.
                        }
                    }
                })
                .or_insert(received);
        }

        // Process only the latest updates.
        for (_, received) in latest_updates {
            // Find existing remote player (in query or pending spawns).
            let existing_entity = remote_players
                .iter()
                .find(|(_, remote, player_host, ..)| {
                    remote.player_id == received.player_id && player_host.0 == host_entity
                })
                .map(|(e, ..)| e)
                .or_else(|| {
                    pending_spawns
                        .get(&(host_entity, received.player_id))
                        .copied()
                });

            // Spawn remote avatar if not found.
            if existing_entity.is_none() {
                info!("Spawning remote player: {}", received.player_id);
                let avatar = AvatarSpawner::default().spawn(&mut commands, &asset_server);

                // Store initial state with first IFrame if applicable.
                let initial_state = match &received.update {
                    TrackingUpdate::IFrame(iframe) => RemotePlayerState {
                        last_iframe: Some(iframe.clone()),
                    },
                    TrackingUpdate::PFrame(_) => RemotePlayerState::default(),
                };

                commands.entity(avatar).insert((
                    RemotePlayer {
                        player_id: received.player_id,
                    },
                    PlayerHost(host_entity),
                    initial_state,
                ));

                // Track pending spawn to prevent duplicates.
                pending_spawns.insert((host_entity, received.player_id), avatar);
            }

            // Apply transform update.
            match received.update {
                TrackingUpdate::IFrame(iframe) => {
                    // Find the player to update.
                    for (_, remote, player_host, mut state, avatar_bones) in
                        remote_players.iter_mut()
                    {
                        if remote.player_id == received.player_id && player_host.0 == host_entity {
                            apply_iframe(&iframe, avatar_bones, &mut bone_transforms);
                            state.last_iframe = Some(iframe.clone());
                            break;
                        }
                    }
                }
                TrackingUpdate::PFrame(pframe) => {
                    for (_, remote, player_host, state, avatar_bones) in remote_players.iter() {
                        if remote.player_id == received.player_id && player_host.0 == host_entity {
                            if let Some(last_iframe) = &state.last_iframe {
                                apply_pframe(
                                    &pframe,
                                    last_iframe,
                                    avatar_bones,
                                    &mut bone_transforms,
                                );
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
}

fn apply_iframe(
    iframe: &TrackingIFrame,
    avatar_bones: &AvatarBones,
    bone_transforms: &mut Query<&mut Transform, With<BoneName>>,
) {
    // Apply hip position and rotation directly to hip bone.
    if let Some(&hips_entity) = avatar_bones.get(&BoneName::Hips)
        && let Ok(mut transform) = bone_transforms.get_mut(hips_entity)
    {
        transform.translation = Vec3::from_array(iframe.translation);
        transform.rotation = Quat::from_array(iframe.rotation);
    }

    // Apply bone rotations to all other joints.
    for joint in &iframe.joints {
        if joint.id == BoneName::Hips {
            continue;
        }

        if let Some(&bone_entity) = avatar_bones.get(&joint.id)
            && let Ok(mut transform) = bone_transforms.get_mut(bone_entity)
        {
            transform.rotation = Quat::from_array(joint.rotation);
        }
    }
}

fn apply_pframe(
    pframe: &TrackingPFrame,
    last_iframe: &TrackingIFrame,
    avatar_bones: &AvatarBones,
    bone_transforms: &mut Query<&mut Transform, With<BoneName>>,
) {
    // Apply hip position and rotation directly to hip bone.
    if let Some(&hips_entity) = avatar_bones.get(&BoneName::Hips)
        && let Ok(mut transform) = bone_transforms.get_mut(hips_entity)
    {
        transform.translation = Vec3::new(
            pframe.translation[0] as f32 / PFRAME_TRANSLATION_SCALE,
            pframe.translation[1] as f32 / PFRAME_TRANSLATION_SCALE,
            pframe.translation[2] as f32 / PFRAME_TRANSLATION_SCALE,
        ) + Vec3::from_array(last_iframe.translation);

        transform.rotation = Quat::from_array([
            pframe.rotation[0] as f32 / PFRAME_ROTATION_SCALE,
            pframe.rotation[1] as f32 / PFRAME_ROTATION_SCALE,
            pframe.rotation[2] as f32 / PFRAME_ROTATION_SCALE,
            pframe.rotation[3] as f32 / PFRAME_ROTATION_SCALE,
        ]) * Quat::from_array(last_iframe.rotation);
    }

    // Apply bone rotations to all other joints.
    for joint in &pframe.joints {
        if joint.id == BoneName::Hips {
            continue;
        }

        if let Some(&bone_entity) = avatar_bones.get(&joint.id)
            && let Ok(mut transform) = bone_transforms.get_mut(bone_entity)
            && let Some(iframe_joint) = last_iframe.joints.iter().find(|j| j.id == joint.id)
        {
            transform.rotation = Quat::from_array([
                joint.rotation[0] as f32 / PFRAME_ROTATION_SCALE,
                joint.rotation[1] as f32 / PFRAME_ROTATION_SCALE,
                joint.rotation[2] as f32 / PFRAME_ROTATION_SCALE,
                joint.rotation[3] as f32 / PFRAME_ROTATION_SCALE,
            ]) * Quat::from_array(iframe_joint.rotation);
        }
    }
}
