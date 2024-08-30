use bevy::prelude::*;
use bevy_vrm::{loader::Vrm, BoneName};

use crate::{
    api::{
        utils::RefResource,
        wired_scene::{
            gltf::node::NodeRes,
            wired::{
                math::types::{Quat, Transform as WTransform, Vec3},
                scene::node::HostNode,
            },
        },
    },
    execution::ScriptTickrate,
    load::ScriptMap,
};

pub const LOCAL_PLAYER_ID: usize = 0;

#[derive(Component, PartialEq, Eq)]
pub struct PlayerId(pub usize);

pub(crate) fn update_player_skeletons(
    bones: Query<(Entity, &PlayerId, &BoneName, &Transform)>,
    mut commands: Commands,
    players: Query<(&PlayerId, &GlobalTransform), With<Handle<Vrm>>>,
    script_map: NonSendMut<ScriptMap>,
    scripts: Query<(Entity, &ScriptTickrate)>,
) {
    for (entity, tickrate) in scripts.iter() {
        if !tickrate.ready_for_update {
            continue;
        }

        let mut scripts = script_map.lock().unwrap();

        let Some((_, store)) = scripts.get_mut(&entity) else {
            continue;
        };

        let data = store.data_mut();
        let player = data.table.get(&data.local_player).unwrap();
        let root = NodeRes::from_res(&player.root, &data.table).unwrap();
        let skeleton = player.skeleton.clone_ref(&data.table).unwrap();

        for (id, transform) in players.iter() {
            if id.0 != LOCAL_PLAYER_ID {
                continue;
            }

            let (scale, rotation, translation) = transform.to_scale_rotation_translation();

            let transform = WTransform {
                translation: Vec3 {
                    x: translation.x,
                    y: translation.y,
                    z: translation.z,
                },
                rotation: Quat {
                    x: rotation.x,
                    y: rotation.y,
                    z: rotation.z,
                    w: rotation.w,
                },
                scale: Vec3 {
                    x: scale.x,
                    y: scale.y,
                    z: scale.z,
                },
            };
            data.set_transform(NodeRes::from_res(&root, &data.table).unwrap(), transform)
                .unwrap();

            let pairs = [
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

            for (bone_ent, bone_id, bone_name, transform) in bones.iter() {
                if bone_id != id {
                    continue;
                }

                for (pair_name, node_res) in pairs.iter() {
                    if pair_name != bone_name {
                        continue;
                    }

                    // Make node entity child of bone.
                    // TODO: Don't do this every tick
                    {
                        if let Ok(nodes) = data.entities.nodes.read() {
                            if let Some(node_ent) = nodes.get(&node_res.rep()) {
                                commands.entity(*node_ent).set_parent(bone_ent);
                            }
                        }
                    }

                    // Set resource transform.
                    let node = data.table.get_mut(node_res).unwrap();
                    node.transform.translation.x = transform.translation.x;
                    node.transform.translation.y = transform.translation.y;
                    node.transform.translation.z = transform.translation.z;
                    node.transform.rotation.x = transform.rotation.x;
                    node.transform.rotation.y = transform.rotation.y;
                    node.transform.rotation.z = transform.rotation.z;
                    node.transform.rotation.w = transform.rotation.w;
                    node.transform.scale.x = transform.scale.x;
                    node.transform.scale.y = transform.scale.y;
                    node.transform.scale.z = transform.scale.z;
                }
            }
        }
    }
}
