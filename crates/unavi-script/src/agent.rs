use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bevy::prelude::*;
use bevy_hsd::{HsdDoc, HsdNodeTreeId, SceneRegistryInner};
use bevy_vrm::BoneName;
use loro::TreeID;

/// Per-script doc + bone-node map + scene registry for one agent-script pair.
pub struct AgentDocEntry {
    pub doc: Arc<loro::LoroDoc>,
    /// Maps each VRM bone to its proxy root node in `doc`.
    pub bone_nodes: Arc<HashMap<BoneName, TreeID>>,
    /// Per-script scene registry (shared with `WiredSceneRt`).
    pub registry: Arc<SceneRegistryInner>,
}

/// Bevy marker for `HsdDoc` entities that are agent docs (one per script).
#[derive(Component)]
pub struct AgentHsdDoc;

/// Marker on HSD proxy-node entities that are parented to VRM bones.
#[derive(Component)]
pub struct BoneProxy;

/// Inserted by unavi-client when `AvatarBonesPopulated` fires.
#[derive(Resource)]
pub struct LocalAgentDocs {
    /// Maps each VRM bone name to its Bevy entity (set by unavi-client).
    pub bone_entities: Arc<HashMap<BoneName, Entity>>,
    pub docs: Arc<Mutex<Vec<Arc<AgentDocEntry>>>>,
}

/// Parents newly-hydrated HSD proxy-node entities to their VRM bone entities.
pub(crate) fn parent_bone_proxies(
    mut commands: Commands,
    agent_doc_entities: Query<(Entity, &HsdDoc, &bevy_hsd::SceneRegistry), With<AgentHsdDoc>>,
    agent_docs: Option<Res<LocalAgentDocs>>,
    new_nodes: Query<(Entity, &HsdNodeTreeId), Added<HsdNodeTreeId>>,
) {
    let Some(ad) = agent_docs else { return };

    for (node_ent, tree_id_comp) in &new_nodes {
        let tree_id_str = tree_id_comp.0.as_str();
        let Ok(tid) = loro::TreeID::try_from(tree_id_str) else {
            continue;
        };

        // Find the agent doc that owns this node.
        let found = agent_doc_entities
            .iter()
            .find_map(|(_, hsd_doc, registry)| {
                let node_map = registry.0.node_map.lock().expect("node_map lock");
                if node_map.contains_key(&tid) {
                    Some(Arc::clone(&hsd_doc.0))
                } else {
                    None
                }
            });
        let Some(doc) = found else { continue };

        let hsd = doc.get_map("hsd");
        let Ok(tree) = hsd.get_or_create_container("nodes", loro::LoroTree::new()) else {
            continue;
        };

        // Only parent root-level proxy nodes.
        let Some(loro::TreeParentId::Root) = tree.parent(tid) else {
            continue;
        };

        let Ok(meta) = tree.get_meta(tid) else {
            continue;
        };
        let Some(loro::ValueOrContainer::Value(loro::LoroValue::String(bone_name_str))) =
            meta.get("bone_name")
        else {
            continue;
        };

        let Some((_, &bone_ent)) = ad
            .bone_entities
            .iter()
            .find(|(b, _)| format!("{b}").trim_matches('"') == bone_name_str.as_str())
        else {
            continue;
        };

        commands
            .entity(node_ent)
            .insert((BoneProxy, ChildOf(bone_ent), Transform::IDENTITY));
    }
}

/// Resets proxy node `Transform` to IDENTITY each `PostUpdate` so that Bevy's
/// transform propagation uses the parent bone's world transform.
pub fn reset_bone_proxies(mut proxies: Query<&mut Transform, With<BoneProxy>>) {
    for mut t in &mut proxies {
        *t = Transform::IDENTITY;
    }
}
