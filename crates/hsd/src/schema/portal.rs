use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    id::{
        DocId,
        PrimId,
    },
    prop_name,
    property::{
        Property,
        name::PropName,
    },
};

pub const GROUP: &str = "portal";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortalReceptor {
    pub document: DocId,
    pub prim:     PrimId,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortalDestination {
    pub receptor: Option<PortalReceptor>,
    pub space:    [u8; 32],
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct PortalAttr {
    pub destination: Option<PortalDestination>,
    pub size_x:      f64,
    pub size_y:      f64,
}

impl Property for PortalAttr {
    const NAME: PropName = prop_name!(GROUP);
}
