use bevy::prelude::*;
use bevy_vrm::BoneName;

pub mod defaults;
pub mod grounded;
pub mod load;
mod mixamo;
pub mod velocity;
pub mod weights;

pub use load::AvatarAnimationNodes;
use weights::{AnimationWeights, TargetAnimationWeights};

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum AnimationName {
    Falling,
    #[default]
    Idle,
    Walk,
    WalkLeft,
    WalkRight,
    Sprint,
    _Other(&'static str),
}

/// Marker to track which animation players have been initialized.
#[derive(Component)]
pub struct AnimationPlayerInitialized;

/// Initializes animation components when `AnimationPlayer` is added.
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

        commands.entity(entity).insert((
            AnimationWeights::default(),
            TargetAnimationWeights::default(),
            graph.clone(),
            AnimationPlayerInitialized,
        ));
    }
}

/// Returns a unique mask group ID (0-53) for each VRM bone.
/// Used for animation masking to disable animation on tracked bones.
#[must_use]
pub const fn bone_mask_group(bone: BoneName) -> u32 {
    match bone {
        BoneName::Hips => 0,
        BoneName::Spine => 1,
        BoneName::Chest => 2,
        BoneName::Neck => 3,
        BoneName::Head => 4,
        BoneName::LeftShoulder => 5,
        BoneName::LeftUpperArm => 6,
        BoneName::LeftLowerArm => 7,
        BoneName::LeftHand => 8,
        BoneName::RightShoulder => 9,
        BoneName::RightUpperArm => 10,
        BoneName::RightLowerArm => 11,
        BoneName::RightHand => 12,
        BoneName::LeftUpperLeg => 13,
        BoneName::LeftLowerLeg => 14,
        BoneName::LeftFoot => 15,
        BoneName::LeftToes => 16,
        BoneName::RightUpperLeg => 17,
        BoneName::RightLowerLeg => 18,
        BoneName::RightFoot => 19,
        BoneName::RightToes => 20,
        BoneName::LeftEye => 21,
        BoneName::RightEye => 22,
        BoneName::Jaw => 23,
        BoneName::LeftThumbProximal => 24,
        BoneName::LeftThumbIntermediate => 25,
        BoneName::LeftThumbDistal => 26,
        BoneName::LeftIndexProximal => 27,
        BoneName::LeftIndexIntermediate => 28,
        BoneName::LeftIndexDistal => 29,
        BoneName::LeftMiddleProximal => 30,
        BoneName::LeftMiddleIntermediate => 31,
        BoneName::LeftMiddleDistal => 32,
        BoneName::LeftRingProximal => 33,
        BoneName::LeftRingIntermediate => 34,
        BoneName::LeftRingDistal => 35,
        BoneName::LeftLittleProximal => 36,
        BoneName::LeftLittleIntermediate => 37,
        BoneName::LeftLittleDistal => 38,
        BoneName::RightThumbProximal => 39,
        BoneName::RightThumbIntermediate => 40,
        BoneName::RightThumbDistal => 41,
        BoneName::RightIndexProximal => 42,
        BoneName::RightIndexIntermediate => 43,
        BoneName::RightIndexDistal => 44,
        BoneName::RightMiddleProximal => 45,
        BoneName::RightMiddleIntermediate => 46,
        BoneName::RightMiddleDistal => 47,
        BoneName::RightRingProximal => 48,
        BoneName::RightRingIntermediate => 49,
        BoneName::RightRingDistal => 50,
        BoneName::RightLittleProximal => 51,
        BoneName::RightLittleIntermediate => 52,
        BoneName::RightLittleDistal => 53,
        BoneName::UpperChest => 54,
    }
}
