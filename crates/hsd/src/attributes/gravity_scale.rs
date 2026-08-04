use serde::{
    Deserialize,
    Serialize,
};

use crate::attributes::Attribute;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GravityScaleAttr {
    pub scale: f64,
}

impl Default for GravityScaleAttr {
    fn default() -> Self {
        Self { scale: 1.0 }
    }
}

impl Attribute for GravityScaleAttr {
    const KEY: &'static str = "gravity_scale";
}
