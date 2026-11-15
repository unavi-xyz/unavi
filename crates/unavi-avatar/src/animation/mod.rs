use bevy::prelude::*;
use serde_vrm::vrm1::vrmc_vrm::humanoid::HumanBoneName;

use crate::animation::{
    load::AvatarAnimationNodes,
    weights::AnimationWeights,
};

pub mod defaults;
pub mod load;
pub mod locomotion;
mod mixamo;
pub mod raw;
pub mod velocity;
pub mod weights;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AnimationName {
    Falling,
    #[default]
    Idle,
    Sprint,
    Walk,
    WalkLeft,
    WalkRight,
}

#[derive(Component)]
#[require(AnimationWeights)]
pub struct AnimationPlayerInitialized;

pub fn init_animation_players(
    mut commands: Commands,
    animation_players: Query<
        (Entity, &ChildOf),
        (With<AnimationPlayer>, Without<AnimationPlayerInitialized>),
    >,
    animation_nodes: Query<&AnimationGraphHandle, With<AvatarAnimationNodes>>,
) {
    for (entity, parent) in animation_players.iter() {
        let Ok(graph) = animation_nodes.get(parent.parent()) else {
            continue;
        };

        commands
            .entity(entity)
            .insert((graph.clone(), AnimationPlayerInitialized));
    }
}

/// The animation graph's mask group for `bone`.
///
/// `HumanBoneName` is a fieldless enum with no two variants sharing a
/// discriminant, so its own discriminant is already a distinct group id;
/// see the test below for the guarantee this relies on.
#[must_use]
pub const fn bone_mask_group(bone: HumanBoneName) -> u32 {
    bone as u32
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use serde_vrm::vrm1::vrmc_vrm::humanoid::HumanBoneName;

    use super::bone_mask_group;

    /// Every bone this animation system knows about (see
    /// [`mixamo::MIXAMO_BONES`](super::mixamo) for the subset Mixamo
    /// clips retarget onto) plus the ones it does not: a full
    /// cross-check that `HumanBoneName as u32` stays injective.
    const ALL_BONES: &[HumanBoneName] = &[
        HumanBoneName::Hips,
        HumanBoneName::Spine,
        HumanBoneName::Chest,
        HumanBoneName::UpperChest,
        HumanBoneName::Neck,
        HumanBoneName::Head,
        HumanBoneName::LeftEye,
        HumanBoneName::RightEye,
        HumanBoneName::Jaw,
        HumanBoneName::LeftShoulder,
        HumanBoneName::LeftUpperArm,
        HumanBoneName::LeftLowerArm,
        HumanBoneName::LeftHand,
        HumanBoneName::RightShoulder,
        HumanBoneName::RightUpperArm,
        HumanBoneName::RightLowerArm,
        HumanBoneName::RightHand,
        HumanBoneName::LeftUpperLeg,
        HumanBoneName::LeftLowerLeg,
        HumanBoneName::LeftFoot,
        HumanBoneName::LeftToes,
        HumanBoneName::RightUpperLeg,
        HumanBoneName::RightLowerLeg,
        HumanBoneName::RightFoot,
        HumanBoneName::RightToes,
        HumanBoneName::LeftThumbMetacarpal,
        HumanBoneName::LeftThumbProximal,
        HumanBoneName::LeftThumbDistal,
        HumanBoneName::LeftIndexProximal,
        HumanBoneName::LeftIndexIntermediate,
        HumanBoneName::LeftIndexDistal,
        HumanBoneName::LeftMiddleProximal,
        HumanBoneName::LeftMiddleIntermediate,
        HumanBoneName::LeftMiddleDistal,
        HumanBoneName::LeftRingProximal,
        HumanBoneName::LeftRingIntermediate,
        HumanBoneName::LeftRingDistal,
        HumanBoneName::LeftLittleProximal,
        HumanBoneName::LeftLittleIntermediate,
        HumanBoneName::LeftLittleDistal,
        HumanBoneName::RightThumbMetacarpal,
        HumanBoneName::RightThumbProximal,
        HumanBoneName::RightThumbDistal,
        HumanBoneName::RightIndexProximal,
        HumanBoneName::RightIndexIntermediate,
        HumanBoneName::RightIndexDistal,
        HumanBoneName::RightMiddleProximal,
        HumanBoneName::RightMiddleIntermediate,
        HumanBoneName::RightMiddleDistal,
        HumanBoneName::RightRingProximal,
        HumanBoneName::RightRingIntermediate,
        HumanBoneName::RightRingDistal,
        HumanBoneName::RightLittleProximal,
        HumanBoneName::RightLittleIntermediate,
        HumanBoneName::RightLittleDistal,
    ];

    #[test]
    fn every_bone_gets_a_distinct_mask_group() {
        let groups: HashSet<u32> = ALL_BONES.iter().copied().map(bone_mask_group).collect();
        assert_eq!(groups.len(), ALL_BONES.len());
        // `AnimationMask` is a u64, so a group past 63 masks nothing.
        assert!(groups.iter().all(|&group| group < u64::BITS));
    }
}
