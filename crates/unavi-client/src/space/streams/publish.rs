use std::{collections::HashMap, sync::Arc, time::Duration};

use bevy::{prelude::*, tasks::TaskPool};
use bevy_vrm::BoneName;
use futures::SinkExt;
use tarpc::tokio_util::codec::{FramedWrite, LengthDelimitedCodec};
use tokio::sync::Mutex;
use unavi_player::{AvatarBones, LocalPlayer, PlayerEntities};
use unavi_server_service::{
    JointIFrame, JointPFrame, TRANSFORM_LENGTH_FIELD_LENGTH, TRANSFORM_MAX_FRAME_LENGTH,
    TrackingIFrame, TrackingPFrame, TrackingUpdate,
};
use wtransport::SendStream;

use crate::space::{Space, connect::HostConnections, connect_info::ConnectInfo};

use super::{JOINT_ROTATION_EPSILON, PFRAME_ROTATION_SCALE, PFRAME_TRANSLATION_SCALE};

const IFRAME_INTERVAL: Duration = Duration::from_secs(5);

type FramedTransformWriter = FramedWrite<SendStream, LengthDelimitedCodec>;

/// Cached transform streams, one per host connection.
#[derive(Resource, Default, Clone)]
pub struct HostTransformStreams(pub Arc<Mutex<HashMap<String, FramedTransformWriter>>>);

/// The interval at which data should be published to the space's server.
#[derive(Component)]
pub struct PublishInterval {
    pub last_tick: Duration,
    pub tickrate: Duration,
}

#[derive(Component, Clone)]
pub struct TransformPublishState {
    last_iframe_time: Duration,
    last_hips_pos: [f32; 3],
    last_hips_rot: [f32; 4],
    last_joint_rotations: HashMap<BoneName, Quat>,
}

impl Default for TransformPublishState {
    fn default() -> Self {
        Self {
            last_iframe_time: Duration::ZERO,
            last_hips_pos: [0.0, 0.0, 0.0],
            last_hips_rot: [0.0, 0.0, 0.0, 1.0],
            last_joint_rotations: HashMap::new(),
        }
    }
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
) -> Option<TrackingUpdate> {
    let is_iframe = current_time - state.last_iframe_time >= IFRAME_INTERVAL;

    let hips_ent = *avatar_bones.get(&BoneName::Hips)?;
    let hips_global = global_transforms.get(hips_ent).ok()?;
    let (_, hips_rot, hips_pos) = hips_global.to_scale_rotation_translation();

    if is_iframe {
        let mut joints = Vec::new();

        for (bone_name, &bone_ent) in avatar_bones.iter() {
            let Ok(transform) = bone_transforms.get(bone_ent) else {
                continue;
            };

            // Only include joint if rotation changed from last published value.
            let should_include = state
                .last_joint_rotations
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
                rotation: [
                    transform.rotation.x,
                    transform.rotation.y,
                    transform.rotation.z,
                    transform.rotation.w,
                ],
            });

            state
                .last_joint_rotations
                .insert(*bone_name, transform.rotation);
        }

        let iframe = TrackingIFrame {
            translation: [hips_pos.x, hips_pos.y, hips_pos.z],
            rotation: [hips_rot.x, hips_rot.y, hips_rot.z, hips_rot.w],
            joints,
        };

        state.last_hips_pos = iframe.translation;
        state.last_hips_rot = iframe.rotation;
        state.last_iframe_time = current_time;

        Some(TrackingUpdate::IFrame(iframe))
    } else {
        let delta_pos = Vec3::new(
            hips_pos.x - state.last_hips_pos[0],
            hips_pos.y - state.last_hips_pos[1],
            hips_pos.z - state.last_hips_pos[2],
        );

        let mut joints = Vec::new();

        for (bone_name, &bone_ent) in avatar_bones.iter() {
            let Ok(transform) = bone_transforms.get(bone_ent) else {
                continue;
            };

            // Only include joint if rotation changed from last published value.
            let should_include = state
                .last_joint_rotations
                .get(bone_name)
                .map(|&last_rot| {
                    rotation_changed(transform.rotation, last_rot, JOINT_ROTATION_EPSILON)
                })
                .unwrap_or(true);

            if !should_include {
                continue;
            }

            let rotation = quantize_rotation(transform.rotation);

            joints.push(JointPFrame {
                id: *bone_name,
                rotation,
            });

            state
                .last_joint_rotations
                .insert(*bone_name, transform.rotation);
        }

        let pframe = TrackingPFrame {
            translation: quantize_translation(delta_pos),
            rotation: quantize_rotation(hips_rot),
            joints,
        };

        Some(TrackingUpdate::PFrame(pframe))
    }
}

async fn get_or_create_transform_stream(
    streams: &Arc<Mutex<HashMap<String, FramedTransformWriter>>>,
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

    let header = from_client::StreamHeader::Transform;
    let header_bytes = bincode::encode_to_vec(&header, bincode::config::standard())?;
    framed.send(header_bytes.into()).await?;

    let meta = from_client::TransformMeta { placeholder: true };
    let meta_bytes = bincode::encode_to_vec(&meta, bincode::config::standard())?;
    framed.send(meta_bytes.into()).await?;

    guard.insert(connect_url.to_string(), framed);

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

    // Group spaces by host to send transforms once per host.
    let mut hosts_to_publish: HashMap<String, (TransformPublishState, TrackingUpdate)> =
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

        // Only publish once per host, using the first space's state.
        hosts_to_publish
            .entry(connect_url)
            .or_insert_with(|| (state.clone(), update));
    }

    // Publish transforms to each unique host.
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
                get_or_create_transform_stream(&streams_arc, &connect_url, &connection).await
            {
                error!("Failed to create transform stream: {e:?}");
                return;
            }

            let mut streams_guard = streams_arc.lock().await;
            let Some(framed) = streams_guard.get_mut(&connect_url) else {
                return;
            };

            let update_bytes =
                match bincode::serde::encode_to_vec(&update, bincode::config::standard()) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        error!("Failed to encode transform update: {e:?}");
                        return;
                    }
                };

            if let Err(e) = framed.send(update_bytes.into()).await {
                error!("Failed to send transform update: {e:?}");
                streams_guard.remove(&connect_url);
            }
        })
        .detach();
    }
}
