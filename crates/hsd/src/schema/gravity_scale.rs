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
};

pub const GROUP: &str = "gravity_scale";

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GravityScaleAttr {
    pub scale: f64,
}

impl Default for GravityScaleAttr {
    fn default() -> Self {
        Self { scale: 1.0 }
    }
}

impl Property for GravityScaleAttr {
    const NAME: PropName = prop_name!(GROUP);
}
