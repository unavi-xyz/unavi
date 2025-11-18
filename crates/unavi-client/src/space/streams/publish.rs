use std::{collections::HashMap, sync::Arc, time::Duration};

use bevy::{prelude::*, tasks::TaskPool};
use bevy_vrm::BoneName;
use futures::SinkExt;
use tarpc::tokio_util::codec::{FramedWrite, LengthDelimitedCodec};
use tokio::sync::Mutex;
use unavi_player::{AvatarBones, LocalPlayer, PlayerEntities};
use unavi_server_service::{
    JointIFrame, JointPFrame, TRANSFORM_LENGTH_FIELD_LENGTH, TRANSFORM_MAX_FRAME_LENGTH,
    TrackingIFrame, TrackingPFrame,
};
use wtransport::SendStream;

use crate::space::{Space, connect::HostConnections, connect_info::ConnectInfo};

use super::{JOINT_ROTATION_EPSILON, PFRAME_ROTATION_SCALE, PFRAME_TRANSLATION_SCALE};

const IFRAME_INTERVAL: Duration = Duration::from_secs(5);

type FramedTransformWriter = FramedWrite<SendStream, LengthDelimitedCodec>;

pub struct HostStreams {
    pub iframe_stream: FramedTransformWriter,
    pub pframe_stream: Option<SendStream>,
}

#[derive(Resource, Default, Clone)]
pub struct HostTransformStreams(pub Arc<Mutex<HashMap<String, HostStreams>>>);

/// The interval at which data should be published to the space's server.
#[derive(Component)]
pub struct PublishInterval {
    pub last_tick: Duration,
    pub tickrate: Duration,
}

#[derive(Component, Clone, Default)]
pub struct TransformPublishState {
    last_iframe_time: Duration,
    current_iframe_id: u8,

    iframe_hips_pos: Vec3,
    iframe_hips_rot: Quat,

    iframe_joint_rot: HashMap<BoneName, Quat>,
    prev_joint_rot: HashMap<BoneName, Quat>,
}

enum TransformResult {
    IFrame(TrackingIFrame),
    PFrame(TrackingPFrame),
}

fn quantize_rotation(rot: Quat) -> [i16; 4] {
    [
        (rot.x * PFRAME_ROTATION_SCALE) as i16,
        (rot.y * PFRAME_ROTATION_SCALE) as i16,
        (rot.z * PFRAME_ROTATION_SCALE) as i16,
        (rot.w * PFRAME_ROTATION_SCALE) as i16,
    ]
}

fn quantize_translation(delta: Vec3) -> [i16; 3] {
    [
        (delta.x * PFRAME_TRANSLATION_SCALE).clamp(i16::MIN as f32, i16::MAX as f32) as i16,
        (delta.y * PFRAME_TRANSLATION_SCALE).clamp(i16::MIN as f32, i16::MAX as f32) as i16,
        (delta.z * PFRAME_TRANSLATION_SCALE).clamp(i16::MIN as f32, i16::MAX as f32) as i16,
    ]
}

fn rotation_changed(current: Quat, last: Quat, epsilon: f32) -> bool {
    let dot = current.dot(last).abs();
    dot < (1.0 - epsilon)
}

