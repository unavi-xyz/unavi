use std::{collections::HashMap, sync::Arc, time::Duration};

use bevy::prelude::*;
use bevy_vrm::BoneName;
use futures::SinkExt;
use scc::HashMap as SccHashMap;
use smallvec::SmallVec;
use tarpc::tokio_util::codec::{FramedWrite, LengthDelimitedCodec};
use unavi_player::{AvatarBones, LocalPlayer, PlayerEntities};
use unavi_server_service::{
    JointIFrame, JointPFrame, TRANSFORM_LENGTH_FIELD_LENGTH, TRANSFORM_MAX_FRAME_LENGTH,
    TrackingIFrame, TrackingPFrame,
};
use wtransport::SendStream;

use crate::space::{
    Space,
    networking::{NetworkingThread, thread::NetworkCommand},
};

use super::{JOINT_ROTATION_EPSILON, PFRAME_ROTATION_SCALE, PFRAME_TRANSLATION_SCALE};

const IFRAME_INTERVAL: Duration = Duration::from_secs(5);

use tokio::sync::Mutex as TokioMutex;

type FramedTransformWriter = FramedWrite<SendStream, LengthDelimitedCodec>;

#[derive(Clone)]
pub struct HostStreams {
    pub iframe_stream: Arc<TokioMutex<FramedTransformWriter>>,
    pub pframe_stream: Option<Arc<TokioMutex<SendStream>>>,
}

/// The interval at which data should be published to the space's server.
#[derive(Component, Default)]
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

#[derive(Debug)]
pub enum TransformResult {
    IFrame(TrackingIFrame),
    PFrame(TrackingPFrame),
}

#[allow(clippy::cast_possible_truncation)]
fn quantize_rotation(rot: Quat) -> [i16; 4] {
    [
        (rot.x * PFRAME_ROTATION_SCALE) as i16,
        (rot.y * PFRAME_ROTATION_SCALE) as i16,
        (rot.z * PFRAME_ROTATION_SCALE) as i16,
        (rot.w * PFRAME_ROTATION_SCALE) as i16,
    ]
}

