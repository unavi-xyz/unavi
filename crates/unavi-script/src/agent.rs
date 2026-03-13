use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bevy::prelude::*;
use bevy_hsd::{
    HsdDoc, NodeId,
    cache::{SceneRegistry, SceneRegistryInner},
};
use bevy_vrm::BoneName;
use loro::{LoroDoc, LoroTree, LoroValue, TreeID, TreeParentId, ValueOrContainer};

pub struct AgentDocEntry {
    pub doc: Arc<LoroDoc>,
    pub bone_nodes: Arc<HashMap<BoneName, TreeID>>,
    pub registry: Arc<SceneRegistryInner>,
}

#[derive(Component)]
pub struct AgentHsdDoc;

/// Marker on HSD proxy-node entities that are parented to VRM bones.
#[derive(Component)]
pub struct BoneProxy;

#[derive(Resource)]
pub struct LocalAgentDocs {
    pub bone_entities: Arc<HashMap<BoneName, Entity>>,
    pub docs: Arc<Mutex<Vec<Arc<AgentDocEntry>>>>,
}

pub(crate) fn parent_bone_proxies(
    mut commands: Commands,
    agent_doc_entities: Query<(Entity, &HsdDoc, &SceneRegistry), With<AgentHsdDoc>>,
    agent_docs: Option<Res<LocalAgentDocs>>,
    new_nodes: Query<(Entity, &NodeId), Added<NodeId>>,
) {
    let Some(ad) = agent_docs else { return };

    for (node_ent, node_id) in &new_nodes {
        let id = &node_id.0;

        // Find the agent doc that owns this node.
        let found = agent_doc_entities
            .iter()
            .find_map(|(_, hsd_doc, registry)| {
                let node_map = registry.0.node_map.lock().expect("node_map lock");
                if let Some(node) = node_map.get(id)
                    && let Some(tree_id) = *node.tree_id.lock().expect("lock tree id")
                {
                    Some((tree_id, Arc::clone(&hsd_doc.0)))
                } else {
                    None
                }
            });
        let Some((tree_id, doc)) = found else {
            continue;
        };

        let hsd = doc.get_map("hsd");
        let Ok(tree) = hsd.get_or_create_container("nodes", LoroTree::new()) else {
            continue;
        };

        // Only parent root-level proxy nodes.
        let Some(TreeParentId::Root) = tree.parent(tree_id) else {
            continue;
        };

        let Ok(meta) = tree.get_meta(tree_id) else {
            continue;
        };
        let Some(ValueOrContainer::Value(LoroValue::String(bone_name_str))) = meta.get("bone_name")
        else {
            continue;
        };

        let Some((_, &bone_ent)) = ad
            .bone_entities
            .iter()
            .find(|(b, _)| b.to_string().trim_matches('"') == bone_name_str.as_str())
        else {
            continue;
        };

        commands
            .entity(node_ent)
            .insert((BoneProxy, ChildOf(bone_ent), Transform::IDENTITY));
    }
}

pub fn reset_bone_proxies(mut proxies: Query<&mut Transform, With<BoneProxy>>) {
    for mut t in &mut proxies {
        *t = Transform::IDENTITY;
    }
}
