use std::{collections::HashMap, sync::Arc};

use bevy::prelude::*;
use bevy_hsd::{HsdDoc, HsdEntityMap, HsdNodeTreeId};
use bevy_vrm::BoneName;
use loro::{LoroDoc, LoroMap, LoroTree, LoroValue, TreeParentId};
use unavi_avatar::{AvatarBones, AvatarBonesPopulated};
use unavi_locomotion::{AgentEntities, LocalAgent};
use unavi_script::agent::LocalAgentHsdDoc;

/// Marker for the ECS entity holding the agent HSD document.
#[derive(Component)]
pub struct AgentHsdDoc;

pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, init_agent_hsd_doc)
            .add_systems(PreUpdate, sync_agent_bone_transforms)
            .add_systems(PostUpdate, parent_bone_proxy_entities);
    }
}

/// All VRM bone names in declaration order (for proxy node creation).
const ALL_BONES: &[BoneName] = &[
    BoneName::Hips,
    BoneName::Spine,
    BoneName::Chest,
    BoneName::UpperChest,
    BoneName::Neck,
    BoneName::Head,
    BoneName::LeftEye,
    BoneName::RightEye,
    BoneName::Jaw,
    BoneName::LeftShoulder,
    BoneName::LeftUpperArm,
    BoneName::LeftLowerArm,
    BoneName::LeftHand,
    BoneName::RightShoulder,
    BoneName::RightUpperArm,
    BoneName::RightLowerArm,
    BoneName::RightHand,
    BoneName::LeftUpperLeg,
    BoneName::LeftLowerLeg,
    BoneName::LeftFoot,
    BoneName::LeftToes,
    BoneName::RightUpperLeg,
    BoneName::RightLowerLeg,
    BoneName::RightFoot,
    BoneName::RightToes,
    BoneName::LeftThumbProximal,
    BoneName::LeftThumbIntermediate,
    BoneName::LeftThumbDistal,
    BoneName::LeftIndexProximal,
    BoneName::LeftIndexIntermediate,
    BoneName::LeftIndexDistal,
    BoneName::LeftMiddleProximal,
    BoneName::LeftMiddleIntermediate,
    BoneName::LeftMiddleDistal,
    BoneName::LeftRingProximal,
    BoneName::LeftRingIntermediate,
    BoneName::LeftRingDistal,
    BoneName::LeftLittleProximal,
    BoneName::LeftLittleIntermediate,
    BoneName::LeftLittleDistal,
    BoneName::RightThumbProximal,
    BoneName::RightThumbIntermediate,
    BoneName::RightThumbDistal,
    BoneName::RightIndexProximal,
    BoneName::RightIndexIntermediate,
    BoneName::RightIndexDistal,
    BoneName::RightMiddleProximal,
    BoneName::RightMiddleIntermediate,
    BoneName::RightMiddleDistal,
    BoneName::RightRingProximal,
    BoneName::RightRingIntermediate,
    BoneName::RightRingDistal,
    BoneName::RightLittleProximal,
    BoneName::RightLittleIntermediate,
    BoneName::RightLittleDistal,
];

/// Creates the agent HSD document when the local avatar's bones populate.
/// Inserts `LocalAgentHsdDoc` and spawns the `HsdDoc` entity.
pub fn init_agent_hsd_doc(
    mut commands: Commands,
    local_agents: Query<&AgentEntities, With<LocalAgent>>,
    avatars: Query<&AvatarBones, Added<AvatarBonesPopulated>>,
    existing: Option<Res<LocalAgentHsdDoc>>,
) {
    if existing.is_some() {
        return;
    }
    let Ok(agent_ents) = local_agents.single() else {
        return;
    };
    let Ok(bones) = avatars.get(agent_ents.avatar) else {
        return;
    };

    let doc = Arc::new(LoroDoc::new());
    let mut bone_nodes = HashMap::new();

    {
        let hsd = doc.get_map("hsd");
        let tree: LoroTree = hsd
            .get_or_create_container("nodes", LoroTree::new())
            .expect("agent nodes tree");

        for &bone_name in ALL_BONES {
            // Only create proxy nodes for bones present in the VRM.
            if !bones.contains_key(&bone_name) {
                continue;
            }
            let node_id = tree
                .create(TreeParentId::Root)
                .expect("create bone proxy node");
            let meta = tree.get_meta(node_id).expect("node meta");
            let bone_str = format!("{bone_name}");
            meta.insert("bone_name", bone_str.trim_matches('"'))
                .expect("insert bone_name");
            bone_nodes.insert(bone_name, node_id);
        }

        doc.commit();
    }

    let bone_nodes = Arc::new(bone_nodes);
    commands.insert_resource(LocalAgentHsdDoc {
        doc: Arc::clone(&doc),
        bone_nodes: Arc::clone(&bone_nodes),
    });
    commands.spawn((Name::new("agent"), AgentHsdDoc, HsdDoc(doc)));
}

