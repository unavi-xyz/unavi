use loro_surgeon::{
    Hydrate,
    Reconcile,
    bytes::ByteArray,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile, Debug, Clone, Default, Serialize, Deserialize)]
#[loro(default)]
#[serde(default)]
pub struct PortalReceptor {
    pub document: ByteArray<32>,
    pub prim:     String,
}

#[serde_with::skip_serializing_none]
#[derive(Hydrate, Reconcile, Debug, Clone, Default, Serialize, Deserialize)]
#[loro(default)]
#[serde(default)]
pub struct PortalDestination {
    pub receptor: Option<PortalReceptor>,
    pub space:    ByteArray<32>,
}

#[serde_with::skip_serializing_none]
#[derive(Hydrate, Reconcile, Debug, Clone, Default, Serialize, Deserialize)]
#[loro(default)]
#[serde(default)]
pub struct PortalAttr {
    pub allow_incoming: bool,
    pub destination:    Option<PortalDestination>,
    pub size_x:         f64,
    pub size_y:         f64,
}

impl Attribute for PortalAttr {
    const KEY: &str = "portal";
}
