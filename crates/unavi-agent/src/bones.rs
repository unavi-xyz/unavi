use bevy::prelude::*;
use bevy_vrm::BoneName;
use unavi_avatar::bones::AvatarBones;

use crate::{
    AgentAvatar,
    LocalAgentEntities,
};

pub fn apply_head_tracking(
    agents: Query<(&AgentAvatar, &LocalAgentEntities)>,
    avatars: Query<&AvatarBones>,
    mut transforms: Query<&mut Transform>,
) {
    for (avatar_ent, entities) in agents.iter() {
        let Ok(avatar_bones) = avatars.get(avatar_ent.0) else {
            continue;
        };

        let Some(&head_bone) = avatar_bones.get(&BoneName::Head) else {
            continue;
        };

        let Ok(head_transform) = transforms.get(entities.tracked_head) else {
            continue;
        };
        let rotation = head_transform.rotation;

        if let Ok(mut bone_transform) = transforms.get_mut(head_bone) {
            bone_transform.rotation = rotation;
        }
    }
}
