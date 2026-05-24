use lorosurgeon::{Hydrate, Reconcile};
use serde::{Deserialize, Serialize};

use crate::attributes::Attribute;

fn default_rotation() -> Vec<f32> {
    vec![0.0, 0.0, 0.0, 1.0]
}
fn default_scale() -> Vec<f32> {
    vec![1.0, 1.0, 1.0]
}
fn default_translation() -> Vec<f32> {
    vec![0.0, 0.0, 0.0]
}

#[derive(Hydrate, Reconcile, Debug, Clone, Serialize, Deserialize)]
pub struct XformAttr {
    #[loro(with = "crate::attributes::value_array::rotation")]
    #[serde(default = "default_rotation")]
    pub rotation: Vec<f32>,
    #[loro(with = "crate::attributes::value_array::scale")]
    #[serde(default = "default_scale")]
    pub scale: Vec<f32>,
    #[loro(with = "crate::attributes::value_array::translation")]
    #[serde(default = "default_translation")]
    pub translation: Vec<f32>,
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
