use const_format::concatcp;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    prop_name,
    property::{
        Property,
        name::PropName,
    },
    relationship,
};

pub const GROUP: &str = "material";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ColorVec(pub Vec<f64>);

/// How a material composites against what is already drawn.
///
/// `Mask` uses the sibling [`MaterialAttr::alpha_cutoff`] field for its
/// threshold.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlphaMode {
    Add,
    Blend,
    Mask,
    Multiply,
    #[default]
    Opaque,
    Premultiplied,
}

/// Texture slots are the relationship fields beside this one.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MaterialAttr {
    pub alpha_cutoff: Option<f64>,
    pub alpha_mode:   Option<AlphaMode>,
    pub base_color:   Option<ColorVec>,
    pub double_sided: Option<bool>,
    pub emissive:     Option<ColorVec>,
    pub metallic:     Option<f64>,
    pub roughness:    Option<f64>,
}

impl Property for MaterialAttr {
    const NAME: PropName = prop_name!(concatcp!(GROUP, "/value"));
}

relationship!(Binding => BINDING, concatcp!(GROUP, "/binding"));
relationship!(BaseColorTexture => BASE_COLOR_TEXTURE, concatcp!(GROUP, "/base_color_texture"));
relationship!(EmissiveTexture => EMISSIVE_TEXTURE, concatcp!(GROUP, "/emissive_texture"));
relationship!(
    MetallicRoughnessTexture => METALLIC_ROUGHNESS_TEXTURE,
    concatcp!(GROUP, "/metallic_roughness_texture")
);
relationship!(NormalTexture => NORMAL_TEXTURE, concatcp!(GROUP, "/normal_texture"));
relationship!(OcclusionTexture => OCCLUSION_TEXTURE, concatcp!(GROUP, "/occlusion_texture"));
