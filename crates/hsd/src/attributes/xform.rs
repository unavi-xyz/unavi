use loro_surgeon::{Hydrate, Reconcile};
use serde::{Deserialize, Serialize};

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

#[derive(Hydrate, Reconcile, Debug, Clone, Serialize, Deserialize)]
pub struct XformAttr {
    #[loro(default = "default_rotation")]
    #[serde(default = "default_rotation")]
    pub rotation: [f32; 4],
    #[loro(default = "default_scale")]
    #[serde(default = "default_scale")]
    pub scale: [f32; 3],
    #[loro(default = "default_translation")]
    #[serde(default = "default_translation")]
    pub translation: [f32; 3],
}

impl Default for XformAttr {
    fn default() -> Self {
        Self {
            rotation: default_rotation(),
            scale: default_scale(),
            translation: default_translation(),
        }
    }
}

impl Attribute for XformAttr {
    const KEY: &str = "xform";
}
