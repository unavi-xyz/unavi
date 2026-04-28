use std::collections::BTreeMap;

use loro_surgeon::{Hydrate, Reconcile};
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
#[derive(Debug, Clone, Default, Hydrate, Reconcile, Serialize, Deserialize)]
pub struct Hsd {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub assets: BTreeMap<SmolStr, HydratedHash>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub images: BTreeMap<SmolStr, HsdImage>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub materials: BTreeMap<SmolStr, HsdMaterial>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub meshes: BTreeMap<SmolStr, HsdMesh>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub nodes: BTreeMap<SmolStr, HsdNode>,
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
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub nodes: BTreeMap<SmolStr, HsdxNode>,
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
#[skip_serializing_none]
pub struct HsdImageBase<T>
where
    T: Hydrate + Reconcile,
{
    #[loro(default)]
    pub address_mode_u: Option<i64>,
    #[loro(default)]
    pub address_mode_v: Option<i64>,
    #[loro(default)]
    pub address_mode_w: Option<i64>,
    #[loro(default)]
    pub data: Option<T>,
    #[loro(default)]
    pub mag_filter: Option<i64>,
    #[loro(default)]
    pub min_filter: Option<i64>,
    #[loro(default)]
    pub mipmap_filter: Option<i64>,
    #[loro(default)]
    pub name: Option<SmolStr>,
    #[loro(default)]
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
#[skip_serializing_none]
pub struct HsdMaterial {
    #[loro(default)]
    pub alpha_cutoff: Option<f64>,
    #[loro(default)]
    pub alpha_mode: Option<SmolStr>,
    #[loro(default)]
    pub base_color: Option<Vec<f64>>,
    #[loro(default)]
    pub base_color_texture: Option<SmolStr>,
    #[loro(default)]
    pub double_sided: Option<bool>,
    #[loro(default)]
    pub emissive: Option<Vec<f64>>,
    #[loro(default)]
    pub emissive_texture: Option<SmolStr>,
    #[loro(default)]
    pub metallic: Option<f64>,
    #[loro(default)]
    pub metallic_roughness_texture: Option<SmolStr>,
    #[loro(default)]
    pub name: Option<SmolStr>,
    #[loro(default)]
    pub normal_texture: Option<SmolStr>,
    #[loro(default)]
    pub occlusion_texture: Option<SmolStr>,
    #[loro(default)]
    pub roughness: Option<f64>,
    #[loro(default)]
    pub unlit: Option<bool>,
}

#[derive(Debug, Clone, Default, Hydrate, Reconcile, Serialize, Deserialize)]
#[skip_serializing_none]
pub struct HsdNodeBase<T>
where
    T: Hydrate + Reconcile,
{
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
    pub scripts: Option<Vec<T>>,
    #[loro(default)]
    pub translation: Option<Vec<f64>>,
}

impl HsdNode {
    #[must_use]
    pub fn from_hsdx(value: HsdxNode, scripts: Option<Vec<HydratedHash>>) -> Self {
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
pub struct HsdMesh {
    #[loro(default)]
    pub attributes: BTreeMap<SmolStr, HydratedHash>,
    #[loro(default)]
    pub indices: Option<HydratedHash>,
    #[loro(default)]
    pub name: Option<SmolStr>,
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
