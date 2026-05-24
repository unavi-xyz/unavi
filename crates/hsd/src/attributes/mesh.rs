use std::collections::BTreeMap;

use loro_surgeon::{
    bytes::ByteArray,
    {Hydrate, Reconcile},
};
use serde::{Deserialize, Serialize};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile, Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Topology {
    PointList,
    LineList,
    LineStrip,
    #[default]
    TriangleList,
    TriangleStrip,
}

#[serde_with::skip_serializing_none]
#[derive(Hydrate, Reconcile, Debug, Clone, Default, Serialize, Deserialize)]
#[loro(default)]
#[serde(default)]
pub struct MeshAttr {
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, ByteArray<32>>,
    pub indices: Option<ByteArray<32>>,
    pub topology: Topology,
}

impl Attribute for MeshAttr {
    const KEY: &str = "mesh";
}
