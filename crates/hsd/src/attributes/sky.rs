use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    attributes::material::ColorVec,
    prop_name,
    property::{
        Property,
        name::PropName,
    },
};

pub const GROUP: &str = "sky";

/// The colour a space's sky takes, in the encoding of a material colour.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkyAttr {
    pub color: ColorVec,
}

impl Property for SkyAttr {
    const NAME: PropName = prop_name!(GROUP);
}
