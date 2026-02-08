use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use bevy::prelude::*;
use bevy_vrm::BoneName;
use unavi_avatar::AvatarBones;
use unavi_locomotion::{AgentConfig, AgentEntities, AgentRig, LocalAgent};

use crate::networking::{
    publish_utils::{IFRAME_FREQ, PUBLISH_INTERVAL, transform_changed},
    thread::{
        NetworkCommand, NetworkingThread,
        space::types::pose::{
            AgentIFrame, AgentPFrame, BonePose, IFrameTransform, PFrameRootTransform,
            PFrameTransform,
        },
    },
};

/// Stores the last I-frame positions for P-frame delta encoding,
/// and last-sent transforms for epsilon filtering.
#[derive(Default)]
pub(super) struct IFrameBaseline {
    root: Vec3,
    root_rot: Quat,
    bones: HashMap<BoneName, (Vec3, Quat)>,
}

pub(super) fn publish_agent_transforms(
    nt: Res<NetworkingThread>,
    local_agent: Query<(&AgentConfig, &AgentEntities), With<LocalAgent>>,
    avatar_bones: Query<&AvatarBones>,
    body_transforms: Query<&GlobalTransform, (With<AgentRig>, Without<BoneName>)>,
    bone_transforms: Query<&Transform, With<BoneName>>,
    tracked_bones: Res<TrackedBones>,
    time: Res<Time>,
    mut last: Local<Duration>,
    mut count: Local<u64>,
    mut baseline: Local<IFrameBaseline>,
) {
    let now = time.elapsed();
    if now.saturating_sub(*last) < PUBLISH_INTERVAL {
        return;
    }
    *last = now;
    *count += 1;

    let Ok((config, entities)) = local_agent.single() else {
        return;
    };

    let Ok(root_tr) = body_transforms.get(entities.body) else {
        return;
    };

    let mut root_pos = root_tr.translation();
    root_pos.y -= config.float_height();

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
                    bones.push(BonePose {
                        id: *bone_name,
                        transform: IFrameTransform::new(bone_tr.translation, bone_tr.rotation),
                    });

                    baseline
                        .bones
                        .insert(*bone_name, (bone_tr.translation, bone_tr.rotation));
                }
            }
        }

        baseline.root = root_pos;
        baseline.root_rot = root_rot;

        let frame = AgentIFrame {
            root: IFrameTransform::new(root_pos, root_rot),
            bones,
        };

        if let Err(err) = nt
            .command_tx
            .try_send(NetworkCommand::PublishAgentIFrame(frame))
        {
            error!(?err, "send error");
        }
    } else {
        // Build P-frame with delta positions, filtering unchanged bones.
        let mut bones = Vec::with_capacity(tracked_bones.0.len());

        if let Some(avatar_bones) = bones_map {
            for bone_name in &tracked_bones.0 {
                if let Some(&bone_entity) = avatar_bones.get(bone_name)
                    && let Ok(bone_tr) = bone_transforms.get(bone_entity)
                {
                    let (baseline_pos, last_rot) = baseline
                        .bones
                        .get(bone_name)
                        .copied()
                        .unwrap_or((bone_tr.translation, bone_tr.rotation));

                    if !transform_changed(
                        bone_tr.translation,
                        bone_tr.rotation,
                        baseline_pos,
                        last_rot,
                    ) {
                        continue;
                    }

                    bones.push(BonePose {
                        id: *bone_name,
                        transform: PFrameTransform::new(
                            bone_tr.translation,
                            baseline_pos,
                            bone_tr.rotation,
                        ),
                    });

                    baseline
                        .bones
                        .insert(*bone_name, (bone_tr.translation, bone_tr.rotation));
                }
            }
        }

        let frame = AgentPFrame {
            root: PFrameRootTransform::new(root_pos, baseline.root, root_rot),
            bones,
        };

        if let Err(err) = nt
            .command_tx
            .try_send(NetworkCommand::PublishAgentPFrame(frame))
        {
            error!(?err, "send error");
        }
    }
}

/// Set of bones to include in pose updates.
#[derive(Resource, Default, Clone)]
pub struct TrackedBones(pub HashSet<BoneName>);

