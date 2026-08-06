//! `.hsda`: the authoring tier, and the only hand-written one.
//!
//! Human names, relative paths to script crates, images and nested prefabs.
//! Compilation replaces every path with content, so this is source, not a
//! round-trip of the compiled package.

use std::collections::BTreeMap;

use anyhow::Result;
use hsd::attributes::material_graph::GraphValue;
use ron::extensions::Extensions;
use serde::{
    Deserialize,
    Serialize,
};
use serde_with::skip_serializing_none;

pub const EXTENSION: &str = "hsda";

fn ron_options() -> ron::Options {
    ron::Options::default()
        .with_default_extension(Extensions::IMPLICIT_SOME)
        .with_default_extension(Extensions::UNWRAP_NEWTYPES)
}

#[derive(Serialize, Deserialize, Default)]
pub struct Hsda(pub Vec<HsdaPrim>);

impl Hsda {
    pub fn parse(s: &str) -> Result<Self, ron::error::SpannedError> {
        ron_options().from_str(s)
    }

    pub fn to_ron(&self) -> Result<String, ron::Error> {
        ron_options().to_string_pretty(self, ron::ser::PrettyConfig::default())
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HsdaPrim {
    pub attributes:    HsdaAttributes,
    /// Cross-prim references by prim name, resolved to ids at build time.
    pub relationships: BTreeMap<String, String>,
    pub children:      Vec<Self>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HsdaAttributes {
    pub collider:       Option<HsdaCollider>,
    pub gravity_scale:  Option<f64>,
    pub image:          Option<HsdaImage>,
    pub material:       Option<HsdaMaterial>,
    pub material_graph: Option<HsdaMaterialGraph>,
    pub name:           Option<String>,
    /// Path to another `.hsda`, compiled and inlined as a nested package.
    pub prefab:         Option<String>,
    pub rigid_body:     Option<HsdaRigidBody>,
    /// Path to a wasm crate's `Cargo.toml`.
    pub script:         Option<String>,
    pub spawn:          Option<HsdaSpawn>,
    pub xform:          Option<HsdaXform>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HsdaSpawn {
    pub radius: f64,
}

#[derive(Serialize, Deserialize)]
pub enum HsdaCollider {
    Capsule { height: f64, radius: f64 },
    Cuboid { x: f64, y: f64, z: f64 },
    Cylinder { height: f64, radius: f64 },
    Sphere(f64),
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HsdaImage {
    pub data:           String,
    pub address_mode_u: Option<i64>,
    pub address_mode_v: Option<i64>,
    pub address_mode_w: Option<i64>,
    pub mag_filter:     Option<i64>,
    pub min_filter:     Option<i64>,
    pub mipmap_filter:  Option<i64>,
    pub srgb:           Option<bool>,
}

/// Texture fields name a prim; they compile to relationship properties, not
/// into the material payload.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HsdaMaterial {
    pub alpha_cutoff:               Option<f64>,
    pub alpha_mode:                 Option<String>,
    pub base_color:                 Option<Vec<f64>>,
    pub base_color_texture:         Option<String>,
    pub double_sided:               Option<bool>,
    pub emissive:                   Option<Vec<f64>>,
    pub emissive_texture:           Option<String>,
    pub metallic:                   Option<f64>,
    pub metallic_roughness_texture: Option<String>,
    pub normal_texture:             Option<String>,
    pub occlusion_texture:          Option<String>,
    pub roughness:                  Option<f64>,
}

/// Path to a `.shader` file, compiled into the `material:graph_data` slot
/// entry, plus optional per-instance overrides of the graph's public inputs.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HsdaMaterialGraph {
    pub path:      String,
    pub overrides: BTreeMap<u16, GraphValue>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HsdaRigidBody {
    pub kind:            String,
    pub angular_damping: Option<f64>,
    pub friction:        Option<f64>,
    pub linear_damping:  Option<f64>,
    pub mass:            Option<f64>,
    pub restitution:     Option<f64>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HsdaXform {
    pub translation: Option<Vec<f32>>,
    pub rotation:    Option<Vec<f32>>,
    pub scale:       Option<Vec<f32>>,
}
