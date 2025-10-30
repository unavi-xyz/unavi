use bevy::prelude::*;
use bevy_vrm::BoneName;

use crate::{PlayerCamera, eye_offset::EyeOffset};

#[derive(Component)]
pub struct AvatarHead(pub Entity);

pub(crate) fn set_avatar_head(
    avatars: Query<Entity, (With<EyeOffset>, Without<AvatarHead>)>,
    bones: Query<(Entity, &BoneName)>,
    mut commands: Commands,
    parents: Query<&ChildOf>,
) {
    for avatar_ent in avatars.iter() {
        for (bone_ent, bone_name) in bones.iter() {
            if *bone_name != BoneName::Head {
                continue;
            }

            if is_child(bone_ent, avatar_ent, &parents) {
                commands.entity(avatar_ent).insert(AvatarHead(bone_ent));
                break;
            }
        }
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

#[derive(Component)]
pub struct BaseRotation(pub Quat);

pub(crate) fn rotate_avatar_head(
    avatars: Query<(&AvatarHead, &EyeOffset)>,
    mut bones: Query<
        (&mut Transform, Option<&BaseRotation>),
        (With<BoneName>, Without<PlayerCamera>),
    >,
    mut player_cam: Query<&mut Transform, With<PlayerCamera>>,
    mut commands: Commands,
) {
    for (head, offset) in avatars.iter() {
        let (mut head_tr, base) = bones.get_mut(head.0).expect("Avatar head bone not found");

        let Some(base) = base else {
            commands
                .entity(head.0)
                .insert(BaseRotation(head_tr.rotation));
            continue;
        };

        let mut camera_tr = player_cam.single_mut().unwrap();
        camera_tr.translation = offset.0;

        let new_rot = base.0 * camera_tr.rotation;
        head_tr.rotation = new_rot;
    }
}
