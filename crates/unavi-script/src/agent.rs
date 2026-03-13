use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_hsd::{NodeId, cache::SceneRegistryInner};
use bevy_vrm::BoneName;
use smol_str::SmolStr;
use unavi_avatar::bones::AvatarBones;

pub struct AgentDocEntry {
    pub bone_nodes: Arc<HashMap<BoneName, SmolStr>>,
    pub bone_node_ids: Arc<HashMap<SmolStr, BoneName>>,
    pub registry: Arc<SceneRegistryInner>,
}

/// Marker on HSD proxy-node entities that are parented to VRM bones.
#[derive(Component)]
pub struct BoneProxy;

#[derive(Component, Default)]
pub struct AgentDocs {
    pub docs: Arc<Mutex<Vec<Arc<AgentDocEntry>>>>,
}

pub(crate) fn on_avatar_bones_added(
    trigger: On<Add, AvatarBones>,
    mut commands: Commands,
    child_of: Query<&ChildOf, With<AvatarBones>>,
) {
    let Ok(child_of) = child_of.get(trigger.entity) else {
        return;
    };
    commands
        .entity(child_of.parent())
        .insert(AgentDocs::default());
}

pub(crate) fn parent_bone_proxies(
    mut commands: Commands,
    new_nodes: Query<(Entity, &NodeId), Added<NodeId>>,
    agent_docs: Query<(&AgentDocs, &AvatarBones)>,
) {
    for (node_ent, node_id) in &new_nodes {
        info!("-> ATTEMPT PARENT");
        let id = &node_id.0;

        let bone_ent = agent_docs.iter().find_map(|(ad, avatar_bones)| {
            let docs = ad.docs.lock().expect("agent docs lock");
            docs.iter().find_map(|entry| {
                entry
                    .bone_node_ids
                    .get(id)
                    .and_then(|bone| avatar_bones.get(bone).copied())
            })
        });
        let Some(bone_ent) = bone_ent else { continue };

        info!("<- SUCESS PARENTED");
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
