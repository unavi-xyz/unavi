use loro_surgeon::{Hydrate, Reconcile, bytes::ByteArray};
use serde::{Deserialize, Serialize};

use crate::attributes::Attribute;

#[serde_with::skip_serializing_none]
#[derive(Hydrate, Reconcile, Debug, Clone, Default, Serialize, Deserialize)]
#[loro(default)]
#[serde(default)]
pub struct ImageAttr {
    pub address_mode_u: Option<i64>,
    pub address_mode_v: Option<i64>,
    pub address_mode_w: Option<i64>,
    pub data: ByteArray<32>,
    pub mag_filter: Option<i64>,
    pub min_filter: Option<i64>,
    pub mipmap_filter: Option<i64>,
    pub srgb: Option<bool>,
}

impl Attribute for ImageAttr {
    const KEY: &str = "image";
}
