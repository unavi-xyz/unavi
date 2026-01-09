use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use bevy::prelude::*;
use bevy_vrm::BoneName;
use unavi_player::{AvatarBones, LocalPlayer, PlayerEntities};

use crate::networking::thread::{
    NetworkCommand, NetworkingThread,
    space::{BonePose, IFrameTransform, PFrameTransform, PlayerIFrame, PlayerPFrame},
};

const PUBLISH_HZ: u64 = 20;
const IFRAME_FREQ: u64 = PUBLISH_HZ * 3;
const PUBLISH_TICKRATE: Duration = Duration::from_millis(1000 / PUBLISH_HZ);

/// Set of bones to include in pose updates.
/// Empty = just root (desktop mode, no VR tracking).
#[derive(Resource, Default, Clone)]
pub struct TrackedBones(pub HashSet<BoneName>);

/// Stores the last I-frame positions for P-frame delta encoding.
#[derive(Default)]
pub(super) struct IFrameBaseline {
    root: Vec3,
    bones: HashMap<BoneName, Vec3>,
}

pub(super) fn publish_player_transforms(
    nt: Res<NetworkingThread>,
    players: Query<(&GlobalTransform, &PlayerEntities), With<LocalPlayer>>,
    avatar_bones: Query<&AvatarBones>,
    bone_transforms: Query<&GlobalTransform, With<BoneName>>,
    tracked_bones: Res<TrackedBones>,
    time: Res<Time>,
    mut last: Local<Duration>,
    mut count: Local<u64>,
    mut baseline: Local<IFrameBaseline>,
) {
    let now = time.elapsed();
    if now.saturating_sub(*last) < PUBLISH_TICKRATE {
        return;
    }
    *last = now;
    *count += 1;

    let Ok((root_tr, entities)) = players.single() else {
        return;
    };

    let root_pos = root_tr.translation();
    let root_rot = root_tr.to_scale_rotation_translation().1;

    // Collect bone transforms.
    let bones_map = avatar_bones.get(entities.avatar).ok();

    let is_iframe = (*count).is_multiple_of(IFRAME_FREQ);

    if is_iframe {
        // Build I-frame with absolute positions.
        let mut bones = Vec::with_capacity(tracked_bones.0.len());

        if let Some(avatar_bones) = bones_map {
            for bone_name in &tracked_bones.0 {
                if let Some(&bone_entity) = avatar_bones.get(bone_name)
                    && let Ok(bone_tr) = bone_transforms.get(bone_entity)
                {
                    let pos = bone_tr.translation();
                    let rot = bone_tr.to_scale_rotation_translation().1;

                    bones.push(BonePose {
                        id: *bone_name,
                        transform: IFrameTransform::new(pos, rot),
                    });

                    baseline.bones.insert(*bone_name, pos);
                }
            }
        }

        baseline.root = root_pos;
        info!(?root_pos, "-> i-frame");

        let frame = PlayerIFrame {
            root: IFrameTransform::new(root_pos, root_rot),
            bones,
        };

        if let Err(err) = nt.command_tx.send(NetworkCommand::PublishIFrame(frame)) {
            error!(?err, "send error");
        }
    } else {
        // Build P-frame with delta positions.
        let mut bones = Vec::with_capacity(tracked_bones.0.len());

        if let Some(avatar_bones) = bones_map {
            for bone_name in &tracked_bones.0 {
                if let Some(&bone_entity) = avatar_bones.get(bone_name)
                    && let Ok(bone_tr) = bone_transforms.get(bone_entity)
                {
                    let (_, rot, pos) = bone_tr.to_scale_rotation_translation();
                    let baseline_pos = baseline.bones.get(bone_name).copied().unwrap_or(pos);

                    bones.push(BonePose {
                        id: *bone_name,
                        transform: PFrameTransform::new(pos, baseline_pos, rot),
                    });
                }
            }
        }

        let frame = PlayerPFrame {
            root: PFrameTransform::new(root_pos, baseline.root, root_rot),
            bones,
        };

        if let Err(err) = nt.command_tx.send(NetworkCommand::PublishPFrame(frame)) {
            error!(?err, "send error");
        }
    }
}