/// Writes VRM bone `GlobalTransform`s into the agent HSD proxy node
/// metadata each frame, so scripts can read `global-transform()`.
pub fn sync_agent_bone_transforms(
    agent_doc: Option<Res<LocalAgentHsdDoc>>,
    local_agents: Query<&AgentEntities, With<LocalAgent>>,
    avatars: Query<&AvatarBones>,
    bone_transforms: Query<&GlobalTransform>,
) {
    let Some(ad) = agent_doc else { return };
    let Ok(agent_ents) = local_agents.single() else {
        return;
    };
    let Ok(bones) = avatars.get(agent_ents.avatar) else {
        return;
    };

    let hsd = ad.doc.get_map("hsd");
    let Ok(tree) = hsd.get_or_create_container("nodes", LoroTree::new()) else {
        return;
    };

    for (bone_name, &tree_id) in ad.bone_nodes.iter() {
        let Some(&bone_ent) = bones.get(bone_name) else {
            continue;
        };
        let Ok(gt) = bone_transforms.get(bone_ent) else {
            continue;
        };
        let Ok(meta) = tree.get_meta(tree_id) else {
            continue;
        };
        let t = gt.translation();
        let r = gt.to_scale_rotation_translation().1;
        let s = gt.scale();
        write_f64s_to_map(
            &meta,
            "translation",
            &[f64::from(t.x), f64::from(t.y), f64::from(t.z)],
        );
        write_f64s_to_map(
            &meta,
            "rotation",
            &[
                f64::from(r.x),
                f64::from(r.y),
                f64::from(r.z),
                f64::from(r.w),
            ],
        );
        write_f64s_to_map(
            &meta,
            "scale",
            &[f64::from(s.x), f64::from(s.y), f64::from(s.z)],
        );
    }

    ad.doc.commit();
}

fn write_f64s_to_map(map: &LoroMap, key: &str, vals: &[f64]) {
    let Ok(list) = map.get_or_create_container(key, loro::LoroList::new()) else {
        return;
    };
    let len = list.len();
    if len > 0 {
        let _ = list.delete(0, len);
    }
    for &v in vals {
        let _ = list.push(LoroValue::Double(v));
    }
}

/// Parents HSD bone-proxy node entities to their corresponding VRM bone
/// entities when they are first hydrated. Only runs on newly-added nodes
/// that belong to the agent HSD document.
pub fn parent_bone_proxy_entities(
    mut commands: Commands,
    agent_doc_entity: Query<Entity, With<AgentHsdDoc>>,
    agent_doc: Option<Res<LocalAgentHsdDoc>>,
    local_agents: Query<&AgentEntities, With<LocalAgent>>,
    avatars: Query<&AvatarBones>,
    // All new HSD node entities
    new_nodes: Query<(Entity, &HsdNodeTreeId), Added<HsdNodeTreeId>>,
    // Lookup entity's HSD doc entity
    entity_maps: Query<&HsdEntityMap>,
) {
    let Some(ad) = agent_doc else { return };
    let Ok(agent_ent) = agent_doc_entity.single() else {
        return;
    };
    let Ok(agent_ents) = local_agents.single() else {
        return;
    };
    let Ok(bones) = avatars.get(agent_ents.avatar) else {
        return;
    };

    let hsd = ad.doc.get_map("hsd");
    let Ok(tree) = hsd.get_or_create_container("nodes", LoroTree::new()) else {
        return;
    };

    // Find the entity map for the agent doc entity so we can confirm nodes
    // belong to it.
    let Ok(emap) = entity_maps.get(agent_ent) else {
        return;
    };

    for (node_ent, tree_id_comp) in &new_nodes {
        // Check this node belongs to the agent doc.
        let tree_id_str = tree_id_comp.0.as_str();
        if !emap.nodes.contains_key(tree_id_str) {
            continue;
        }

        // Parse tree_id and check if it is a root-level proxy node.
        let Ok(tree_id) = loro::TreeID::try_from(tree_id_str) else {
            continue;
        };
        let Some(loro::TreeParentId::Root) = tree.parent(tree_id) else {
            continue;
        };

        // Read the bone_name from metadata.
        let Ok(meta) = tree.get_meta(tree_id) else {
            continue;
        };
        let Some(loro::ValueOrContainer::Value(LoroValue::String(bone_name_str))) =
            meta.get("bone_name")
        else {
            continue;
        };

        // Find the matching VRM bone entity.
        let Some((&vrm_bone, &bone_ent)) = bones
            .iter()
            .find(|(b, _)| format!("{b}").trim_matches('"') == bone_name_str.as_str())
        else {
            continue;
        };
        let _ = vrm_bone;

        // Parent the proxy node entity to the VRM bone entity.
        commands
            .entity(node_ent)
            .insert((ChildOf(bone_ent), Transform::IDENTITY));
    }
}
