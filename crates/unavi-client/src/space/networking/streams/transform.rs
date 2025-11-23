use std::{collections::HashMap, sync::Arc};

use bevy::{prelude::*, tasks::futures_lite::StreamExt};
use bevy_vrm::BoneName;
use scc::HashMap as SccHashMap;
use tarpc::tokio_util::codec::LengthDelimitedCodec;
use tokio::sync::watch;
use unavi_player::{AvatarBones, AvatarSpawner};
use unavi_server_service::{
    TRANSFORM_LENGTH_FIELD_LENGTH, TRANSFORM_MAX_FRAME_LENGTH, TrackingIFrame, TrackingPFrame,
    from_server::TransformMeta,
};
use wtransport::{RecvStream, StreamId};

use crate::space::{
    Host, HostTransformChannels, PlayerHost, RemotePlayer,
    networking::streams::publish::PublishInterval,
};

use super::{PFRAME_ROTATION_SCALE, PFRAME_TRANSLATION_SCALE};

#[derive(Component)]
pub struct RemotePlayerTickrate {
    pub tickrate_ms: u64,
}

#[derive(Component)]
pub struct PlayerSmoothingTargets {
    pub hips_translation: Vec3,
    pub hips_rotation: Quat,
    pub joint_rotations: HashMap<BoneName, Quat>,
}

#[derive(Clone, Debug)]
pub struct FinalTransform {
    pub hips_translation: Vec3,
    pub hips_rotation: Quat,
    pub joint_rotations: HashMap<BoneName, Quat>,
}

#[derive(Clone)]
pub struct PlayerTransformState {
    pub tx: watch::Sender<FinalTransform>,
    pub rx: watch::Receiver<FinalTransform>,
    pub last_iframe: Option<TrackingIFrame>,
    pub current_iframe_id: u8,
    pub last_pframe_stream_id: Option<StreamId>,
}
pub type TransformChannels = Arc<SccHashMap<u64, PlayerTransformState>>;

pub async fn recv_iframe_stream(
    stream: RecvStream,
    player_channels: TransformChannels,
    #[cfg(feature = "devtools-network")] connect_url: String,
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

    let player_id = meta.player;

    while let Some(bytes) = framed.next().await {
        let bytes_ref = bytes?;
        #[cfg(feature = "devtools-network")]
        let byte_count = bytes_ref.len();

        let (iframe, _) = bincode::serde::decode_from_slice::<TrackingIFrame, _>(
            &bytes_ref,
            bincode::config::standard(),
        )?;

        #[cfg(feature = "devtools-network")]
        {
            use crate::devtools::events::{NETWORK_EVENTS, NetworkEvent};

            let _ = NETWORK_EVENTS.0.send(NetworkEvent::Download {
                host: connect_url.clone(),
                bytes: byte_count,
            });
            let _ = NETWORK_EVENTS.0.send(NetworkEvent::ValidTick {
                host: connect_url.clone(),
            });
        }

        let final_transform = compute_final_transform_from_iframe(&iframe);

        // Try to update existing state.
        let updated = player_channels
            .update_async(&player_id, |_, state| {
                let _ = state.tx.send(final_transform.clone());
                state.current_iframe_id = iframe.iframe_id;
                state.last_iframe = Some(iframe.clone());
                state.last_pframe_stream_id = None;
            })
            .await
            .is_some();

        if !updated {
            // Insert new state.
            let (tx, rx) = watch::channel(final_transform.clone());
            let _ = player_channels
                .insert_async(
                    player_id,
                    PlayerTransformState {
                        current_iframe_id: iframe.iframe_id,
                        last_iframe: Some(iframe.clone()),
                        last_pframe_stream_id: None,
                        rx,
                        tx,
                    },
                )
                .await;
        }
    }

    Ok(())
}

