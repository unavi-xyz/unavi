use bevy::prelude::*;
use bevy_vrm::{loader::Vrm, BoneName};

use crate::{
    api::{utils::RefResource, wired::scene::bindings::node::HostNode},
    execution::ScriptTickrate,
    load::ScriptMap,
};

use super::bindings::api::Node;

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
        let root = Node::from_res(&player.root, &data.table).unwrap();
        let skeleton = player.skeleton.clone_ref(&data.table).unwrap();

        for (id, player_transform) in players.iter() {
            if id.0 != LOCAL_PLAYER_ID {
                continue;
            }

            data.set_transform(
                Node::from_res(&root, &data.table).unwrap(),
                player_transform.compute_transform().into(),
            )
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

            for (bone_ent, bone_id, bone_name, bone_transform) in bones.iter() {
                if bone_id != id {
                    continue;
                }

                for (pair_name, node_res) in pairs.iter() {
                    if pair_name != bone_name {
                        continue;
                    }

                    if let Ok(nodes) = data.entities.nodes.read() {
                        if let Some(node_ent) = nodes.get(&node_res.rep()) {
                            commands
                                .entity(*node_ent)
                                // Reset transform (in case scripts try to change it).
                                .insert(Transform::default())
                                // Parent node entity to bone.
                                .set_parent(bone_ent);
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
