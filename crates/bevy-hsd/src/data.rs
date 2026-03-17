use std::collections::BTreeMap;

use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use loro::{LoroMap, LoroTree, LoroValue, TreeID, TreeParentId};
use loro_surgeon::{Hydrate, HydrateError, Reconcile, ReconcileError};
use smol_str::{SmolStr, ToSmolStr};
use wired_schemas::HydratedHash;

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
    pub material: Option<SmolStr>,
    #[loro(default)]
    pub mesh: Option<SmolStr>,
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

#[derive(Component, Debug, Clone, Hydrate, Reconcile)]
pub enum HsdCollider {
    Capsule {
        height: f64,
        radius: f64,
    },
    ConvexHull(HydratedHash),
    Cuboid {
        x: f64,
        y: f64,
        z: f64,
    },
    Cylinder {
        height: f64,
        radius: f64,
    },
    Sphere(f64),
    Trimesh {
        indices: HydratedHash,
        vertices: HydratedHash,
    },
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

pub struct HsdData {
    pub materials: BTreeMap<SmolStr, HsdMaterial>,
    pub meshes: BTreeMap<SmolStr, HsdMesh>,
    pub nodes: BTreeMap<TreeID, HsdNode>,
}

pub struct HsdNode {
    pub id: SmolStr,
    pub parent_id: Option<SmolStr>,
    pub data: HsdNodeData,
}

pub(crate) fn hydrate_hsd(map: &LoroMap) -> Result<HsdData, HydrateError> {
    let value = map.get_deep_value();
    let LoroValue::Map(root) = &value else {
        return Err(HydrateError::TypeMismatch {
            expected: "map".into(),
            actual: format!("{value:?}").into(),
        });
    };

    let materials = match root.get("materials") {
        Some(v) => BTreeMap::<SmolStr, HsdMaterial>::hydrate(v)?,
        None => BTreeMap::new(),
    };

    let meshes = match root.get("meshes") {
        Some(v) => BTreeMap::<SmolStr, HsdMesh>::hydrate(v)?,
        None => BTreeMap::new(),
    };

    let tree = map
        .get_or_create_container("nodes", LoroTree::new())
        .map_err(|e| HydrateError::Custom(format!("failed to get tree: {e}").into()))?;

    let tree_nodes = tree.get_nodes(false);
    let mut nodes = BTreeMap::new();

    for tree_node in &tree_nodes {
        let meta = tree
            .get_meta(tree_node.id)
            .map_err(|e| HydrateError::Custom(format!("failed to get meta: {e}").into()))?;
        let meta_value = meta.get_deep_value();
        let data = HsdNodeData::hydrate(&meta_value)?;

        let parent_tree_id = match tree_node.parent {
            TreeParentId::Node(pid) => Some(pid.to_smolstr()),
            _ => None,
        };

        nodes.insert(
            tree_node.id,
            HsdNode {
                id: tree_node.id.to_smolstr(),
                parent_id: parent_tree_id,
                data,
            },
        );
    }

    Ok(HsdData {
        materials,
        meshes,
        nodes,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HydratedTopology(pub PrimitiveTopology);

impl Default for HydratedTopology {
    fn default() -> Self {
        Self(PrimitiveTopology::TriangleList)
    }
}

impl Hydrate for HydratedTopology {
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
        let LoroValue::I64(num) = value else {
            return Err(HydrateError::TypeMismatch {
                expected: "i64".into(),
                actual: format!("{value:?}").into(),
            });
        };

        let inner = match num {
            0 => PrimitiveTopology::PointList,
            1 => PrimitiveTopology::LineList,
            2 => PrimitiveTopology::LineStrip,
            3 => PrimitiveTopology::TriangleList,
            4 => PrimitiveTopology::TriangleStrip,
            other => {
                return Err(HydrateError::TypeMismatch {
                    expected: "valid topology".into(),
                    actual: format!("{other}").into(),
                });
            }
        };

        Ok(Self(inner))
    }
}

impl Reconcile for HydratedTopology {
    fn reconcile(&self, _map: &LoroMap) -> Result<(), ReconcileError> {
        Err(ReconcileError::Custom(
            "HydratedTopology cannot be reconciled as a root container".into(),
        ))
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        let num: i64 = match self.0 {
            PrimitiveTopology::PointList => 0,
            PrimitiveTopology::LineList => 1,
            PrimitiveTopology::LineStrip => 2,
            PrimitiveTopology::TriangleList => 3,
            PrimitiveTopology::TriangleStrip => 4,
        };
        map.insert(key, num)?;
        Ok(())
    }

    fn to_loro_value(&self) -> Option<LoroValue> {
        let num: i64 = match self.0 {
            PrimitiveTopology::PointList => 0,
            PrimitiveTopology::LineList => 1,
            PrimitiveTopology::LineStrip => 2,
            PrimitiveTopology::TriangleList => 3,
            PrimitiveTopology::TriangleStrip => 4,
        };
        Some(LoroValue::I64(num))
    }
}
