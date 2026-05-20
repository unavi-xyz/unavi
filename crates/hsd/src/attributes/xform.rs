use lorosurgeon::{Hydrate, Reconcile};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile, Debug, Clone)]
pub struct XformAttr {
    #[loro(with = "crate::attributes::value_array::rotation")]
    pub rotation: Vec<f32>,
    #[loro(with = "crate::attributes::value_array::scale")]
    pub scale: Vec<f32>,
    #[loro(with = "crate::attributes::value_array::translation")]
    pub translation: Vec<f32>,
}

impl Attribute for XformAttr {
    const KEY: &str = "xform";
}
