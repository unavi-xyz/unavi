use bevy::prelude::*;
use bevy_vr_controller::player::PlayerAvatar;
use bevy_vrm::BoneName;
use unavi_constants::player::LOCAL_PLAYER_ID;
use unavi_player::id::PlayerId;

use crate::{execution::ScriptTickrate, load::ScriptMap};

#[derive(Component)]
pub struct CopyTransform(pub Entity);

#[derive(Component)]
pub struct CopyGlobalTransform(pub Entity);

pub(crate) fn update_player_skeletons(
    bones: Query<(Entity, &PlayerId, &BoneName, &Transform)>,
    mut commands: Commands,
    players: Query<(Entity, &PlayerId), With<PlayerAvatar>>,
    script_map: NonSendMut<ScriptMap>,
    scripts: Query<(Entity, &ScriptTickrate)>,
) {
    for (entity, tickrate) in scripts.iter() {
        if !tickrate.ready_for_update {
            continue;
        }

        let mut scripts = script_map.lock().unwrap();

        let Some(script) = scripts.get_mut(&entity) else {
            continue;
        };

        let data = script.store.data_mut();
        let player = &data.api.wired_player.as_ref().unwrap().local_player.read();

        for (player_ent, id) in players.iter() {
            if id.0 != LOCAL_PLAYER_ID {
                continue;
            }

            commands
                .entity(*player.root.read().entity.get().unwrap())
                .insert(CopyGlobalTransform(player_ent));

            let pairs = [
                (BoneName::Hips, &player.skeleton.hips),
                (BoneName::Chest, &player.skeleton.chest),
                (BoneName::UpperChest, &player.skeleton.upper_chest),
                (BoneName::Neck, &player.skeleton.neck),
                (BoneName::Head, &player.skeleton.head),
                (BoneName::Spine, &player.skeleton.spine),
                (BoneName::LeftShoulder, &player.skeleton.left_shoulder),
                (BoneName::LeftUpperArm, &player.skeleton.left_upper_arm),
                (BoneName::LeftLowerArm, &player.skeleton.left_lower_arm),
                (BoneName::LeftHand, &player.skeleton.left_hand),
                (BoneName::LeftUpperLeg, &player.skeleton.left_upper_leg),
                (BoneName::LeftLowerLeg, &player.skeleton.left_lower_leg),
                (BoneName::LeftFoot, &player.skeleton.left_foot),
                (BoneName::RightShoulder, &player.skeleton.right_shoulder),
                (BoneName::RightUpperArm, &player.skeleton.right_upper_arm),
                (BoneName::RightLowerArm, &player.skeleton.right_lower_arm),
                (BoneName::RightHand, &player.skeleton.right_hand),
                (BoneName::RightUpperLeg, &player.skeleton.right_upper_leg),
                (BoneName::RightLowerLeg, &player.skeleton.right_lower_leg),
                (BoneName::RightFoot, &player.skeleton.right_foot),
            ];

            for (bone_ent, bone_id, bone_name, bone_transform) in bones.iter() {
                if bone_id != id {
                    continue;
                }

                for (pair_name, node) in pairs.iter() {
                    if pair_name != bone_name {
                        continue;
                    }

                    let mut node = node.write();

                    // We do not use parenting, because it causes issues when
                    // physics colliders are children of the player rigid body.
                    commands
                        .entity(*node.entity.get().unwrap())
                        .insert(CopyTransform(bone_ent));

                    // Set node resource transform.
                    node.transform = *bone_transform;
                }
            }
        }
    }
}

pub(crate) fn copy_transforms(
    mut targets: Query<(&CopyTransform, &mut Transform)>,
    transforms: Query<&Transform, Without<CopyTransform>>,
) {
    for (target, mut transform) in targets.iter_mut() {
        if let Ok(found) = transforms.get(target.0) {
            *transform = *found;
        }
    }
}

pub(crate) fn copy_global_transforms(
    mut targets: Query<(&CopyGlobalTransform, &mut Transform)>,
    global_transforms: Query<&GlobalTransform, Without<CopyGlobalTransform>>,
) {
    for (target, mut transform) in targets.iter_mut() {
        if let Ok(found) = global_transforms.get(target.0) {
            *transform = found.compute_transform();
        }
    }
}