fn record_transforms(
    state: &mut TransformPublishState,
    current_time: Duration,
    avatar_bones: &AvatarBones,
    bone_transforms: &Query<&Transform, With<BoneName>>,
    global_transforms: &Query<&GlobalTransform>,
) -> Option<TransformResult> {
    let is_iframe = current_time - state.last_iframe_time >= IFRAME_INTERVAL;

    let hips_ent = *avatar_bones.get(&BoneName::Hips)?;
    let hips_global = global_transforms.get(hips_ent).ok()?;
    let (_, hips_rot, hips_pos) = hips_global.to_scale_rotation_translation();

    if is_iframe {
        state.current_iframe_id = state.current_iframe_id.wrapping_add(1);

        let mut joints = Vec::new();

        for (bone_name, &bone_ent) in avatar_bones.iter() {
            let Ok(transform) = bone_transforms.get(bone_ent) else {
                continue;
            };

            let should_include = state
                .iframe_joint_rot
                .get(bone_name)
                .map(|&last_rot| {
                    rotation_changed(transform.rotation, last_rot, JOINT_ROTATION_EPSILON)
                })
                .unwrap_or(true);

            if !should_include {
                continue;
            }

            joints.push(JointIFrame {
                id: *bone_name,
                rotation: transform.rotation.to_array(),
            });

            state
                .iframe_joint_rot
                .insert(*bone_name, transform.rotation);
            state.prev_joint_rot.insert(*bone_name, Quat::default());
        }

        let iframe = TrackingIFrame {
            iframe_id: state.current_iframe_id,
            translation: hips_pos.to_array(),
            rotation: hips_rot.to_array(),
            joints,
        };

        state.last_iframe_time = current_time;
        state.iframe_hips_pos = hips_pos;
        state.iframe_hips_rot = hips_rot;

        Some(TransformResult::IFrame(iframe))
    } else {
        let delta_hips_pos = hips_pos - state.iframe_hips_pos;
        let delta_hips_rot = (hips_rot * state.iframe_hips_rot.inverse()).normalize();

        let mut joints = Vec::new();

        for (bone_name, &bone_ent) in avatar_bones.iter() {
            let Ok(transform) = bone_transforms.get(bone_ent) else {
                continue;
            };

            let Some(delta_rot) = state
                .iframe_joint_rot
                .get(bone_name)
                .map(|&base_rot| (transform.rotation * base_rot.inverse()).normalize())
            else {
                continue;
            };

            let should_include = state
                .prev_joint_rot
                .get(bone_name)
                .map(|&prev_rot| rotation_changed(delta_rot, prev_rot, JOINT_ROTATION_EPSILON))
                .unwrap_or(true);

            if !should_include {
                continue;
            }

            joints.push(JointPFrame {
                id: *bone_name,
                rotation: quantize_rotation(delta_rot),
            });

            state.prev_joint_rot.insert(*bone_name, delta_rot);
        }

        let pframe = TrackingPFrame {
            iframe_id: state.current_iframe_id,
            translation: quantize_translation(delta_hips_pos),
            rotation: quantize_rotation(delta_hips_rot),
            joints,
        };

        Some(TransformResult::PFrame(pframe))
    }
}

async fn get_or_create_iframe_stream(
    streams: &Arc<Mutex<HashMap<String, HostStreams>>>,
    connect_url: &str,
    connection: &wtransport::Connection,
) -> anyhow::Result<()> {
    use unavi_server_service::from_client;

    let mut guard = streams.lock().await;

    if guard.contains_key(connect_url) {
        return Ok(());
    }

    let stream = connection.open_uni().await?.await?;

    let mut framed = LengthDelimitedCodec::builder()
        .little_endian()
        .length_field_length(TRANSFORM_LENGTH_FIELD_LENGTH)
        .max_frame_length(TRANSFORM_MAX_FRAME_LENGTH)
        .new_write(stream);

    let header = from_client::StreamHeader::TransformIFrame;
    let header_bytes = bincode::encode_to_vec(&header, bincode::config::standard())?;
    framed.send(header_bytes.into()).await?;

    guard.insert(
        connect_url.to_string(),
        HostStreams {
            iframe_stream: framed,
            pframe_stream: None,
        },
    );

    Ok(())
}

async fn send_iframe(
    streams: &Arc<Mutex<HashMap<String, HostStreams>>>,
    connect_url: &str,
    iframe: &TrackingIFrame,
) -> anyhow::Result<()> {
    let mut guard = streams.lock().await;
    let Some(host_streams) = guard.get_mut(connect_url) else {
        anyhow::bail!("No iframe stream for {}", connect_url);
    };

    let iframe_bytes = bincode::serde::encode_to_vec(iframe, bincode::config::standard())?;
    #[cfg(feature = "devtools-network")]
    let byte_count = iframe_bytes.len();
    host_streams.iframe_stream.send(iframe_bytes.into()).await?;

    #[cfg(feature = "devtools-network")]
    {
        use crate::devtools::events::{NETWORK_EVENTS, NetworkEvent};

        let _ = NETWORK_EVENTS.0.send(NetworkEvent::Upload {
            host: connect_url.to_string(),
            bytes: byte_count,
            is_iframe: true,
        });
    }

    Ok(())
}

