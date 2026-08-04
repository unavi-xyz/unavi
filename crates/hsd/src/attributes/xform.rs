use serde::{
    Deserialize,
    Serialize,
};

use crate::attributes::Attribute;

const fn default_rotation() -> [f32; 4] {
    [0.0, 0.0, 0.0, 1.0]
}
const fn default_scale() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}
const fn default_translation() -> [f32; 3] {
    [0.0, 0.0, 0.0]
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct XformAttr {
    #[serde(default = "default_rotation")]
    pub rotation:    [f32; 4],
    #[serde(default = "default_scale")]
    pub scale:       [f32; 3],
    #[serde(default = "default_translation")]
    pub translation: [f32; 3],
}

impl Default for XformAttr {
    fn default() -> Self {
        Self {
            rotation:    default_rotation(),
            scale:       default_scale(),
            translation: default_translation(),
        }
    }
}

impl Attribute for XformAttr {
    const KEY: &'static str = "xform";
}
