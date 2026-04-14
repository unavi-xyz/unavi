use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// HSD/HSDX file format (RON).
///
/// Shared between author-time (`.hsdx`) and compiled client-ready (`.hsd`).
/// String refs in `scripts` and `assets` are relative paths:
/// - `.hsdx`: `./Cargo.toml`, `../dep/asset.hsdx`
/// - `.hsd`:  `./foo.wasm`,   `./dep.hsd`
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HsdFile {
    #[serde(default)]
    pub assets: BTreeMap<String, String>,
    #[serde(default)]
    pub images: BTreeMap<String, HsdImageDef>,
    #[serde(default)]
    pub materials: BTreeMap<String, HsdMaterialDef>,
    #[serde(default)]
    pub nodes: BTreeMap<String, HsdNodeDef>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HsdImageDef {
    pub path: String,
    #[serde(default)]
    pub address_mode_u: Option<i64>,
    #[serde(default)]
    pub address_mode_v: Option<i64>,
    #[serde(default)]
    pub address_mode_w: Option<i64>,
    #[serde(default)]
    pub mag_filter: Option<i64>,
    #[serde(default)]
    pub min_filter: Option<i64>,
    #[serde(default)]
    pub mipmap_filter: Option<i64>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub srgb: Option<bool>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HsdMaterialDef {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub base_color: Option<Vec<f64>>,
    #[serde(default)]
    pub base_color_texture: Option<String>,
    #[serde(default)]
    pub roughness: Option<f64>,
    #[serde(default)]
    pub metallic: Option<f64>,
    #[serde(default)]
    pub alpha_cutoff: Option<f64>,
    #[serde(default)]
    pub alpha_mode: Option<String>,
    #[serde(default)]
    pub double_sided: Option<bool>,
    #[serde(default)]
    pub emissive: Option<Vec<f64>>,
    #[serde(default)]
    pub emissive_texture: Option<String>,
    #[serde(default)]
    pub metallic_roughness_texture: Option<String>,
    #[serde(default)]
    pub normal_texture: Option<String>,
    #[serde(default)]
    pub occlusion_texture: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HsdNodeDef {
    #[serde(default)]
    pub scripts: Vec<String>,
}

impl HsdFile {
    pub fn parse(s: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(s)
    }

    pub fn to_ron(&self) -> Result<String, ron::Error> {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
    }
}
