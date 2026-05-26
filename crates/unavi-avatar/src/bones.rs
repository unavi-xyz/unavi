use std::collections::HashMap;

use bevy::prelude::*;
use bevy_vrm::{BoneName, VrmInstanceId};

#[derive(Component, Deref, DerefMut)]
pub struct AvatarBones(pub HashMap<BoneName, Entity>);

pub(crate) fn populate_avatar_bones(
    avatars: Query<Entity, (With<VrmInstanceId>, Without<AvatarBones>)>,
    bones: Query<(Entity, &BoneName)>,
    mut commands: Commands,
    parents: Query<&ChildOf>,
) {
    for entity in avatars {
        let mut avatar_bones = HashMap::new();

        for (bone_ent, bone_name) in bones.iter() {
            if !is_child(bone_ent, entity, &parents) {
                continue;
            }
            avatar_bones.insert(*bone_name, bone_ent);
        }

        if avatar_bones.is_empty() {
            continue;
        }

        commands.entity(entity).insert(AvatarBones(avatar_bones));
    }
}

fn is_child(target_child: Entity, target_parent: Entity, parents: &Query<&ChildOf>) -> bool {
    if target_child == target_parent {
        true
    } else if let Ok(child_of) = parents.get(target_child) {
        is_child(child_of.parent(), target_parent, parents)
    } else {
        false
    }
}
