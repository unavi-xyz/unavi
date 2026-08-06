//! HSS (Hyper-Space Shader), HSD's shader graph format: a closed-by-
//! construction node graph, compiled host-side into WGSL.
//!
//! Never accepts shader text as data: nodes are a flat, ordered list where a
//! node's inputs may reference only nodes at a strictly lower index, so a
//! cycle cannot be constructed and no graph-traversal cycle check is needed.
//! Fixed arity per node kind bounds total shader cost to a static function of
//! node count.
//!
//! A graph has two independent networks, mirroring USD's `Material` prim
//! terminals (`surface`, `displacement`): [`SurfaceGraph`] computes the
//! fragment-stage look, in either a lit (PBR) or unlit shape, and the
//! optional [`DisplacementGraph`] computes a vertex-stage position/normal
//! offset — both unreachable through a single fixed terminal set.
//!
//! The compiled graph is slot content (`material:graph_data`), never a hash
//! inside an attribute payload; [`overrides::GraphOverridesAttr`] is the
//! small attribute that names the same prim's per-instance tint of the
//! graph's public inputs.

use serde::{
    Deserialize,
    Serialize,
};

pub mod overrides;
#[cfg(test)] mod tests;
pub mod validate;

/// File extension for a hand-written shader graph source: Hyper-Space
/// Shader, HSD's shader format for The Wired.
pub const EXTENSION: &str = "hss";
/// Per-network node cap: shader cost is a static function of node count
/// alone, and surface/displacement run as different shader stages with
/// independent budgets.
pub const MAX_NODES: usize = 128;
/// Texture-sample node cap. Surface only — see
/// [`validate::GraphError::TextureSampleInDisplacement`].
pub const MAX_TEXTURE_SAMPLES: usize = 4;
/// Public-input cap: matches the fixed uniform budget of the generated
/// `AsBindGroup` (one `vec4` slot per input).
pub const MAX_PUBLIC_INPUTS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueKind {
    Float,
    Vec2,
    Vec3,
    Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GraphValue {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Color([f32; 4]),
}

impl GraphValue {
    #[must_use]
    pub const fn kind(&self) -> ValueKind {
        match self {
            Self::Float(_) => ValueKind::Float,
            Self::Vec2(_) => ValueKind::Vec2,
            Self::Vec3(_) => ValueKind::Vec3,
            Self::Color(_) => ValueKind::Color,
        }
    }
}

/// A node input.
///
/// Either a constant baked into the compiled graph, a public input
/// overridable per-instance without recompiling, or another node's output.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Port {
    Const(GraphValue),
    /// Index into [`ShaderGraph::public_inputs`].
    Input(u16),
    /// Index into the enclosing network's node list. Must be strictly less
    /// than the index of the node this port belongs to, and never reaches
    /// across networks — surface and displacement run as different shader
    /// stages and share no per-invocation values.
    Node(u16),
}

/// A graph node.
///
/// The zero-arity leaves are the only way a graph reaches shader-stage
/// context; each is legal in exactly one network (enforced by
/// [`validate::validate`], not by the type — see
/// [`validate::GraphError::WrongNetwork`]): `Uv`/`WorldNormal`/
/// `WorldPosition`/`VertexColor` are surface-only (fragment-stage
/// varyings), `LocalPosition`/`LocalNormal` are displacement-only
/// (vertex-stage attributes), `Time` is legal in both.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Node {
    Uv,
    WorldNormal,
    WorldPosition,
    VertexColor,
    LocalPosition,
    LocalNormal,
    Time,
    Add {
        a: Port,
        b: Port,
    },
    Mul {
        a: Port,
        b: Port,
    },
    Lerp {
        a: Port,
        b: Port,
        t: Port,
    },
    Dot {
        a: Port,
        b: Port,
    },
    /// Legal in both networks — the basic oscillator a `Time`-driven pulse,
    /// wave or sway effect needs, in either the fragment or vertex stage.
    Sin {
        x: Port,
    },
    Cos {
        x: Port,
    },
    /// Power exponent on `1 - dot(N, V)`. `N`/`V` are the fragment's own
    /// normal/view vectors, not graph inputs — a graph cannot construct an
    /// arbitrary Fresnel term, only parameterize the host-provided one.
    /// Surface-only: `N`/`V` are not defined in the vertex stage.
    Fresnel {
        power: Port,
    },
    Noise {
        uv: Port,
    },
    /// `slot` selects one of a fixed `MAX_TEXTURE_SAMPLES` texture bindings,
    /// never an open-ended name — consistent with every other fixed-arity
    /// choice in this format. Surface-only for v1; see
    /// [`validate::GraphError::TextureSampleInDisplacement`].
    TextureSample {
        uv:   Port,
        slot: u8,
    },
    /// The only branching this format allows: a hard switch on `cond`, no
    /// general control flow. Matches how GPUs actually execute (SIMT).
    Select {
        cond: Port,
        a:    Port,
        b:    Port,
    },
}

