use std::collections::BTreeMap;

use bevy::prelude::*;
use loro::LoroValue;
use loro_surgeon::{Hydrate, HydrateError, Reconcile};
use smol_str::SmolStr;
use wired_schemas::HydratedHash;

use crate::hydration::topology::HydratedTopology;

#[derive(Component, Debug, Clone, Default, Hydrate, Reconcile)]
pub struct HsdMaterial {
    #[loro(default)]
    pub alpha_cutoff: Option<f64>,
    #[loro(default)]
    pub alpha_mode: Option<SmolStr>,
    #[loro(default)]
    pub base_color: Option<Vec<f64>>,
    #[loro(default)]
    pub base_color_texture: Option<HydratedHash>,
    #[loro(default)]
    pub double_sided: Option<bool>,
    #[loro(default)]
    pub emissive: Option<Vec<f64>>,
    #[loro(default)]
    pub emissive_texture: Option<HydratedHash>,
    #[loro(default)]
    pub metallic: Option<f64>,
    #[loro(default)]
    pub metallic_roughness_texture: Option<HydratedHash>,
    #[loro(default)]
    pub name: Option<SmolStr>,
    #[loro(default)]
    pub normal_texture: Option<HydratedHash>,
    #[loro(default)]
    pub occlusion_texture: Option<HydratedHash>,
    #[loro(default)]
    pub roughness: Option<f64>,
}

#[derive(Component, Debug, Clone, Default, Hydrate, Reconcile)]
pub struct HsdMesh {
    #[loro(default)]
    pub attributes: BTreeMap<SmolStr, HydratedHash>,
    #[loro(default)]
    pub indices: Option<HydratedHash>,
    #[loro(default)]
    pub name: Option<SmolStr>,
    pub topology: HydratedTopology,
}

#[derive(Debug, Clone, Default, Hydrate, Reconcile)]
pub struct HsdNodeData {
    #[loro(default)]
    pub collider: Option<HsdCollider>,
    #[loro(default)]
    pub material: Option<i64>,
    #[loro(default)]
    pub mesh: Option<i64>,
    #[loro(default)]
    pub name: Option<SmolStr>,
    #[loro(default)]
    pub rigid_body: Option<HsdRigidBody>,
    #[loro(default)]
    pub rotation: Option<Vec<f64>>,
    #[loro(default)]
    pub scale: Option<Vec<f64>>,
    #[loro(default)]
    pub scripts: Option<Vec<HydratedHash>>,
    #[loro(default)]
    pub translation: Option<Vec<f64>>,
}

#[derive(Component, Debug, Clone, Default, Hydrate, Reconcile)]
pub struct HsdCollider {
    pub shape: SmolStr,
    pub size: Vec<f64>,
}

#[derive(Component, Debug, Clone, Default, Hydrate, Reconcile)]
pub struct HsdRigidBody {
    #[loro(default)]
    pub angular_damping: Option<f64>,
    #[loro(default)]
    pub friction: Option<f64>,
    pub kind: SmolStr,
    #[loro(default)]
    pub linear_damping: Option<f64>,
    #[loro(default)]
    pub mass: Option<f64>,
    #[loro(default)]
    pub restitution: Option<f64>,
}

/// Full hydrated HSD document data.
pub struct HsdData {
    pub materials: Vec<HsdMaterial>,
    pub meshes: Vec<HsdMesh>,
    pub nodes: Vec<HsdNode>,
}

/// A hydrated tree node with its parent info.
pub struct HsdNode {
    pub tree_id: String,
    pub parent_tree_id: Option<String>,
    pub data: HsdNodeData,
}

/// Hydrate an HSD document from a `LoroMap` (the "hsd" map).
pub fn hydrate_hsd(map: &loro::LoroMap) -> Result<HsdData, HydrateError> {
    let value = map.get_deep_value();
    let LoroValue::Map(root) = &value else {
        return Err(HydrateError::TypeMismatch {
            expected: "map".into(),
            actual: format!("{value:?}").into(),
        });
    };

    // Hydrate materials list.
    let materials = match root.get("materials") {
        Some(v) => Vec::<HsdMaterial>::hydrate(v)?,
        None => Vec::new(),
    };

    // Hydrate meshes list.
    let meshes = match root.get("meshes") {
        Some(v) => Vec::<HsdMesh>::hydrate(v)?,
        None => Vec::new(),
    };

    // Hydrate nodes from the tree container.
    let tree = map
        .get_or_create_container("nodes", loro::LoroTree::new())
        .map_err(|e| HydrateError::Custom(format!("failed to get tree: {e}").into()))?;

    let tree_nodes = tree.get_nodes(false);
    let mut nodes = Vec::with_capacity(tree_nodes.len());

    for tree_node in &tree_nodes {
        let meta = tree
            .get_meta(tree_node.id)
            .map_err(|e| HydrateError::Custom(format!("failed to get meta: {e}").into()))?;
        let meta_value = meta.get_deep_value();
        let data = HsdNodeData::hydrate(&meta_value)?;

        let parent_tree_id = match tree_node.parent {
            loro::TreeParentId::Node(pid) => Some(format!("{}@{}", pid.counter, pid.peer)),
            _ => None,
        };

        nodes.push(HsdNode {
            tree_id: format!("{}@{}", tree_node.id.counter, tree_node.id.peer),
            parent_tree_id,
            data,
        });
    }

    Ok(HsdData {
        materials,
        meshes,
        nodes,
    })
}
