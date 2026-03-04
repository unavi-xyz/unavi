use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bevy::prelude::*;
use bevy_hsd::{HsdDoc, HsdEntityMap, HsdNodeTreeId};
use bevy_vrm::BoneName;
use loro::{LoroDoc, LoroMap, LoroTree, LoroValue, TreeID};

/// Per-script doc + bone-node map for one agent-script pair.
pub struct AgentDocEntry {
    pub doc: Arc<LoroDoc>,
    /// Maps each VRM bone to its proxy root node in `doc`.
    pub bone_nodes: Arc<HashMap<BoneName, TreeID>>,
}

/// Bevy marker for `HsdDoc` entities that are agent docs (one per script).
/// Defined here so both unavi-script (creates them) and unavi-client
/// (parents bone proxies) can use it without a circular dep.
#[derive(Component)]
pub struct AgentHsdDoc;

/// Marker on HSD proxy-node entities that are parented to VRM bones.
/// Used to reset Transform to IDENTITY after HSD may overwrite it.
#[derive(Component)]
pub struct BoneProxy;

/// Inserted by unavi-client when `AvatarBonesPopulated` fires.
/// `load_scripts` reads `bone_entities` to create per-script docs and
/// pushes new entries into `docs` for transform-sync.
#[derive(Resource)]
pub struct LocalAgentDocs {
    /// Maps each VRM bone name to its Bevy entity (set by unavi-client).
    pub bone_entities: Arc<HashMap<BoneName, Entity>>,
    pub docs: Arc<Mutex<Vec<Arc<AgentDocEntry>>>>,
}

/// Parents newly-hydrated HSD proxy-node entities to their VRM bone entities.
/// Only root-level nodes (bone proxies) in agent docs are reparented.
pub fn parent_bone_proxies(
    mut commands: Commands,
    agent_doc_entities: Query<(Entity, &HsdDoc), With<AgentHsdDoc>>,
    agent_docs: Option<Res<LocalAgentDocs>>,
    new_nodes: Query<(Entity, &HsdNodeTreeId), Added<HsdNodeTreeId>>,
    entity_maps: Query<&HsdEntityMap>,
) {
    let Some(ad) = agent_docs else { return };

    for (node_ent, tree_id_comp) in &new_nodes {
        let tree_id_str = tree_id_comp.0.as_str();

        // Find the agent doc that owns this node.
        let agent_doc = agent_doc_entities.iter().find_map(|(agent_ent, hsd_doc)| {
            let emap = entity_maps.get(agent_ent).ok()?;
            if emap.nodes.contains_key(tree_id_str) {
                Some(Arc::clone(&hsd_doc.0))
            } else {
                None
            }
        });
        let Some(doc) = agent_doc else { continue };

        let Ok(tree_id) = loro::TreeID::try_from(tree_id_str) else {
            continue;
        };
        let hsd = doc.get_map("hsd");
        let Ok(tree) = hsd.get_or_create_container("nodes", LoroTree::new()) else {
            continue;
        };

        // Only parent root-level proxy nodes (bone proxies, not script nodes).
        let Some(loro::TreeParentId::Root) = tree.parent(tree_id) else {
            continue;
        };

        let Ok(meta) = tree.get_meta(tree_id) else {
            continue;
        };
        let Some(loro::ValueOrContainer::Value(LoroValue::String(bone_name_str))) =
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

/// Writes VRM bone `GlobalTransform`s into each agent doc's proxy nodes.
/// Runs `PreUpdate` so transforms are committed during the script tick.
pub(crate) fn sync_agent_bone_transforms(
    agent_docs: Option<Res<LocalAgentDocs>>,
    bone_transforms: Query<&GlobalTransform>,
) {
    let Some(ad) = agent_docs else { return };

    let docs = ad.docs.lock().expect("lock agent doc");
    for entry in docs.iter() {
        let hsd = entry.doc.get_map("hsd");
        let Ok(tree) = hsd.get_or_create_container("nodes", LoroTree::new()) else {
            continue;
        };

        for (bone_name, &tree_id) in entry.bone_nodes.iter() {
            let Some(&bone_ent) = ad.bone_entities.get(bone_name) else {
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
    }
}

/// Resets proxy node `Transform` to IDENTITY each `PostUpdate` so that Bevy's
/// transform propagation uses the parent bone's world transform, not any
/// local offset written by HSD's `update_node_components`.
pub fn reset_bone_proxies(mut proxies: Query<&mut Transform, With<BoneProxy>>) {
    for mut t in &mut proxies {
        *t = Transform::IDENTITY;
    }
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