pub async fn recv_pframe_stream(
    stream: RecvStream,
    player_channels: TransformChannels,
    #[cfg(feature = "devtools-network")] connect_url: String,
) -> anyhow::Result<()> {
    let stream_id = stream.id();

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

    let player_id = meta.player;

    if let Some(bytes) = framed.next().await {
        let bytes_ref = bytes?;
        #[cfg(feature = "devtools-network")]
        let byte_count = bytes_ref.len();

        let (pframe, _) = bincode::serde::decode_from_slice::<TrackingPFrame, _>(
            &bytes_ref,
            bincode::config::standard(),
        )?;

        player_channels
            .read_async(&player_id, |_, state| {
                if pframe.iframe_id != state.current_iframe_id {
                    #[cfg(feature = "devtools-network")]
                    {
                        use crate::devtools::events::{NETWORK_EVENTS, NetworkEvent};

                        let _ = NETWORK_EVENTS.0.send(NetworkEvent::DroppedFrame {
                            host: connect_url.clone(),
                        });
                    }
                    return Err(());
                }

                if let Some(last_stream_id) = state.last_pframe_stream_id
                    && stream_id < last_stream_id
                {
                    #[cfg(feature = "devtools-network")]
                    {
                        use crate::devtools::events::{NETWORK_EVENTS, NetworkEvent};

                        let _ = NETWORK_EVENTS.0.send(NetworkEvent::DroppedFrame {
                            host: connect_url.clone(),
                        });
                    }
                    return Err(());
                }

                #[cfg(feature = "devtools-network")]
                {
                    use crate::devtools::events::{NETWORK_EVENTS, NetworkEvent};

                    let _ = NETWORK_EVENTS.0.send(NetworkEvent::Download {
                        host: connect_url.clone(),
                        bytes: byte_count,
                    });
                    let _ = NETWORK_EVENTS.0.send(NetworkEvent::ValidTick {
                        host: connect_url.clone(),
                    });
                }

                if let Some(last_iframe) = &state.last_iframe {
                    let final_transform = compute_final_transform_from_pframe(&pframe, last_iframe);
                    let _ = state.tx.send(final_transform);
                }

                Ok(())
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Player not found"))?
            .map_err(|()| anyhow::anyhow!("Frame dropped"))?;

        let _ = player_channels
            .update_async(&player_id, |_, state| {
                state.last_pframe_stream_id = Some(stream_id);
            })
            .await;
    }

    Ok(())
}

fn compute_final_transform_from_iframe(iframe: &TrackingIFrame) -> FinalTransform {
    let mut joint_rotations = HashMap::with_capacity(iframe.joints.len());

    for joint in &iframe.joints {
        if joint.id == BoneName::Hips {
            continue;
        }

        joint_rotations.insert(joint.id, Quat::from_array(joint.rotation));
    }

    FinalTransform {
        hips_translation: Vec3::from_array(iframe.translation),
        hips_rotation: Quat::from_array(iframe.rotation),
        joint_rotations,
    }
}

fn compute_final_transform_from_pframe(
    pframe: &TrackingPFrame,
    last_iframe: &TrackingIFrame,
) -> FinalTransform {
    let hips_translation = Vec3::from_array([
        f32::from(pframe.translation[0]) / PFRAME_TRANSLATION_SCALE + last_iframe.translation[0],
        f32::from(pframe.translation[1]) / PFRAME_TRANSLATION_SCALE + last_iframe.translation[1],
        f32::from(pframe.translation[2]) / PFRAME_TRANSLATION_SCALE + last_iframe.translation[2],
    ]);

    let hips_rotation = (Quat::from_array([
        f32::from(pframe.rotation[0]) / PFRAME_ROTATION_SCALE,
        f32::from(pframe.rotation[1]) / PFRAME_ROTATION_SCALE,
        f32::from(pframe.rotation[2]) / PFRAME_ROTATION_SCALE,
        f32::from(pframe.rotation[3]) / PFRAME_ROTATION_SCALE,
    ]) * Quat::from_array(last_iframe.rotation))
    .normalize();

    let mut joint_rotations = HashMap::with_capacity(pframe.joints.len());

    for joint in &pframe.joints {
        if joint.id == BoneName::Hips {
            continue;
        }

        if let Some(iframe_joint) = last_iframe.joints.iter().find(|j| j.id == joint.id) {
            joint_rotations.insert(
                joint.id,
                (Quat::from_array([
                    f32::from(joint.rotation[0]) / PFRAME_ROTATION_SCALE,
                    f32::from(joint.rotation[1]) / PFRAME_ROTATION_SCALE,
                    f32::from(joint.rotation[2]) / PFRAME_ROTATION_SCALE,
                    f32::from(joint.rotation[3]) / PFRAME_ROTATION_SCALE,
                ]) * Quat::from_array(iframe_joint.rotation))
                .normalize(),
            );
        }
    }

    FinalTransform {
        hips_translation,
        hips_rotation,
        joint_rotations,
    }
}

pub fn apply_player_transforms(
    asset_server: Res<AssetServer>,
    hosts: Query<(Entity, &Host, &HostTransformChannels)>,
    spaces: Query<&PublishInterval>,
    mut commands: Commands,
    mut pending_spawns: Local<HashMap<(Entity, u64), Entity>>,
    remote_players: Query<(Entity, &RemotePlayer, &PlayerHost, &AvatarBones)>,
    mut smoothing_query: Query<&mut PlayerSmoothingTargets>,
) {
    pending_spawns.retain(|_, entity| !remote_players.iter().any(|(e, ..)| e == *entity));

    for (host_entity, _, channel) in hosts.iter() {
        let server_tickrate_ms = spaces
            .iter()
            .map(|interval| interval.tickrate.as_millis() as u64)
            .next()
            .unwrap_or(50);

        channel.players.iter_sync(|player_id, state| {
            let Ok(true) = state.rx.has_changed() else {
                return false;
            };

            let final_transform = state.rx.borrow().clone();

            let existing_entity = remote_players
                .iter()
                .find(|(_, remote, player_host, _)| {
                    remote.player_id == *player_id && player_host.0 == host_entity
                })
                .map(|(e, ..)| e)
                .or_else(|| pending_spawns.get(&(host_entity, *player_id)).copied());

            if existing_entity.is_none() {
                info!("Spawning remote player: {}", player_id);
                let avatar = AvatarSpawner::default().spawn(&mut commands, &asset_server);

                commands.entity(avatar).insert((
                    RemotePlayer {
                        player_id: *player_id,
                    },
                    PlayerHost(host_entity),
                    RemotePlayerTickrate {
                        tickrate_ms: server_tickrate_ms,
                    },
                    PlayerSmoothingTargets {
                        hips_translation: final_transform.hips_translation,
                        hips_rotation: final_transform.hips_rotation,
                        joint_rotations: final_transform.joint_rotations,
                    },
                ));

                pending_spawns.insert((host_entity, *player_id), avatar);
                return false;
            }

            for (player_entity, remote, player_host, _) in remote_players.iter() {
                if remote.player_id != *player_id || player_host.0 != host_entity {
                    continue;
                }

                // Update smoothing targets or add component if missing.
                if let Ok(mut smoothing) = smoothing_query.get_mut(player_entity) {
                    smoothing.hips_translation = final_transform.hips_translation;
                    smoothing.hips_rotation = final_transform.hips_rotation;
                    smoothing
                        .joint_rotations
                        .clone_from(&final_transform.joint_rotations);
                } else {
                    commands
                        .entity(player_entity)
                        .insert(PlayerSmoothingTargets {
                            hips_translation: final_transform.hips_translation,
                            hips_rotation: final_transform.hips_rotation,
                            joint_rotations: final_transform.joint_rotations,
                        });
                }

                break;
            }

            false // Never early-exit
        });
    }
}

const SMOOTHING_FACTOR: f32 = 0.8;

pub fn smooth_network_transforms(
    time: Res<Time>,
    mut remote_players: Query<(
        Entity,
        &RemotePlayerTickrate,
        &mut PlayerSmoothingTargets,
        &AvatarBones,
    )>,
    mut transforms: Query<&mut Transform>,
) {
    let delta = time.delta_secs();

    for (player_entity, tickrate, smoothing, avatar_bones) in &mut remote_players {
        // Calculate adaptive smoothing speed based on tickrate.
        // Higher tickrate = faster smoothing (more responsive).
        let tickrate_hz = 1000.0 / tickrate.tickrate_ms as f32;
        let smooth_speed = tickrate_hz * SMOOTHING_FACTOR;
        let t = (delta * smooth_speed).min(1.0);

        // Smooth root transform.
        if let Ok(mut transform) = transforms.get_mut(player_entity) {
            transform.translation = transform.translation.lerp(smoothing.hips_translation, t);
            transform.rotation = transform.rotation.slerp(smoothing.hips_rotation, t);
        }

        // Keep hips at origin.
        if let Some(&hips_entity) = avatar_bones.get(&BoneName::Hips)
            && let Ok(mut transform) = transforms.get_mut(hips_entity)
        {
            transform.translation = Vec3::default();
            transform.rotation = Quat::default();
        }

        // Smooth joint rotations.
        for (bone_id, target_rotation) in &smoothing.joint_rotations {
            if let Some(&bone_entity) = avatar_bones.get(bone_id)
                && let Ok(mut transform) = transforms.get_mut(bone_entity)
            {
                transform.rotation = transform.rotation.slerp(*target_rotation, t);
            }
        }
    }
}
