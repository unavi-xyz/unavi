use bevy::prelude::*;
use bevy_vrm::BoneName;
use unavi_avatar::bones::AvatarBones;

use crate::{
    AgentAvatar,
    LocalAgentEntities,
    tracking::{
        TrackedHead,
        TrackedPose,
    },
};

/// Applies tracked head pose to the avatar's head bone.
pub fn apply_head_tracking(
    agents: Query<(&AgentAvatar, &LocalAgentEntities)>,
    tracked_heads: Query<&TrackedPose, With<TrackedHead>>,
    avatars: Query<&AvatarBones>,
    mut bones: Query<&mut Transform, With<BoneName>>,
) {
    for (avatar_ent, entities) in agents.iter() {
        let Ok(pose) = tracked_heads.get(entities.tracked_head) else {
            continue;
        };

        let Ok(avatar_bones) = avatars.get(avatar_ent.0) else {
            continue;
        };

        let Some(&head_bone) = avatar_bones.get(&BoneName::Head) else {
            continue;
        };

        let Ok(mut head_transform) = bones.get_mut(head_bone) else {
            continue;
        };

        head_transform.rotation = pose.rotation;
    }
}
