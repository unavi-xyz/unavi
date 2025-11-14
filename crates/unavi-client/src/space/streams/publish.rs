use std::time::Duration;

use bevy::{prelude::*, tasks::TaskPool};
use bevy_vrm::BoneName;
use futures::SinkExt;
use tarpc::tokio_util::codec::LengthDelimitedCodec;
use unavi_player::{AvatarBones, LocalPlayer, PlayerEntities};
use unavi_server_service::{
    JointIFrame, JointPFrame, TRANSFORM_LENGTH_FIELD_LENGTH, TRANSFORM_MAX_FRAME_LENGTH,
    TrackingIFrame, TrackingPFrame, TrackingUpdate,
};

use crate::space::{Space, connect::HostConnections, connect_info::ConnectInfo};

// 1 tick = 20hz (default server value)
const IFRAME_INTERVAL_TICKS: u32 = 30;

/// The interval at which data should be published to the space's server.
#[derive(Component)]
pub struct PublishInterval {
    pub last_tick: Duration,
    pub tickrate: Duration,
}

#[derive(Component, Default)]
pub struct TransformPublishState {
    tick_count: u32,
    last_hips_pos: [f32; 3],
}

fn quantize_rotation(rot: Quat) -> [i16; 4] {
    const SCALE: f32 = i16::MAX as f32;
    [
        (rot.x * SCALE) as i16,
        (rot.y * SCALE) as i16,
        (rot.z * SCALE) as i16,
        (rot.w * SCALE) as i16,
    ]
}

fn quantize_position_delta(delta: Vec3) -> [i16; 3] {
    const SCALE: f32 = 1000.0;
    [
        (delta.x * SCALE).clamp(i16::MIN as f32, i16::MAX as f32) as i16,
        (delta.y * SCALE).clamp(i16::MIN as f32, i16::MAX as f32) as i16,
        (delta.z * SCALE).clamp(i16::MIN as f32, i16::MAX as f32) as i16,
    ]
}

fn record_transforms(
    state: &mut TransformPublishState,
    avatar_bones: &AvatarBones,
    bone_transforms: &Query<&Transform, With<BoneName>>,
    global_transforms: &Query<&GlobalTransform>,
) -> Option<TrackingUpdate> {
    let is_iframe = state.tick_count.is_multiple_of(IFRAME_INTERVAL_TICKS);
    state.tick_count += 1;

    let hips_ent = *avatar_bones.get(&BoneName::Hips)?;
    let hips_global = global_transforms.get(hips_ent).ok()?;
    let hips_pos = hips_global.translation();

    if is_iframe {
        let mut joints = Vec::new();

        for (bone_name, &bone_ent) in avatar_bones.iter() {
            let Ok(transform) = bone_transforms.get(bone_ent) else {
                continue;
            };

            joints.push(JointIFrame {
                id: *bone_name,
                rot: [
                    transform.rotation.x,
                    transform.rotation.y,
                    transform.rotation.z,
                    transform.rotation.w,
                ],
            });
        }

        let iframe = TrackingIFrame {
            pos: [hips_pos.x, hips_pos.y, hips_pos.z],
            joints,
        };

        state.last_hips_pos = iframe.pos;

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

            let rot = quantize_rotation(transform.rotation);

            joints.push(JointPFrame {
                id: *bone_name,
                rot,
            });
        }

        let pframe = TrackingPFrame {
            pos: quantize_position_delta(delta_pos),
            joints,
        };

        Some(TrackingUpdate::PFrame(pframe))
    }
}

pub fn publish_transform_data(
    time: Res<Time>,
    host_connections: Res<HostConnections>,
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

    for (_, connect_info, mut interval, mut state) in spaces.iter_mut() {
        if now - interval.last_tick < interval.tickrate {
            continue;
        }

        interval.last_tick = now;

        let Some(update) = record_transforms(
            &mut state,
            avatar_bones,
            &bone_transforms,
            &global_transforms,
        ) else {
            continue;
        };

        let connect_url = connect_info.connect_url.to_string();
        let connections_arc = host_connections.0.clone();

        let pool = TaskPool::get_thread_executor();
        pool.spawn(async move {
            let guard = connections_arc.read().await;
            let Some(host) = guard.get(&connect_url) else {
                return;
            };
            let connection = host.connection.clone();
            drop(guard);

            if let Err(e) = send_transform_update(&connection, update).await {
                error!("Failed to send transform update: {e:?}");
            }
        })
        .detach();
    }
}

async fn send_transform_update(
    connection: &wtransport::Connection,
    update: TrackingUpdate,
) -> anyhow::Result<()> {
    use unavi_server_service::from_client;

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

    let update_bytes = bincode::serde::encode_to_vec(&update, bincode::config::standard())?;
    framed.send(update_bytes.into()).await?;

    framed.into_inner().finish().await?;

    Ok(())
}
