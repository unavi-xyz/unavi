use bevy::prelude::*;
use bevy_vrm::BoneName;

use crate::tracking::{TrackedHead, TrackedPose};

/// Maps tracked poses to VRM avatar bones.
#[derive(Component, Default)]
pub struct AvatarBones {
    /// VRM head bone entity.
    pub head: Option<Entity>,
}

/// Marker to track which avatars have had bones populated.
#[derive(Component)]
pub(crate) struct AvatarBonesPopulated;

/// Finds and stores VRM bone entities for avatars with loaded scenes.
pub(crate) fn populate_avatar_bones(
    mut commands: Commands,
    avatars: Query<(Entity, &AvatarBones), (With<bevy_vrm::VrmScene>, Without<AvatarBonesPopulated>)>,
    bones: Query<(Entity, &BoneName)>,
    parents: Query<&ChildOf>,
) {
    for (avatar_ent, _) in avatars.iter() {
        let mut avatar_bones = AvatarBones::default();

        for (bone_ent, bone_name) in bones.iter() {
            if !is_child(bone_ent, avatar_ent, &parents) {
                continue;
            }

            if bone_name == &BoneName::Head {
                avatar_bones.head = Some(bone_ent);
            }
        }

        if avatar_bones.head.is_some() {
            commands.entity(avatar_ent).insert((avatar_bones, AvatarBonesPopulated));
        }
    }
}

/// Applies tracked head pose to the avatar's head bone.
pub(crate) fn apply_head_tracking(
    players: Query<&crate::PlayerEntities>,
    tracked_heads: Query<&TrackedPose, With<TrackedHead>>,
    avatars: Query<&AvatarBones>,
    mut bones: Query<&mut Transform, With<BoneName>>,
) {
    for entities in players.iter() {
        let Ok(pose) = tracked_heads.get(entities.tracked_head) else {
            continue;
        };

        let Ok(avatar_bones) = avatars.get(entities.avatar) else {
            continue;
        };

        let Some(head_bone) = avatar_bones.head else {
            continue;
        };

        let Ok(mut head_transform) = bones.get_mut(head_bone) else {
            continue;
        };

        head_transform.rotation = pose.rotation;
    }
}

/// Walks up the parent tree, searching for a specific Entity.
fn is_child(target_child: Entity, target_parent: Entity, parents: &Query<&ChildOf>) -> bool {
    if target_child == target_parent {
        true
    } else if let Ok(child_of) = parents.get(target_child) {
        is_child(child_of.parent(), target_parent, parents)
    } else {
        false
    }
}
