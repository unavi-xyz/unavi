use bevy::prelude::*;
use bevy_vrm::BoneName;

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
/// `BoneName` is a fieldless enum with no two variants sharing a
/// discriminant, so its own discriminant is already a distinct group id;
/// see the test below for the guarantee this relies on.
#[must_use]
pub const fn bone_mask_group(bone: BoneName) -> u32 {
    bone as u32
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use bevy_vrm::BoneName;

    use super::bone_mask_group;

    /// Every bone this animation system knows about (see
    /// [`mixamo::MIXAMO_BONES`](super::mixamo) for the subset Mixamo
    /// clips retarget onto) plus the ones it does not: a full
    /// cross-check that `BoneName as u32` stays injective.
    const ALL_BONES: &[BoneName] = &[
        BoneName::Hips,
        BoneName::Spine,
        BoneName::Chest,
        BoneName::UpperChest,
        BoneName::Neck,
        BoneName::Head,
        BoneName::LeftEye,
        BoneName::RightEye,
        BoneName::Jaw,
        BoneName::LeftShoulder,
        BoneName::LeftUpperArm,
        BoneName::LeftLowerArm,
        BoneName::LeftHand,
        BoneName::RightShoulder,
        BoneName::RightUpperArm,
        BoneName::RightLowerArm,
        BoneName::RightHand,
        BoneName::LeftUpperLeg,
        BoneName::LeftLowerLeg,
        BoneName::LeftFoot,
        BoneName::LeftToes,
        BoneName::RightUpperLeg,
        BoneName::RightLowerLeg,
        BoneName::RightFoot,
        BoneName::RightToes,
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
    ];

    #[test]
    fn every_bone_gets_a_distinct_mask_group() {
        let groups: HashSet<u32> = ALL_BONES.iter().copied().map(bone_mask_group).collect();
        assert_eq!(groups.len(), ALL_BONES.len());
        // `AnimationMask` is a u64, so a group past 63 masks nothing.
        assert!(groups.iter().all(|&group| group < u64::BITS));
    }
}