#[allow(unused)]
impl TrackedBones {
    /// Returns a set containing left hand and all left finger bones.
    pub fn left_hand() -> HashSet<BoneName> {
        HashSet::from([
            BoneName::LeftHand,
            BoneName::LeftThumbProximal,
            BoneName::LeftThumbIntermediate,
            BoneName::LeftThumbDistal,
            BoneName::LeftIndexProximal,
            BoneName::LeftIndexIntermediate,
            BoneName::LeftIndexDistal,
            BoneName::LeftMiddleProximal,
            BoneName::LeftMiddleIntermediate,
            BoneName::LeftMiddleDistal,
            BoneName::LeftRingProximal,
            BoneName::LeftRingIntermediate,
            BoneName::LeftRingDistal,
            BoneName::LeftLittleProximal,
            BoneName::LeftLittleIntermediate,
            BoneName::LeftLittleDistal,
        ])
    }

    /// Returns a set containing right hand and all right finger bones.
    pub fn right_hand() -> HashSet<BoneName> {
        HashSet::from([
            BoneName::RightHand,
            BoneName::RightThumbProximal,
            BoneName::RightThumbIntermediate,
            BoneName::RightThumbDistal,
            BoneName::RightIndexProximal,
            BoneName::RightIndexIntermediate,
            BoneName::RightIndexDistal,
            BoneName::RightMiddleProximal,
            BoneName::RightMiddleIntermediate,
            BoneName::RightMiddleDistal,
            BoneName::RightRingProximal,
            BoneName::RightRingIntermediate,
            BoneName::RightRingDistal,
            BoneName::RightLittleProximal,
            BoneName::RightLittleIntermediate,
            BoneName::RightLittleDistal,
        ])
    }

    /// Returns a set containing all VRM bones.
    pub fn full() -> HashSet<BoneName> {
        HashSet::from([
            BoneName::Hips,
            BoneName::LeftUpperLeg,
            BoneName::RightUpperLeg,
            BoneName::LeftLowerLeg,
            BoneName::RightLowerLeg,
            BoneName::LeftFoot,
            BoneName::RightFoot,
            BoneName::Spine,
            BoneName::Chest,
            BoneName::Neck,
            BoneName::Head,
            BoneName::LeftShoulder,
            BoneName::RightShoulder,
            BoneName::LeftUpperArm,
            BoneName::RightUpperArm,
            BoneName::LeftLowerArm,
            BoneName::RightLowerArm,
            BoneName::LeftHand,
            BoneName::RightHand,
            BoneName::LeftToes,
            BoneName::RightToes,
            BoneName::LeftEye,
            BoneName::RightEye,
            BoneName::Jaw,
            BoneName::LeftThumbProximal,
            BoneName::LeftThumbIntermediate,
            BoneName::LeftThumbDistal,
            BoneName::LeftIndexProximal,
            BoneName::LeftIndexIntermediate,
            BoneName::LeftIndexDistal,
            BoneName::LeftMiddleProximal,
            BoneName::LeftMiddleIntermediate,
            BoneName::LeftMiddleDistal,
            BoneName::LeftRingProximal,
            BoneName::LeftRingIntermediate,
            BoneName::LeftRingDistal,
            BoneName::LeftLittleProximal,
            BoneName::LeftLittleIntermediate,
            BoneName::LeftLittleDistal,
            BoneName::RightThumbProximal,
            BoneName::RightThumbIntermediate,
            BoneName::RightThumbDistal,
            BoneName::RightIndexProximal,
            BoneName::RightIndexIntermediate,
            BoneName::RightIndexDistal,
            BoneName::RightMiddleProximal,
            BoneName::RightMiddleIntermediate,
            BoneName::RightMiddleDistal,
            BoneName::RightRingProximal,
            BoneName::RightRingIntermediate,
            BoneName::RightRingDistal,
            BoneName::RightLittleProximal,
            BoneName::RightLittleIntermediate,
            BoneName::RightLittleDistal,
            BoneName::UpperChest,
        ])
    }
}
