use loro_surgeon::{
    Hydrate,
    Reconcile,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile, Debug, Clone, Default, Serialize, Deserialize)]
#[loro(default)]
#[serde(default)]
pub struct SpawnAttr {
    pub radius: f64,
}

impl Attribute for SpawnAttr {
    const KEY: &str = "spawn";
}
