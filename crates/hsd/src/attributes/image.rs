use lorosurgeon::{ByteArray, Hydrate, MaybeMissing, Reconcile};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile, Debug, Clone)]
#[loro(default)]
pub struct ImageAttr {
    pub address_mode_u: MaybeMissing<i64>,
    pub address_mode_v: MaybeMissing<i64>,
    pub address_mode_w: MaybeMissing<i64>,
    pub data: ByteArray<32>,
    pub mag_filter: MaybeMissing<i64>,
    pub min_filter: MaybeMissing<i64>,
    pub mipmap_filter: MaybeMissing<i64>,
    pub srgb: MaybeMissing<bool>,
}

impl Attribute for ImageAttr {
    const KEY: &str = "image";
}
