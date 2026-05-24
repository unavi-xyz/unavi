use std::collections::BTreeMap;

use lorosurgeon::{ByteArray, Hydrate, MaybeMissing, Reconcile};
use serde::{Deserialize, Serialize};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile, Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum Topology {
    PointList,
    LineList,
    LineStrip,
    #[default]
    TriangleList,
    TriangleStrip,
}

#[derive(Hydrate, Reconcile, Debug, Clone, Default, Serialize, Deserialize)]
#[loro(default)]
#[serde(default)]
pub struct MeshAttr {
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, ByteArray<32>>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub indices: MaybeMissing<ByteArray<32>>,
    pub topology: Topology,
}

impl Attribute for MeshAttr {
    const KEY: &str = "mesh";
}
