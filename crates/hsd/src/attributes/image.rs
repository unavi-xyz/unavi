use lorosurgeon::{ByteArray, Hydrate, MaybeMissing, Reconcile};
use serde::{Deserialize, Serialize};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile, Debug, Clone, Default, Serialize, Deserialize)]
#[loro(default)]
#[serde(default)]
pub struct ImageAttr {
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub address_mode_u: MaybeMissing<i64>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub address_mode_v: MaybeMissing<i64>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub address_mode_w: MaybeMissing<i64>,
    pub data: ByteArray<32>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub mag_filter: MaybeMissing<i64>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub min_filter: MaybeMissing<i64>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub mipmap_filter: MaybeMissing<i64>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub srgb: MaybeMissing<bool>,
}

impl Attribute for ImageAttr {
    const KEY: &str = "image";
}
