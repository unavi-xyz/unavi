use std::sync::mpsc::Sender;

use bevy::{prelude::*, tasks::futures_lite::StreamExt};
use bevy_vrm::BoneName;
use tarpc::tokio_util::codec::LengthDelimitedCodec;
use unavi_player::{AvatarBones, PlayerSpawner};
use unavi_server_service::{
    TRANSFORM_LENGTH_FIELD_LENGTH, TRANSFORM_MAX_FRAME_LENGTH, TrackingUpdate,
    from_server::TransformMeta,
};
use wtransport::RecvStream;

use crate::space::{Host, HostTransformChannel, PlayerHost, RemotePlayer, RemotePlayerState};

use super::{PFRAME_ROTATION_SCALE, PFRAME_TRANSLATION_SCALE};

pub struct RecievedTransform {
    player_id: u64,
    update: TrackingUpdate,
}

pub async fn recv_transform_stream(
    stream: RecvStream,
    transform_tx: Sender<RecievedTransform>,
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

        transform_tx.send(RecievedTransform {
            player_id: meta.player,
            update,
        })?;
    }

    Ok(())
}

pub fn apply_player_transforms(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    hosts: Query<(Entity, &Host, &HostTransformChannel)>,
    mut remote_players: Query<(
        &RemotePlayer,
        &PlayerHost,
        &mut RemotePlayerState,
        &AvatarBones,
    )>,
    mut bone_transforms: Query<&mut Transform, With<BoneName>>,
) {
    for (host_entity, _, channel) in hosts.iter() {
        let Ok(rx) = channel.rx.lock() else {
            continue;
        };

        while let Ok(received) = rx.try_recv() {
            // Find existing remote player or spawn new one.
            let mut found = false;
            let mut player_entity = None;

            for (remote, player_host, _, _) in remote_players.iter() {
                if remote.player_id == received.player_id && player_host.0 == host_entity {
                    found = true;
                    break;
                }
            }

            // Spawn remote player if not found.
            if !found {
                let spawned = PlayerSpawner::default().spawn(&mut commands, &asset_server);
                commands.entity(spawned).insert((
                    RemotePlayer {
                        player_id: received.player_id,
                    },
                    PlayerHost(host_entity),
                    RemotePlayerState::default(),
                ));
                player_entity = Some(spawned);
            }

            // Apply transform update.
            match received.update {
                TrackingUpdate::IFrame(iframe) => {
                    // Find the player to update.
                    for (remote, player_host, mut state, avatar_bones) in remote_players.iter_mut()
                    {
                        if remote.player_id == received.player_id && player_host.0 == host_entity {
                            apply_iframe(&iframe, avatar_bones, &mut bone_transforms);
                            state.last_iframe = Some(iframe.clone());
                            break;
                        }
                    }

                    // If just spawned, apply to new entity.
                    if let Some(_entity) = player_entity {
                        // Will be applied next frame after AvatarBones populates.
                    }
                }
                TrackingUpdate::PFrame(pframe) => {
                    for (remote, player_host, state, avatar_bones) in remote_players.iter() {
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
    iframe: &unavi_server_service::TrackingIFrame,
    avatar_bones: &AvatarBones,
    bone_transforms: &mut Query<&mut Transform, With<BoneName>>,
) {
    // Apply bone rotations.
    for joint in &iframe.joints {
        if let Some(&bone_entity) = avatar_bones.get(&joint.id)
            && let Ok(mut transform) = bone_transforms.get_mut(bone_entity)
        {
            transform.rotation = Quat::from_array(joint.rotation);
        }
    }

    // Hip position is handled by the hips bone's global transform.
    if let Some(&hips_entity) = avatar_bones.get(&BoneName::Hips)
        && let Ok(mut transform) = bone_transforms.get_mut(hips_entity)
    {
        transform.translation = Vec3::from_array(iframe.translation);
    }
}

fn apply_pframe(
    pframe: &unavi_server_service::TrackingPFrame,
    last_iframe: &unavi_server_service::TrackingIFrame,
    avatar_bones: &AvatarBones,
    bone_transforms: &mut Query<&mut Transform, With<BoneName>>,
) {
    // Apply bone rotations (dequantize).
    for joint in &pframe.joints {
        if let Some(&bone_entity) = avatar_bones.get(&joint.id)
            && let Ok(mut transform) = bone_transforms.get_mut(bone_entity)
        {
            transform.rotation = Quat::from_array([
                joint.rotation[0] as f32 / PFRAME_ROTATION_SCALE,
                joint.rotation[1] as f32 / PFRAME_ROTATION_SCALE,
                joint.rotation[2] as f32 / PFRAME_ROTATION_SCALE,
                joint.rotation[3] as f32 / PFRAME_ROTATION_SCALE,
            ]);
        }
    }

    // Apply translation delta to hips.
    if let Some(&hips_entity) = avatar_bones.get(&BoneName::Hips)
        && let Ok(mut transform) = bone_transforms.get_mut(hips_entity)
    {
        let delta = Vec3::new(
            pframe.translation[0] as f32 / PFRAME_TRANSLATION_SCALE,
            pframe.translation[1] as f32 / PFRAME_TRANSLATION_SCALE,
            pframe.translation[2] as f32 / PFRAME_TRANSLATION_SCALE,
        );
        // Reconstruct translation from last iframe + delta.
        let last_translation = Vec3::from_array(last_iframe.translation);
        transform.translation = last_translation + delta;
    }
}
