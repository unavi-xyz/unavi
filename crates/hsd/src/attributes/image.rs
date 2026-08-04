use serde::{
    Deserialize,
    Serialize,
};

use crate::attributes::Attribute;

/// Sampler settings only; the encoded image is the `image:data` bulk entry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageAttr {
    pub address_mode_u: Option<i64>,
    pub address_mode_v: Option<i64>,
    pub address_mode_w: Option<i64>,
    pub mag_filter:     Option<i64>,
    pub min_filter:     Option<i64>,
    pub mipmap_filter:  Option<i64>,
    pub srgb:           Option<bool>,
}

impl Attribute for ImageAttr {
    const KEY: &'static str = "image";
}
