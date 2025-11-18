use std::{collections::HashMap, sync::Arc};

use bevy::{prelude::*, tasks::futures_lite::StreamExt};
use bevy_vrm::BoneName;
use tarpc::tokio_util::codec::LengthDelimitedCodec;
use tokio::sync::{RwLock, watch};
use unavi_player::{AvatarBones, AvatarSpawner};
use unavi_server_service::{
    TRANSFORM_LENGTH_FIELD_LENGTH, TRANSFORM_MAX_FRAME_LENGTH, TrackingIFrame, TrackingPFrame,
    from_server::TransformMeta,
};
use wtransport::{RecvStream, StreamId};

use crate::space::{Host, HostTransformChannels, PlayerHost, RemotePlayer};

use super::{PFRAME_ROTATION_SCALE, PFRAME_TRANSLATION_SCALE};

#[derive(Clone, Debug)]
pub struct FinalTransform {
    pub hips_translation: Vec3,
    pub hips_rotation: Quat,
    pub joint_rotations: HashMap<BoneName, Quat>,
}

pub struct PlayerTransformState {
    pub tx: watch::Sender<FinalTransform>,
    pub rx: watch::Receiver<FinalTransform>,
    pub last_iframe: Option<TrackingIFrame>,
    pub current_iframe_id: u8,
    pub last_pframe_stream_id: Option<StreamId>,
}
pub type TransformChannels = Arc<RwLock<HashMap<u64, PlayerTransformState>>>;

pub async fn recv_iframe_stream(
    stream: RecvStream,
    player_channels: TransformChannels,
    #[cfg(feature = "devtools-network")] _connect_url: String,
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
                host: _connect_url.clone(),
                bytes: byte_count,
            });
            let _ = NETWORK_EVENTS.0.send(NetworkEvent::ValidTick {
                host: _connect_url.clone(),
            });
        }

        let final_transform = compute_final_transform_from_iframe(&iframe);

        let channels = player_channels.read().await;

        if let Some(state) = channels.get(&player_id) {
            let _ = state.tx.send(final_transform);
            drop(channels);
            let mut channels = player_channels.write().await;
            if let Some(state) = channels.get_mut(&player_id) {
                state.current_iframe_id = iframe.iframe_id;
                state.last_iframe = Some(iframe);
                state.last_pframe_stream_id = None;
            }
        } else {
            drop(channels);
            let mut channels = player_channels.write().await;
            let (tx, rx) = watch::channel(final_transform.clone());
            channels.insert(
                player_id,
                PlayerTransformState {
                    current_iframe_id: iframe.iframe_id,
                    last_iframe: Some(iframe.clone()),
                    last_pframe_stream_id: None,
                    rx,
                    tx,
                },
            );
        }
    }

    Ok(())
}

