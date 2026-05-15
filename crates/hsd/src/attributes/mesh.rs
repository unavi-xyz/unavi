use std::collections::BTreeMap;

use lorosurgeon::{ByteArray, Hydrate, MaybeMissing, Reconcile};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile, Debug, Clone)]
#[loro(default)]
pub struct MeshAttr {
    pub attributes: BTreeMap<String, ByteArray<32>>,
    pub indices: MaybeMissing<ByteArray<32>>,
    pub topology: i64,
}

impl Attribute for MeshAttr {
    const KEY: &str = "mesh";
}
