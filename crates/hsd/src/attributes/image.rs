use serde::{
    Deserialize,
    Serialize,
};

use crate::attributes::Attribute;

/// How a sampler treats coordinates outside `[0, 1]`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressMode {
    #[default]
    Repeat,
    MirrorRepeat,
    ClampToEdge,
}

/// How a sampler picks between texels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterMode {
    #[default]
    Linear,
    Nearest,
}

/// Sampler settings only; the encoded image is the `image:data` slot.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageAttr {
    pub address_mode_u: Option<AddressMode>,
    pub address_mode_v: Option<AddressMode>,
    pub address_mode_w: Option<AddressMode>,
    pub mag_filter:     Option<FilterMode>,
    pub min_filter:     Option<FilterMode>,
    pub mipmap_filter:  Option<FilterMode>,
    pub srgb:           Option<bool>,
}

impl Attribute for ImageAttr {
    const KEY: &'static str = "image";
}
