use serde::{
    Deserialize,
    Serialize,
};

use crate::attributes::Attribute;

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct SpawnAttr {
    pub radius: f64,
}

impl Attribute for SpawnAttr {
    const KEY: &'static str = "spawn";
}
