use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    attributes::Attribute,
    id::{
        DocId,
        PrimId,
    },
};

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

impl Attribute for PortalAttr {
    const KEY: &'static str = "portal";
}
