use lorosurgeon::{Hydrate, Reconcile};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile)]
#[loro(default)]
pub struct Xform {
    pub rotation: Vec<f32>,
    pub scale: Vec<f32>,
    pub translation: Vec<f32>,
}

impl Attribute for Xform {
    const KEY: &str = "xform";
}
