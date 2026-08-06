//! `.hsda`: the authoring tier, and the only hand-written one.
//!
//! Human names, relative paths to script crates, images and nested prefabs.
//! Compilation replaces every path with content, so this is source, not a
//! round-trip of the compiled package.

use std::collections::BTreeMap;

use ron::extensions::Extensions;
use serde::{
    Deserialize,
    Serialize,
};
use serde_with::skip_serializing_none;

use crate::attributes::material_graph::value::GraphValue;

pub const EXTENSION: &str = "hsda";

fn ron_options() -> ron::Options {
    ron::Options::default()
        .with_default_extension(Extensions::IMPLICIT_SOME)
        .with_default_extension(Extensions::UNWRAP_NEWTYPES)
}

#[derive(Serialize, Deserialize, Default)]
pub struct Source(pub Vec<SourcePrim>);

impl Source {
    pub fn parse(s: &str) -> Result<Self, ron::error::SpannedError> {
        ron_options().from_str(s)
    }

    pub fn to_ron(&self) -> Result<String, ron::Error> {
        ron_options().to_string_pretty(self, ron::ser::PrettyConfig::default())
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SourcePrim {
    pub attributes:    SourceAttributes,
    /// Cross-prim references by prim name, resolved to ids at build time.
    pub relationships: BTreeMap<String, String>,
    pub children:      Vec<Self>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SourceAttributes {
    pub collider:       Option<SourceCollider>,
    pub gravity_scale:  Option<f64>,
    pub image:          Option<SourceImage>,
    pub material:       Option<SourceMaterial>,
    pub material_graph: Option<SourceMaterialGraph>,
    pub name:           Option<String>,
    /// Path to another `.hsda`, compiled and inlined as a nested package.
    pub prefab:         Option<String>,
    pub rigid_body:     Option<SourceRigidBody>,
    /// Path to a wasm crate's `Cargo.toml`.
    pub script:         Option<String>,
    pub spawn:          Option<SourceSpawn>,
    pub xform:          Option<SourceXform>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SourceSpawn {
    pub radius: f64,
}

#[derive(Serialize, Deserialize)]
pub enum SourceCollider {
    Capsule { height: f64, radius: f64 },
    Cuboid { x: f64, y: f64, z: f64 },
    Cylinder { height: f64, radius: f64 },
    Sphere(f64),
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SourceImage {
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
pub struct SourceMaterial {
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

/// Path to a `.hss` file, compiled into the `material:graph_data` slot
/// entry, plus optional per-instance overrides of the graph's public inputs.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SourceMaterialGraph {
    pub path:      String,
    pub overrides: BTreeMap<u16, GraphValue>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SourceRigidBody {
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
pub struct SourceXform {
    pub translation: Option<Vec<f32>>,
    pub rotation:    Option<Vec<f32>>,
    pub scale:       Option<Vec<f32>>,
}
