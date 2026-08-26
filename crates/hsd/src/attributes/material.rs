use serde::{
    Deserialize,
    Serialize,
};

use crate::attributes::Attribute;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ColorVec(pub Vec<f64>);

/// Texture slots are relationship properties (`material:base_color_texture`
/// and friends), not fields of this payload.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MaterialAttr {
    pub alpha_cutoff: Option<f64>,
    pub alpha_mode:   Option<String>,
    pub base_color:   Option<ColorVec>,
    pub double_sided: Option<bool>,
    pub emissive:     Option<ColorVec>,
    pub metallic:     Option<f64>,
    pub roughness:    Option<f64>,
}

impl Attribute for MaterialAttr {
    const KEY: &'static str = "material";
}

/// The relationship carrying a prim's material from another prim.
pub const BINDING: &str = "material:binding";

pub const BASE_COLOR_TEXTURE: &str = "material:base_color_texture";
pub const EMISSIVE_TEXTURE: &str = "material:emissive_texture";
pub const METALLIC_ROUGHNESS_TEXTURE: &str = "material:metallic_roughness_texture";
pub const NORMAL_TEXTURE: &str = "material:normal_texture";
pub const OCCLUSION_TEXTURE: &str = "material:occlusion_texture";

pub const TEXTURES: [&str; 5] = [
    BASE_COLOR_TEXTURE,
    EMISSIVE_TEXTURE,
    METALLIC_ROUGHNESS_TEXTURE,
    NORMAL_TEXTURE,
    OCCLUSION_TEXTURE,
];