pub async fn recv_pframe_stream(
    stream: RecvStream,
    player_channels: TransformChannels,
    #[cfg(feature = "devtools-network")] _connect_url: String,
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

        let channels = player_channels.read().await;
        if let Some(state) = channels.get(&player_id) {
            if pframe.iframe_id != state.current_iframe_id {
                #[cfg(feature = "devtools-network")]
                {
                    use crate::devtools::events::{NETWORK_EVENTS, NetworkEvent};

                    let _ = NETWORK_EVENTS.0.send(NetworkEvent::DroppedFrame {
                        host: _connect_url.clone(),
                    });
                }
                return Ok(());
            }

            if let Some(last_stream_id) = state.last_pframe_stream_id
                && stream_id < last_stream_id
            {
                #[cfg(feature = "devtools-network")]
                {
                    use crate::devtools::events::{NETWORK_EVENTS, NetworkEvent};

                    let _ = NETWORK_EVENTS.0.send(NetworkEvent::DroppedFrame {
                        host: _connect_url.clone(),
                    });
                }
                return Ok(());
            }

            #[cfg(feature = "devtools-network")]
            {
                use crate::devtools::events::{NETWORK_EVENTS, NetworkEvent};

                let _ = NETWORK_EVENTS.0.send(NetworkEvent::Download {
                    host: _connect_url.clone(),
                    bytes: byte_count,
                });
                let _ = NETWORK_EVENTS.0.send(NetworkEvent::ValidTick {
                    host: _connect_url.clone(),
                });
            }

            if let Some(last_iframe) = &state.last_iframe {
                let final_transform = compute_final_transform_from_pframe(&pframe, last_iframe);
                let _ = state.tx.send(final_transform);
            }

            drop(channels);
            let mut channels = player_channels.write().await;
            if let Some(state) = channels.get_mut(&player_id) {
                state.last_pframe_stream_id = Some(stream_id);
            }
        }
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
        pframe.translation[0] as f32 / PFRAME_TRANSLATION_SCALE + last_iframe.translation[0],
        pframe.translation[1] as f32 / PFRAME_TRANSLATION_SCALE + last_iframe.translation[1],
        pframe.translation[2] as f32 / PFRAME_TRANSLATION_SCALE + last_iframe.translation[2],
    ]);

    let hips_rotation = (Quat::from_array([
        pframe.rotation[0] as f32 / PFRAME_ROTATION_SCALE,
        pframe.rotation[1] as f32 / PFRAME_ROTATION_SCALE,
        pframe.rotation[2] as f32 / PFRAME_ROTATION_SCALE,
        pframe.rotation[3] as f32 / PFRAME_ROTATION_SCALE,
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
                    joint.rotation[0] as f32 / PFRAME_ROTATION_SCALE,
                    joint.rotation[1] as f32 / PFRAME_ROTATION_SCALE,
                    joint.rotation[2] as f32 / PFRAME_ROTATION_SCALE,
                    joint.rotation[3] as f32 / PFRAME_ROTATION_SCALE,
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
    mut commands: Commands,
    mut pending_spawns: Local<HashMap<(Entity, u64), Entity>>,
    mut transforms: Query<&mut Transform>,
    remote_players: Query<(Entity, &RemotePlayer, &PlayerHost, &AvatarBones)>,
) {
    pending_spawns.retain(|_, entity| !remote_players.iter().any(|(e, ..)| e == *entity));

    for (host_entity, _, channel) in hosts.iter() {
        let Ok(mut channels) = channel.players.try_write() else {
            continue;
        };

        for (&player_id, state) in channels.iter_mut() {
            let Ok(true) = state.rx.has_changed() else {
                continue;
            };

            let final_transform = state.rx.borrow_and_update();

            let existing_entity = remote_players
                .iter()
                .find(|(_, remote, player_host, _)| {
                    remote.player_id == player_id && player_host.0 == host_entity
                })
                .map(|(e, ..)| e)
                .or_else(|| pending_spawns.get(&(host_entity, player_id)).copied());

            if existing_entity.is_none() {
                info!("Spawning remote player: {}", player_id);
                let avatar = AvatarSpawner::default().spawn(&mut commands, &asset_server);

                commands
                    .entity(avatar)
                    .insert((RemotePlayer { player_id }, PlayerHost(host_entity)));

                pending_spawns.insert((host_entity, player_id), avatar);
                continue;
            }

            for (player_entity, remote, player_host, avatar_bones) in remote_players.iter() {
                if remote.player_id != player_id || player_host.0 != host_entity {
                    continue;
                }

                if let Ok(mut transform) = transforms.get_mut(player_entity) {
                    transform.translation = final_transform.hips_translation;
                    transform.rotation = final_transform.hips_rotation;
                }

                if let Some(&hips_entity) = avatar_bones.get(&BoneName::Hips)
                    && let Ok(mut transform) = transforms.get_mut(hips_entity)
                {
                    transform.translation = Vec3::default();
                    transform.rotation = Quat::default();
                }

                for (bone_id, rotation) in &final_transform.joint_rotations {
                    if let Some(&bone_entity) = avatar_bones.get(bone_id)
                        && let Ok(mut transform) = transforms.get_mut(bone_entity)
                    {
                        transform.rotation = *rotation;
                    }
                }

                break;
            }
        }
    }
}
