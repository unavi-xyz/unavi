use std::collections::BTreeMap;

use loro::{LoroMap, LoroTree, TreeParentId};
use loro_surgeon::{Hydrate, Reconcile, ReconcileError};

use loro_surgeon::TreeNode;
use ron::extensions::Extensions;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use smol_str::SmolStr;
use topology::HydratedTopology;
use wired_records::HydratedHash;

pub mod topology;

fn ron_options() -> ron::Options {
    ron::Options::default()
        .with_default_extension(Extensions::IMPLICIT_SOME)
        .with_default_extension(Extensions::UNWRAP_NEWTYPES)
}

#[skip_serializing_none]
#[derive(Debug, Clone, Default, Hydrate, Serialize, Deserialize)]
pub struct Hsd {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub assets: BTreeMap<SmolStr, HydratedHash>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub images: BTreeMap<SmolStr, HsdImage>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub materials: BTreeMap<SmolStr, HsdMaterial>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub meshes: BTreeMap<SmolStr, HsdMesh>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<TreeNode<HsdNode>>,
}

impl Reconcile for Hsd {
    fn reconcile(&self, map: &LoroMap) -> Result<(), ReconcileError> {
        self.assets.reconcile_field(map, "assets")?;
        self.images.reconcile_field(map, "images")?;
        self.materials.reconcile_field(map, "materials")?;
        self.meshes.reconcile_field(map, "meshes")?;
        let tree = map.get_or_create_container("nodes", LoroTree::new())?;
        for id in tree.roots() {
            tree.delete(id)?;
        }
        for node in &self.nodes {
            node.insert_into(&tree, TreeParentId::Root)?;
        }
        Ok(())
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        let nested = map.get_or_create_container(key, LoroMap::new())?;
        self.reconcile(&nested)
    }
}

impl Hsd {
    pub fn parse(s: &str) -> Result<Self, ron::error::SpannedError> {
        ron_options().from_str(s)
    }

    pub fn to_ron(&self) -> Result<String, ron::Error> {
        ron_options().to_string_pretty(self, ron::ser::PrettyConfig::default())
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Hsdx {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub assets: BTreeMap<SmolStr, SmolStr>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub images: BTreeMap<SmolStr, HsdxImage>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub materials: BTreeMap<SmolStr, HsdMaterial>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub meshes: BTreeMap<SmolStr, HsdMesh>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<TreeNode<HsdxNode>>,
}

impl Hsdx {
    pub fn parse(s: &str) -> Result<Self, ron::error::SpannedError> {
        ron_options().from_str(s)
    }

    pub fn to_ron(&self) -> Result<String, ron::Error> {
        ron_options().to_string_pretty(self, ron::ser::PrettyConfig::default())
    }
}

pub type HsdImage = HsdImageBase<HydratedHash>;
pub type HsdxImage = HsdImageBase<String>;

pub type HsdNode = HsdNodeBase<HydratedHash>;
pub type HsdxNode = HsdNodeBase<String>;

#[derive(Debug, Clone, Default, Hydrate, Reconcile, Serialize, Deserialize)]
#[loro(default)]
#[skip_serializing_none]
pub struct HsdImageBase<T>
where
    T: Hydrate + Reconcile,
{
    pub address_mode_u: Option<i64>,
    pub address_mode_v: Option<i64>,
    pub address_mode_w: Option<i64>,
    pub data: Option<T>,
    pub mag_filter: Option<i64>,
    pub min_filter: Option<i64>,
    pub mipmap_filter: Option<i64>,
    pub name: Option<SmolStr>,
    pub srgb: Option<bool>,
}

impl HsdImage {
    #[must_use]
    pub fn from_hsdx(value: HsdxImage, data: Option<HydratedHash>) -> Self {
        Self {
            address_mode_u: value.address_mode_u,
            address_mode_v: value.address_mode_v,
            address_mode_w: value.address_mode_w,
            data,
            mag_filter: value.mag_filter,
            min_filter: value.min_filter,
            mipmap_filter: value.mipmap_filter,
            name: value.name,
            srgb: value.srgb,
        }
    }
}

#[derive(Debug, Clone, Default, Hydrate, Reconcile, Serialize, Deserialize)]
#[loro(default)]
#[skip_serializing_none]
pub struct HsdMaterial {
    pub alpha_cutoff: Option<f64>,
    pub alpha_mode: Option<SmolStr>,
    pub base_color: Option<Vec<f64>>,
    pub base_color_texture: Option<SmolStr>,
    pub double_sided: Option<bool>,
    pub emissive: Option<Vec<f64>>,
    pub emissive_texture: Option<SmolStr>,
    pub metallic: Option<f64>,
    pub metallic_roughness_texture: Option<SmolStr>,
    pub name: Option<SmolStr>,
    pub normal_texture: Option<SmolStr>,
    pub occlusion_texture: Option<SmolStr>,
    pub roughness: Option<f64>,
    pub unlit: Option<bool>,
}

#[derive(Debug, Clone, Default, Hydrate, Reconcile, Serialize, Deserialize)]
#[loro(default)]
#[skip_serializing_none]
pub struct HsdNodeBase<T>
where
    T: Hydrate + Reconcile,
{
    pub collider: Option<HsdCollider>,
    pub material: Option<SmolStr>,
    pub mesh: Option<SmolStr>,
    pub name: Option<SmolStr>,
    pub rigid_body: Option<HsdRigidBody>,
    pub rotation: Option<Vec<f64>>,
    pub scale: Option<Vec<f64>>,
    #[serde(default)]
    pub scripts: Vec<T>,
    pub translation: Option<Vec<f64>>,
}

impl HsdNode {
    #[must_use]
    pub fn from_hsdx(value: HsdxNode, scripts: Vec<HydratedHash>) -> Self {
        Self {
            collider: value.collider,
            material: value.material,
            mesh: value.mesh,
            name: value.name,
            rigid_body: value.rigid_body,
            rotation: value.rotation,
            scale: value.scale,
            scripts,
            translation: value.translation,
        }
    }
}

#[derive(Debug, Clone, Hydrate, Reconcile, Serialize, Deserialize)]
#[loro(default)]
pub struct HsdMesh {
    pub attributes: BTreeMap<SmolStr, HydratedHash>,
    pub indices: Option<HydratedHash>,
    pub name: Option<SmolStr>,
    #[loro(required)]
    pub topology: HydratedTopology,
}

#[derive(Debug, Clone, Hydrate, Reconcile, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Hydrate, Reconcile, Serialize, Deserialize, Default)]
#[loro(default)]
pub struct HsdRigidBody {
    pub angular_damping: Option<f64>,
    pub friction: Option<f64>,
    #[loro(required)]
    pub kind: SmolStr,
    pub linear_damping: Option<f64>,
    pub mass: Option<f64>,
    pub restitution: Option<f64>,
}
