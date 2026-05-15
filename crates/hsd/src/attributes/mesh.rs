use std::collections::BTreeMap;

use lorosurgeon::{ByteArray, Hydrate, Reconcile};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile, Debug)]
pub struct MeshAttr {
    #[loro(default)]
    pub attributes: BTreeMap<String, ByteArray<32>>,
    pub indices: Option<ByteArray<32>>,
    #[loro(default)]
    pub topology: i64,
}

impl Attribute for MeshAttr {
    const KEY: &str = "mesh";
}
