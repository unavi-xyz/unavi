use const_format::concatcp;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    prop_name,
    property::{
        Payload,
        Property,
        name::PropName,
        render_bytes,
    },
};

pub const GROUP: &str = "image";

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageSampler {
    pub address_mode_u: Option<AddressMode>,
    pub address_mode_v: Option<AddressMode>,
    pub address_mode_w: Option<AddressMode>,
    pub mag_filter:     Option<FilterMode>,
    pub min_filter:     Option<FilterMode>,
    pub mipmap_filter:  Option<FilterMode>,
    pub srgb:           Option<bool>,
}

impl Property for ImageSampler {
    const NAME: PropName = prop_name!(concatcp!(GROUP, "/sampler"));
}

/// Encoded image bytes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ImageData(pub Vec<u8>);

impl Property for ImageData {
    const NAME: PropName = prop_name!(concatcp!(GROUP, "/data"));

    fn render(payload: &[u8]) -> String {
        match Self::decode(payload) {
            Ok(value) => render_bytes(value.0.len()),
            Err(err) => format!("<undecodable: {err}>"),
        }
    }
}