/// Which shader stage a network compiles to. Carried in errors so a
/// validation failure names the network it came from; also what
/// [`validate::validate`] checks each leaf [`Node`] against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Network {
    Surface,
    Displacement,
}

/// The fragment-stage network.
///
/// Two closed shapes, chosen at authoring time, not a fully generic
/// multi-output graph — mirrors Unity Shader Graph's Lit/Unlit Master Stack
/// targets and Unreal's Shading Model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SurfaceGraph {
    pub nodes:  Vec<Node>,
    pub output: SurfaceOutput,
}

impl Default for SurfaceGraph {
    fn default() -> Self {
        Self {
            nodes:  Vec::new(),
            output: SurfaceOutput::Unlit(UnlitOutput::default()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SurfaceOutput {
    Lit(LitOutput),
    Unlit(UnlitOutput),
}

/// Fed into a `PbrInput` before `apply_pbr_lighting`, mirroring glTF/PBR's
/// small standard surface-output vocabulary — codegen always targets this
/// one known shape when `Lit` is selected.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LitOutput {
    pub base_color:           Option<Port>,
    pub emissive:             Option<Port>,
    pub metallic:             Option<Port>,
    pub roughness:            Option<Port>,
    pub normal:               Option<Port>,
    pub alpha:                Option<Port>,
    /// Unity's Alpha Clip Threshold / Unreal's Opacity Mask: when set,
    /// codegen emits a `discard` below this threshold — still a single
    /// bounded statement, not a loop or general branch.
    pub alpha_clip_threshold: Option<Port>,
}

/// Written straight to the fragment output; no `PbrInput`, no lighting pass.
/// What `SkyMaterial`, a beam, a hologram, or any additive/emissive VFX
/// needs, and what a `Lit`-only terminal set cannot express.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UnlitOutput {
    pub color:                Port,
    pub alpha_clip_threshold: Option<Port>,
}

impl Default for UnlitOutput {
    fn default() -> Self {
        Self {
            color:                Port::Const(GraphValue::Color([1.0, 1.0, 1.0, 1.0])),
            alpha_clip_threshold: None,
        }
    }
}

/// The vertex-stage network.
///
/// `position_offset` is added to the mesh's local-space vertex position
/// before the standard local→world→clip transform runs; `normal_override`,
/// if set, replaces the local-space normal before it is transformed to
/// world space. Covers displacement, billboarding, and simple vertex
/// animation; the Bevy-side vertex shader splices this in right after
/// fetching the mesh's local-space position/normal, before the standard
/// mesh transform runs (see `bevy_pbr`'s own vertex shader for that part).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DisplacementGraph {
    pub nodes:           Vec<Node>,
    pub position_offset: Option<Port>,
    pub normal_override: Option<Port>,
}

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
    /// by [`overrides::GraphOverridesAttr`]. Index is the [`Port::Input`]
    /// either network's nodes reference — the one thing surface and
    /// displacement share.
    pub public_inputs: Vec<GraphValue>,
    pub surface:       SurfaceGraph,
    pub displacement:  Option<DisplacementGraph>,
}

impl ShaderGraph {
    pub fn encode(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_stdvec(self)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(bytes)
    }
}
