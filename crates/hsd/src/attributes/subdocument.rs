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

#[derive(Hydrate, Reconcile, Debug, Clone, Serialize, Deserialize)]
pub enum SubdocumentAttr {
    Template(ByteArray<32>),
    /// A nested subdocument instanced as its own iroh-docs namespace; the bytes
    /// are its [`NamespaceId`](iroh_docs::NamespaceId).
    Doc(ByteArray<32>),
}

impl Attribute for SubdocumentAttr {
    const KEY: &str = "subdocument";
}
