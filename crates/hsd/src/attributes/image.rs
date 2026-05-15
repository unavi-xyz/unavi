use lorosurgeon::{ByteArray, Hydrate, Reconcile};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile, Debug)]
pub struct ImageAttr {
    pub address_mode_u: Option<i64>,
    pub address_mode_v: Option<i64>,
    pub address_mode_w: Option<i64>,
    pub data: ByteArray<32>,
    pub mag_filter: Option<i64>,
    pub min_filter: Option<i64>,
    pub mipmap_filter: Option<i64>,
    pub name: Option<String>,
    pub srgb: Option<bool>,
}

impl Attribute for ImageAttr {
    const KEY: &str = "image";
}
