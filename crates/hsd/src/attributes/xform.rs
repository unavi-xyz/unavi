use lorosurgeon::{Hydrate, Reconcile};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile, Debug)]
#[loro(default)]
pub struct XformAttr {
    pub rotation: Vec<f32>,
    pub scale: Vec<f32>,
    pub translation: Vec<f32>,
}

impl Attribute for XformAttr {
    const KEY: &str = "xform";
}
