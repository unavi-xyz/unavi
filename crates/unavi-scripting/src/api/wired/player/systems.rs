use bevy::prelude::*;
use bevy_vr_controller::player::PlayerAvatar;
use bevy_vrm::BoneName;
use unavi_constants::player::LOCAL_PLAYER_ID;
use unavi_player::id::PlayerId;

use crate::{
    api::{utils::RefResource, wired::scene::nodes::base::NodeRes},
    execution::ScriptTickrate,
    load::ScriptMap,
};

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
        let player = data
            .table
            .get(&data.api.wired_player.as_ref().unwrap().local_player)
            .unwrap();
        let root = NodeRes::from_res(&player.root, &data.table).unwrap();
        let skeleton = player.skeleton.clone_ref(&data.table).unwrap();

        for (player_ent, id) in players.iter() {
            if id.0 != LOCAL_PLAYER_ID {
                continue;
            }

            if let Ok(nodes) = data.api.wired_scene.as_ref().unwrap().entities.nodes.read() {
                if let Some(node_ent) = nodes.get(&root.rep()) {
                    commands
                        .entity(*node_ent)
                        .insert(CopyGlobalTransform(player_ent));
                }
            }

            let pairs: [(BoneName, &wasm_bridge::component::Resource<NodeRes>); 20] = [
                (BoneName::Hips, &skeleton.hips),
                (BoneName::Chest, &skeleton.chest),
                (BoneName::UpperChest, &skeleton.upper_chest),
                (BoneName::Neck, &skeleton.neck),
                (BoneName::Head, &skeleton.head),
                (BoneName::Spine, &skeleton.spine),
                (BoneName::LeftShoulder, &skeleton.left_shoulder),
                (BoneName::LeftUpperArm, &skeleton.left_upper_arm),
                (BoneName::LeftLowerArm, &skeleton.left_lower_arm),
                (BoneName::LeftHand, &skeleton.left_hand),
                (BoneName::LeftUpperLeg, &skeleton.left_upper_leg),
                (BoneName::LeftLowerLeg, &skeleton.left_lower_leg),
                (BoneName::LeftFoot, &skeleton.left_foot),
                (BoneName::RightShoulder, &skeleton.right_shoulder),
                (BoneName::RightUpperArm, &skeleton.right_upper_arm),
                (BoneName::RightLowerArm, &skeleton.right_lower_arm),
                (BoneName::RightHand, &skeleton.right_hand),
                (BoneName::RightUpperLeg, &skeleton.right_upper_leg),
                (BoneName::RightLowerLeg, &skeleton.right_lower_leg),
                (BoneName::RightFoot, &skeleton.right_foot),
            ];

            for (bone_ent, bone_id, bone_name, bone_transform) in bones.iter() {
                if bone_id != id {
                    continue;
                }

                for (pair_name, node_res) in pairs.iter() {
                    if pair_name != bone_name {
                        continue;
                    }

                    if let Ok(nodes) = data.api.wired_scene.as_ref().unwrap().entities.nodes.read()
                    {
                        if let Some(node_ent) = nodes.get(&node_res.rep()) {
                            commands.entity(*node_ent).insert(
                                // "Parent" node entity to bone.
                                // We do not use actual parenting, because it causes issues when
                                // physics colliders are children of the player rigid body.
                                CopyTransform(bone_ent),
                            );
                        }
                    }

                    // Set node resource transform.
                    let node = data.table.get_mut(node_res).unwrap();
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
