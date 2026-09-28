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

pub const GROUP: &str = "spawn";

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct SpawnAttr {
    pub radius: f64,
}

impl Property for SpawnAttr {
    const NAME: PropName = prop_name!(GROUP);
}
