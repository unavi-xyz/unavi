//! HSS (Hyper-Space Shader), HSD's shader graph format: a closed node graph,
//! compiled host-side into WGSL. Never accepts shader text as data.
//!
//! Nodes are a flat, ordered list where a node's inputs may reference only
//! nodes at a strictly lower index, so a cycle cannot be constructed and no
//! graph-traversal cycle check is needed. Fixed arity per node kind bounds
//! total shader cost to a static function of node count.
//!
//! A graph has two independent networks, mirroring USD's `Material` prim
//! terminals (`surface`, `displacement`): [`graph::SurfaceGraph`] computes the
//! fragment-stage look, in either a lit (PBR) or unlit shape, and the optional
//! [`graph::DisplacementGraph`] computes a vertex-stage position/normal
//! offset.
//!
//! The compiled graph is slot content (`material:graph_data`), never an
//! attribute payload; [`overrides::GraphOverridesAttr`] is the small
//! attribute that names the same prim's per-instance tint of the graph's
//! public inputs.

use serde::{
    Deserialize,
    Serialize,
};

pub mod graph;
pub mod node;
pub mod overrides;
pub mod parse;
pub mod validate;
pub mod value;

pub const EXTENSION: &str = "hss";
/// Per-network node cap; surface and displacement run as different stages
/// with independent budgets.
pub const MAX_NODES: usize = 128;
/// Texture-sample node cap. Surface only — see
/// [`validate::error::GraphError::TextureSampleInDisplacement`].
pub const MAX_TEXTURE_SAMPLES: usize = 4;
/// Public-input cap: matches the fixed uniform budget of the generated
/// `AsBindGroup` (one `vec4` slot per input).
pub const MAX_PUBLIC_INPUTS: usize = 16;

/// A compiled, closed shader graph. This is slot content
/// (`material:graph_data`), never an attribute payload — see the module docs.
///
/// No field here may become a `HashMap`: dedup across prims depends on
/// [`Self::encode`] producing byte-identical output for structurally
/// identical graphs, which a `Vec`/`BTreeMap`-only shape guarantees and a
/// `HashMap` would not.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ShaderGraph {
    /// Default values for the graph's public inputs; overridden per-instance
    /// by [`overrides::GraphOverridesAttr`]. Index is the [`node::Port::Input`]
    /// either network's nodes reference — the one thing surface and
    /// displacement share.
    pub public_inputs: Vec<value::GraphValue>,
    pub surface:       graph::SurfaceGraph,
    pub displacement:  Option<graph::DisplacementGraph>,
}

impl ShaderGraph {
    pub fn encode(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_stdvec(self)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(bytes)
    }
}
