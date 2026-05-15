use lorosurgeon::{Hydrate, Reconcile};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile, Debug)]
pub struct XformAttr {
    #[loro(default = "default_rotation")]
    pub rotation: Vec<f32>,
    #[loro(default = "default_scale")]
    pub scale: Vec<f32>,
    #[loro(default)]
    pub translation: Vec<f32>,
}

fn default_rotation() -> Vec<f32> {
    vec![0.0, 0.0, 0.0, 1.0]
}

fn default_scale() -> Vec<f32> {
    vec![1.0; 3]
}

impl Attribute for XformAttr {
    const KEY: &str = "xform";
}