async fn send_pframe(
    streams: &Arc<Mutex<HashMap<String, HostStreams>>>,
    connect_url: &str,
    connection: &wtransport::Connection,
    pframe: &TrackingPFrame,
) -> anyhow::Result<()> {
    use unavi_server_service::from_client;

    let mut guard = streams.lock().await;
    let Some(host_streams) = guard.get_mut(connect_url) else {
        anyhow::bail!("No stream state for {}", connect_url);
    };

    if let Some(mut old_stream) = host_streams.pframe_stream.take() {
        let _ = old_stream.reset(0u32.into());
    }

    drop(guard);

    let stream = connection.open_uni().await?.await?;
    stream.set_priority(1);

    let mut framed = LengthDelimitedCodec::builder()
        .little_endian()
        .length_field_length(TRANSFORM_LENGTH_FIELD_LENGTH)
        .max_frame_length(TRANSFORM_MAX_FRAME_LENGTH)
        .new_write(stream);

    let header = from_client::StreamHeader::TransformPFrame;
    let header_bytes = bincode::encode_to_vec(&header, bincode::config::standard())?;
    framed.send(header_bytes.into()).await?;

    let pframe_bytes = bincode::serde::encode_to_vec(pframe, bincode::config::standard())?;
    #[cfg(feature = "devtools-network")]
    let byte_count = pframe_bytes.len();
    framed.send(pframe_bytes.into()).await?;

    let send_stream = framed.into_inner();

    let mut guard = streams.lock().await;
    if let Some(host_streams) = guard.get_mut(connect_url) {
        host_streams.pframe_stream = Some(send_stream);
    }

    #[cfg(feature = "devtools-network")]
    {
        use crate::devtools::events::{NETWORK_EVENTS, NetworkEvent};

        let _ = NETWORK_EVENTS.0.send(NetworkEvent::Upload {
            host: connect_url.to_string(),
            bytes: byte_count,
            is_iframe: false,
        });
    }

    Ok(())
}

pub fn publish_transform_data(
    time: Res<Time>,
    host_connections: Res<HostConnections>,
    transform_streams: Res<HostTransformStreams>,
    local_players: Query<&PlayerEntities, With<LocalPlayer>>,
    avatars: Query<&AvatarBones>,
    bone_transforms: Query<&Transform, With<BoneName>>,
    global_transforms: Query<&GlobalTransform>,
    mut spaces: Query<(
        &Space,
        &ConnectInfo,
        &mut PublishInterval,
        &mut TransformPublishState,
    )>,
) {
    let Ok(player_ents) = local_players.single() else {
        return;
    };

    let Ok(avatar_bones) = avatars.get(player_ents.avatar) else {
        return;
    };

    let now = time.elapsed();

    let mut hosts_to_publish: HashMap<String, (TransformPublishState, TransformResult)> =
        HashMap::new();

    for (_, connect_info, mut interval, mut state) in spaces.iter_mut() {
        if now - interval.last_tick < interval.tickrate {
            continue;
        }

        interval.last_tick = now;

        let Some(update) = record_transforms(
            &mut state,
            now,
            avatar_bones,
            &bone_transforms,
            &global_transforms,
        ) else {
            continue;
        };

        let connect_url = connect_info.connect_url.to_string();

        hosts_to_publish
            .entry(connect_url)
            .or_insert_with(|| (state.clone(), update));
    }

    for (connect_url, (_, update)) in hosts_to_publish {
        let connections_arc = host_connections.0.clone();
        let streams_arc = transform_streams.0.clone();

        let pool = TaskPool::get_thread_executor();
        pool.spawn(async move {
            let guard = connections_arc.read().await;
            let Some(host) = guard.get(&connect_url) else {
                return;
            };
            let connection = host.connection.clone();
            drop(guard);

            if let Err(e) =
                get_or_create_iframe_stream(&streams_arc, &connect_url, &connection).await
            {
                error!("Failed to create iframe stream: {e:?}");
                return;
            }

            let result = match update {
                TransformResult::IFrame(ref iframe) => {
                    send_iframe(&streams_arc, &connect_url, iframe).await
                }
                TransformResult::PFrame(ref pframe) => {
                    send_pframe(&streams_arc, &connect_url, &connection, pframe).await
                }
            };

            if let Err(e) = result {
                error!("Failed to send transform update: {e:?}");
                streams_arc.lock().await.remove(&connect_url);
            }
        })
        .detach();
    }
}
