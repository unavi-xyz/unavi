use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, Default, Serialize, Deserialize)]
#[skip_serializing_none]
pub struct Hsd {
    pub assets: BTreeMap<String, String>,
    pub images: BTreeMap<String, HsdImage>,
    pub materials: BTreeMap<String, HsdMaterial>,
    pub nodes: BTreeMap<String, HsdNode>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[skip_serializing_none]
pub struct HsdImage {
    pub path: String,
    pub address_mode_u: Option<i64>,
    pub address_mode_v: Option<i64>,
    pub address_mode_w: Option<i64>,
    pub mag_filter: Option<i64>,
    pub min_filter: Option<i64>,
    pub mipmap_filter: Option<i64>,
    pub name: Option<String>,
    pub srgb: Option<bool>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[skip_serializing_none]
pub struct HsdMaterial {
    pub name: Option<String>,
    pub base_color: Option<Vec<f64>>,
    pub base_color_texture: Option<String>,
    pub roughness: Option<f64>,
    pub metallic: Option<f64>,
    pub alpha_cutoff: Option<f64>,
    pub alpha_mode: Option<String>,
    pub double_sided: Option<bool>,
    pub emissive: Option<Vec<f64>>,
    pub emissive_texture: Option<String>,
    pub metallic_roughness_texture: Option<String>,
    pub normal_texture: Option<String>,
    pub occlusion_texture: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[skip_serializing_none]
pub struct HsdNode {
    pub scripts: Vec<String>,
}

impl Hsd {
    pub fn parse(s: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(s)
    }

    pub fn to_ron(&self) -> Result<String, ron::Error> {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
    }
}
