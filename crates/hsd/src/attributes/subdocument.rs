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
    Record(ByteArray<32>),
}

impl Attribute for SubdocumentAttr {
    const KEY: &str = "subdocument";
}