#[allow(clippy::cast_possible_truncation)]
fn quantize_translation(delta: Vec3) -> [i16; 3] {
    [
        (delta.x * PFRAME_TRANSLATION_SCALE).clamp(f32::from(i16::MIN), f32::from(i16::MAX)) as i16,
        (delta.y * PFRAME_TRANSLATION_SCALE).clamp(f32::from(i16::MIN), f32::from(i16::MAX)) as i16,
        (delta.z * PFRAME_TRANSLATION_SCALE).clamp(f32::from(i16::MIN), f32::from(i16::MAX)) as i16,
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
    let is_iframe = current_time
        .checked_sub(state.last_iframe_time)
        .expect("value expected")
        >= IFRAME_INTERVAL;

    let hips_ent = *avatar_bones.get(&BoneName::Hips)?;
    let hips_global = global_transforms.get(hips_ent).ok()?;
    let (_, hips_rot, hips_pos) = hips_global.to_scale_rotation_translation();

    if is_iframe {
        state.current_iframe_id = state.current_iframe_id.wrapping_add(1);

        let mut joints = SmallVec::new();

        for (bone_name, &bone_ent) in avatar_bones.iter() {
            let Ok(transform) = bone_transforms.get(bone_ent) else {
                continue;
            };

            let should_include = state
                .iframe_joint_rot
                .get(bone_name)
                .is_none_or(|&last_rot| {
                    rotation_changed(transform.rotation, last_rot, JOINT_ROTATION_EPSILON)
                });

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

        let mut joints = SmallVec::new();

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

            let should_include = state.prev_joint_rot.get(bone_name).is_none_or(|&prev_rot| {
                rotation_changed(delta_rot, prev_rot, JOINT_ROTATION_EPSILON)
            });

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

pub async fn get_or_create_iframe_stream(
    streams: &Arc<SccHashMap<String, HostStreams>>,
    connect_url: &str,
    connection: &wtransport::Connection,
) -> anyhow::Result<()> {
    use unavi_server_service::from_client;

    if streams.contains_async(connect_url).await {
        return Ok(());
    }

    let mut stream = connection.open_uni().await?.await?;
    stream.set_priority(2); // Higher than p-frame streams.

    let header = from_client::StreamHeader::TransformIFrame;
    let header_bytes = bincode::encode_to_vec(&header, bincode::config::standard())?;
    stream.write_all(&header_bytes).await?;

    let framed = LengthDelimitedCodec::builder()
        .little_endian()
        .length_field_length(TRANSFORM_LENGTH_FIELD_LENGTH)
        .max_frame_length(TRANSFORM_MAX_FRAME_LENGTH)
        .new_write(stream);

    let _ = streams
        .insert_async(
            connect_url.to_string(),
            HostStreams {
                iframe_stream: Arc::new(TokioMutex::new(framed)),
                pframe_stream: None,
            },
        )
        .await;

    Ok(())
}

pub async fn send_iframe(
    streams: &Arc<SccHashMap<String, HostStreams>>,
    connect_url: &str,
    iframe: &TrackingIFrame,
) -> anyhow::Result<()> {
    let iframe_bytes = bincode::serde::encode_to_vec(iframe, bincode::config::standard())?;
    #[cfg(feature = "devtools-network")]
    let byte_count = iframe_bytes.len();

    let iframe_stream = streams
        .read_async(connect_url, |_, entry| Arc::clone(&entry.iframe_stream))
        .await
        .ok_or_else(|| anyhow::anyhow!("No iframe stream for {connect_url}"))?;

    iframe_stream.lock().await.send(iframe_bytes.into()).await?;

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

pub async fn send_pframe(
    streams: &Arc<SccHashMap<String, HostStreams>>,
    connect_url: &str,
    connection: &wtransport::Connection,
    pframe: &TrackingPFrame,
) -> anyhow::Result<()> {
    use unavi_server_service::from_client;

    // Take the old stream and reset it.
    let old_stream = streams
        .update_async(connect_url, |_, entry| entry.pframe_stream.take())
        .await
        .flatten();

    if let Some(stream_mutex) = old_stream {
        let mut stream = stream_mutex.lock().await;
        let _ = stream.reset(0u32.into());
    }

    let mut stream = connection.open_uni().await?.await?;
    stream.set_priority(1);

    let header = from_client::StreamHeader::TransformPFrame;
    let header_bytes = bincode::encode_to_vec(&header, bincode::config::standard())?;
    stream.write_all(&header_bytes).await?;

    let mut framed = LengthDelimitedCodec::builder()
        .little_endian()
        .length_field_length(TRANSFORM_LENGTH_FIELD_LENGTH)
        .max_frame_length(TRANSFORM_MAX_FRAME_LENGTH)
        .new_write(stream);

    let pframe_bytes = bincode::serde::encode_to_vec(pframe, bincode::config::standard())?;
    #[cfg(feature = "devtools-network")]
    let byte_count = pframe_bytes.len();

    framed.send(pframe_bytes.into()).await?;

    let send_stream = framed.into_inner();

    let _ = streams
        .update_async(connect_url, |_, entry| {
            entry.pframe_stream = Some(Arc::new(TokioMutex::new(send_stream)));
        })
        .await;

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
    networking: Res<NetworkingThread>,
    local_players: Query<&PlayerEntities, With<LocalPlayer>>,
    avatars: Query<&AvatarBones>,
    bone_transforms: Query<&Transform, With<BoneName>>,
    global_transforms: Query<&GlobalTransform>,
    mut spaces: Query<(&Space, &mut PublishInterval, &mut TransformPublishState)>,
) {
    let Ok(player_ents) = local_players.single() else {
        return;
    };

    let Ok(avatar_bones) = avatars.get(player_ents.avatar) else {
        return;
    };

    let now = time.elapsed();

    // Find the fastest tickrate among all spaces.
    let mut min_tickrate = None;
    let mut last_tick = None;

    for (_, interval, _) in spaces.iter() {
        match min_tickrate {
            None => {
                min_tickrate = Some(interval.tickrate);
                last_tick = Some(interval.last_tick);
            }
            Some(current_min) if interval.tickrate < current_min => {
                min_tickrate = Some(interval.tickrate);
                last_tick = Some(interval.last_tick);
            }
            _ => {}
        }
    }

    // Check if we should publish this tick.
    let (Some(tickrate), Some(last)) = (min_tickrate, last_tick) else {
        return;
    };

    if now.checked_sub(last).expect("value expected") < tickrate {
        return;
    }

    // Update all intervals.
    for (_, mut interval, _) in &mut spaces {
        interval.last_tick = now;
    }

    // Record transform from any space (they all share the same state).
    let Some((_, _, mut state)) = spaces.iter_mut().next() else {
        return;
    };

    let Some(update) = record_transforms(
        &mut state,
        now,
        avatar_bones,
        &bone_transforms,
        &global_transforms,
    ) else {
        return;
    };

    // Send single command to broadcast to all connections.
    let command = NetworkCommand::PublishTransform { update };

    if let Err(e) = networking.command_tx.send(command) {
        error!("Failed to send publish command: {e:?}");
    }
}
