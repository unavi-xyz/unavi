use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bevy::prelude::*;
use bevy_hsd::{NodeId, cache::SceneRegistryInner};
use bevy_vrm::BoneName;
use smol_str::SmolStr;

pub struct AgentDocEntry {
    pub bone_nodes: Arc<HashMap<BoneName, SmolStr>>,
    pub bone_node_ids: Arc<HashMap<SmolStr, BoneName>>,
    pub registry: Arc<SceneRegistryInner>,
}

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
    agent_docs: Option<Res<LocalAgentDocs>>,
    new_nodes: Query<(Entity, &NodeId), Added<NodeId>>,
) {
    let Some(ad) = agent_docs else { return };

    for (node_ent, node_id) in &new_nodes {
        let id = &node_id.0;

        // Find which agent doc owns this node and what bone entity it maps to.
        let bone_ent = {
            let docs = ad.docs.lock().expect("agent docs lock");
            docs.iter().find_map(|entry| {
                entry
                    .bone_node_ids
                    .get(id)
                    .and_then(|bone| ad.bone_entities.get(bone).copied())
            })
        };
        let Some(bone_ent) = bone_ent else { continue };

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
