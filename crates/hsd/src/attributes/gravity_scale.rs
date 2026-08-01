use loro_surgeon::{
    Hydrate,
    Reconcile,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile, Debug, Clone, Copy, Serialize, Deserialize)]
#[loro(default)]
#[serde(default)]
pub struct GravityScaleAttr {
    pub scale: f64,
}

impl Default for GravityScaleAttr {
    fn default() -> Self {
        Self { scale: 1.0 }
    }
}

impl Attribute for GravityScaleAttr {
    const KEY: &str = "gravity_scale";
}
