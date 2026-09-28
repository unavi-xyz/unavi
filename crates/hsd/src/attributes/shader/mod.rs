//! HSS (Hyper-Space Shader), HSD's shader graph format. Never accepts shader
//! text as data.
//!
//! A node's inputs may reference only nodes at a strictly lower index, which
//! rules out cycles without a traversal check.

use const_format::concatcp;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    prop_name,
    property::{
        Property,
        name::PropName,
    },
    relationship,
};

pub mod graph;
pub mod node;
pub mod overrides;
pub mod parse;
pub mod validate;
pub mod value;

pub const GROUP: &str = "shader";

pub const EXTENSION: &str = "hss";
/// Per-network node cap.
pub const MAX_NODES: usize = 128;
/// Texture-sample node cap. Surface only.
pub const MAX_TEXTURE_SAMPLES: usize = 4;
/// Public-input cap. Matches the generated `AsBindGroup`'s fixed uniform
/// budget of one `vec4` slot per input.
pub const MAX_PUBLIC_INPUTS: usize = 16;

relationship!(Texture0 => TEXTURE_0, concatcp!(GROUP, "/texture:0"));
relationship!(Texture1 => TEXTURE_1, concatcp!(GROUP, "/texture:1"));
relationship!(Texture2 => TEXTURE_2, concatcp!(GROUP, "/texture:2"));
relationship!(Texture3 => TEXTURE_3, concatcp!(GROUP, "/texture:3"));

const TEXTURES: [PropName; MAX_TEXTURE_SAMPLES] = [TEXTURE_0, TEXTURE_1, TEXTURE_2, TEXTURE_3];

/// The relationship naming the texture bound to sample slot `slot`, `None`
/// past [`MAX_TEXTURE_SAMPLES`].
#[must_use]
pub fn texture(slot: u8) -> Option<PropName> {
    TEXTURES.get(usize::from(slot)).cloned()
}

/// A compiled, closed shader graph.
///
/// No field here may be a `HashMap`. Encoding must be byte-identical for
/// structurally identical graphs to dedup correctly.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ShaderGraph {
    /// Default values for the graph's public inputs. The index is shared
    /// between the surface and displacement networks.
    pub public_inputs: Vec<value::GraphValue>,
    pub surface:       graph::SurfaceGraph,
    pub displacement:  Option<graph::DisplacementGraph>,
}

impl Property for ShaderGraph {
    const NAME: PropName = prop_name!(concatcp!(GROUP, "/graph"));
}
