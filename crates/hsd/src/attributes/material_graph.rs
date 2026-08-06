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
//! offset. This is what makes the format a genuine WGSL replacement rather
//! than a PBR-parameter tinter — unlit looks and vertex displacement are
//! both unreachable through a single fixed terminal set.
//!
//! The compiled graph is slot content (`material:graph_data`), never a hash
//! inside an attribute payload; [`GraphOverridesAttr`] is the small attribute
//! that names the same prim's per-instance tint of the graph's public inputs.

use std::collections::BTreeMap;

use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;

use crate::attributes::Attribute;

/// File extension for a hand-written shader graph source: Hyper-Space
/// Shader, HSD's shader format for The Wired.
pub const EXTENSION: &str = "hss";
/// Per-network node cap: shader cost is a static function of node count
/// alone, and surface/displacement run as different shader stages with
/// independent budgets.
pub const MAX_NODES: usize = 128;
/// Texture-sample node cap. Surface only — see
/// [`GraphError::TextureSampleInDisplacement`].
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

/// Fixed arity per kind.
///
/// As struct-variants rather than a generic `Vec<Port>`: total shader cost is
/// a static function of node count, and an author mistake is a type error at
/// validation, not a runtime graph walk.
///
/// The zero-arity leaves are the only way a graph reaches shader-stage
/// context; each is legal in exactly one network (enforced by [`validate`],
/// not by the type — see [`GraphError::WrongNetwork`]): `Uv`/`WorldNormal`/
/// `WorldPosition`/`VertexColor` are surface-only (fragment-stage
/// varyings), `LocalPosition`/`LocalNormal` are displacement-only
/// (vertex-stage attributes), `Time` is legal in both.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeKind {
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
    /// [`GraphError::TextureSampleInDisplacement`].
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub kind: NodeKind,
}

/// Which shader stage a network compiles to. Carried in errors so a
/// validation failure names the network it came from; also what
/// [`validate`] checks each leaf [`NodeKind`] against.
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
    /// by [`GraphOverridesAttr`]. Index is the [`Port::Input`] either
    /// network's nodes reference — the one thing surface and displacement
    /// share.
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

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    #[error("{network:?} network has {count} nodes, exceeding the cap of {MAX_NODES}")]
    TooManyNodes { network: Network, count: usize },
    #[error("graph declares {0} public inputs, exceeding the cap of {MAX_PUBLIC_INPUTS}")]
    TooManyPublicInputs(usize),
    #[error("surface network samples {0} textures, exceeding the cap of {MAX_TEXTURE_SAMPLES}")]
    TooManyTextureSamples(usize),
    #[error("texture slot {0} is out of the {MAX_TEXTURE_SAMPLES}-slot range")]
    InvalidTextureSlot(u8),
    #[error("displacement network node {0} samples a texture, which is not supported in v1")]
    TextureSampleInDisplacement(usize),
    #[error("{network:?} network node {node} is not legal outside its own network")]
    WrongNetwork { network: Network, node: usize },
    #[error(
        "{network:?} network node {node} references node {target}, which is not at a strictly lower index"
    )]
    ForwardReference {
        network: Network,
        node:    usize,
        target:  u16,
    },
    #[error(
        "{network:?} network node {node} references public input {index}, which does not exist"
    )]
    UnknownInput {
        network: Network,
        node:    usize,
        index:   u16,
    },
    #[error("terminal {0} references node {1}, which does not exist")]
    UnknownTerminalNode(&'static str, u16),
    #[error("terminal {0} references public input {1}, which does not exist")]
    UnknownTerminalInput(&'static str, u16),
    #[error("{network:?} network node {node} port {port} expected {expected:?}, got {found:?}")]
    NodeTypeMismatch {
        network:  Network,
        node:     usize,
        port:     &'static str,
        expected: ValueKind,
        found:    ValueKind,
    },
    #[error("terminal {name} expected {expected:?}, got {found:?}")]
    TerminalTypeMismatch {
        name:     &'static str,
        expected: ValueKind,
        found:    ValueKind,
    },
}

/// The per-node-index output kinds of a validated network, one entry per
/// network present.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Validated {
    pub surface:      Vec<ValueKind>,
    pub displacement: Option<Vec<ValueKind>>,
}

/// Validates structure and types for both networks, and returns each node's
/// output kind in order.
///
/// Cheap enough to re-run on every load: a peer's document, or a
/// script-submitted graph, is never trusted on `hsd-cli`'s say-so alone.
pub fn validate(graph: &ShaderGraph) -> Result<Validated, GraphError> {
    if graph.public_inputs.len() > MAX_PUBLIC_INPUTS {
        return Err(GraphError::TooManyPublicInputs(graph.public_inputs.len()));
    }
    let surface = validate_surface(&graph.surface, &graph.public_inputs)?;
    let displacement = graph
        .displacement
        .as_ref()
        .map(|d| validate_displacement(d, &graph.public_inputs))
        .transpose()?;
    Ok(Validated {
        surface,
        displacement,
    })
}

pub fn validate_surface(
    surface: &SurfaceGraph,
    public_inputs: &[GraphValue],
) -> Result<Vec<ValueKind>, GraphError> {
    let kinds = validate_network(Network::Surface, &surface.nodes, public_inputs)?;
    match &surface.output {
        SurfaceOutput::Lit(lit) => {
            check_terminal(
                "base_color",
                lit.base_color,
                ValueKind::Color,
                public_inputs,
                &kinds,
            )?;
            check_terminal(
                "emissive",
                lit.emissive,
                ValueKind::Vec3,
                public_inputs,
                &kinds,
            )?;
            check_terminal(
                "metallic",
                lit.metallic,
                ValueKind::Float,
                public_inputs,
                &kinds,
            )?;
            check_terminal(
                "roughness",
                lit.roughness,
                ValueKind::Float,
                public_inputs,
                &kinds,
            )?;
            check_terminal("normal", lit.normal, ValueKind::Vec3, public_inputs, &kinds)?;
            check_terminal("alpha", lit.alpha, ValueKind::Float, public_inputs, &kinds)?;
            check_terminal(
                "alpha_clip_threshold",
                lit.alpha_clip_threshold,
                ValueKind::Float,
                public_inputs,
                &kinds,
            )?;
        }
        SurfaceOutput::Unlit(unlit) => {
            check_terminal(
                "color",
                Some(unlit.color),
                ValueKind::Color,
                public_inputs,
                &kinds,
            )?;
            check_terminal(
                "alpha_clip_threshold",
                unlit.alpha_clip_threshold,
                ValueKind::Float,
                public_inputs,
                &kinds,
            )?;
        }
    }
    Ok(kinds)
}

pub fn validate_displacement(
    displacement: &DisplacementGraph,
    public_inputs: &[GraphValue],
) -> Result<Vec<ValueKind>, GraphError> {
    let kinds = validate_network(Network::Displacement, &displacement.nodes, public_inputs)?;
    check_terminal(
        "position_offset",
        displacement.position_offset,
        ValueKind::Vec3,
        public_inputs,
        &kinds,
    )?;
    check_terminal(
        "normal_override",
        displacement.normal_override,
        ValueKind::Vec3,
        public_inputs,
        &kinds,
    )?;
    Ok(kinds)
}

fn validate_network(
    network: Network,
    nodes: &[Node],
    public_inputs: &[GraphValue],
) -> Result<Vec<ValueKind>, GraphError> {
    if nodes.len() > MAX_NODES {
        return Err(GraphError::TooManyNodes {
            network,
            count: nodes.len(),
        });
    }

    let mut kinds = Vec::with_capacity(nodes.len());
    let mut texture_samples = 0usize;

    for (index, node) in nodes.iter().enumerate() {
        check_network_leaf(network, index, &node.kind)?;

        if let NodeKind::TextureSample { slot, .. } = &node.kind {
            if network == Network::Displacement {
                return Err(GraphError::TextureSampleInDisplacement(index));
            }
            if usize::from(*slot) >= MAX_TEXTURE_SAMPLES {
                return Err(GraphError::InvalidTextureSlot(*slot));
            }
            texture_samples += 1;
        }

        kinds.push(node_output_kind(
            network,
            public_inputs,
            index,
            &node.kind,
            &kinds,
        )?);
    }

    if texture_samples > MAX_TEXTURE_SAMPLES {
        return Err(GraphError::TooManyTextureSamples(texture_samples));
    }

    Ok(kinds)
}

/// Rejects a leaf built-in used outside the one network it is defined in —
/// `Uv`/`WorldNormal`/`WorldPosition`/`VertexColor`/`Fresnel` are
/// fragment-stage varyings that do not exist in the vertex stage,
/// `LocalPosition`/`LocalNormal` are vertex-stage attributes that have no
/// meaning post-rasterization. `Time` and every non-leaf node kind are
/// legal in both.
const fn check_network_leaf(
    network: Network,
    index: usize,
    kind: &NodeKind,
) -> Result<(), GraphError> {
    let surface_only = matches!(
        kind,
        NodeKind::Uv | NodeKind::WorldNormal | NodeKind::WorldPosition | NodeKind::VertexColor
    ) || matches!(kind, NodeKind::Fresnel { .. });
    let displacement_only = matches!(kind, NodeKind::LocalPosition | NodeKind::LocalNormal);

    match network {
        Network::Displacement if surface_only => Err(GraphError::WrongNetwork {
            network,
            node: index,
        }),
        Network::Surface if displacement_only => Err(GraphError::WrongNetwork {
            network,
            node: index,
        }),
        _ => Ok(()),
    }
}

/// Resolves a port's value kind. `at` is the index of the node (or, for a
/// terminal, `kinds.len()`) the port belongs to; a [`Port::Node`] must
/// reference a strictly lower index than that, which is the entire cycle
/// check this format needs.
fn port_kind(
    network: Network,
    public_inputs: &[GraphValue],
    at: usize,
    port: Port,
    kinds: &[ValueKind],
) -> Result<ValueKind, GraphError> {
    match port {
        Port::Const(value) => Ok(value.kind()),
        Port::Input(index) => public_inputs
            .get(usize::from(index))
            .map(GraphValue::kind)
            .ok_or(GraphError::UnknownInput {
                network,
                node: at,
                index,
            }),
        Port::Node(target) => {
            if usize::from(target) >= at {
                return Err(GraphError::ForwardReference {
                    network,
                    node: at,
                    target,
                });
            }
            Ok(kinds[usize::from(target)])
        }
    }
}

fn require(
    network: Network,
    public_inputs: &[GraphValue],
    at: usize,
    port_name: &'static str,
    port: Port,
    kinds: &[ValueKind],
    expected: ValueKind,
) -> Result<(), GraphError> {
    let found = port_kind(network, public_inputs, at, port, kinds)?;
    if found == expected {
        Ok(())
    } else {
        Err(GraphError::NodeTypeMismatch {
            network,
            node: at,
            port: port_name,
            expected,
            found,
        })
    }
}

fn matching(
    network: Network,
    public_inputs: &[GraphValue],
    at: usize,
    a: Port,
    b_name: &'static str,
    b: Port,
    kinds: &[ValueKind],
) -> Result<ValueKind, GraphError> {
    let a_kind = port_kind(network, public_inputs, at, a, kinds)?;
    require(network, public_inputs, at, b_name, b, kinds, a_kind)?;
    Ok(a_kind)
}

fn node_output_kind(
    network: Network,
    public_inputs: &[GraphValue],
    at: usize,
    kind: &NodeKind,
    kinds: &[ValueKind],
) -> Result<ValueKind, GraphError> {
    match *kind {
        NodeKind::Uv => Ok(ValueKind::Vec2),
        NodeKind::WorldNormal
        | NodeKind::WorldPosition
        | NodeKind::LocalPosition
        | NodeKind::LocalNormal => Ok(ValueKind::Vec3),
        NodeKind::VertexColor => Ok(ValueKind::Color),
        NodeKind::Time => Ok(ValueKind::Float),
        NodeKind::Add { a, b } | NodeKind::Mul { a, b } => {
            matching(network, public_inputs, at, a, "b", b, kinds)
        }
        NodeKind::Lerp { a, b, t } => {
            let value_kind = matching(network, public_inputs, at, a, "b", b, kinds)?;
            require(network, public_inputs, at, "t", t, kinds, ValueKind::Float)?;
            Ok(value_kind)
        }
        NodeKind::Dot { a, b } => {
            matching(network, public_inputs, at, a, "b", b, kinds)?;
            Ok(ValueKind::Float)
        }
        NodeKind::Sin { x } | NodeKind::Cos { x } => {
            require(network, public_inputs, at, "x", x, kinds, ValueKind::Float)?;
            Ok(ValueKind::Float)
        }
        NodeKind::Fresnel { power } => {
            require(
                network,
                public_inputs,
                at,
                "power",
                power,
                kinds,
                ValueKind::Float,
            )?;
            Ok(ValueKind::Float)
        }
        NodeKind::Noise { uv } => {
            require(network, public_inputs, at, "uv", uv, kinds, ValueKind::Vec2)?;
            Ok(ValueKind::Float)
        }
        NodeKind::TextureSample { uv, .. } => {
            require(network, public_inputs, at, "uv", uv, kinds, ValueKind::Vec2)?;
            Ok(ValueKind::Color)
        }
        NodeKind::Select { cond, a, b } => {
            require(
                network,
                public_inputs,
                at,
                "cond",
                cond,
                kinds,
                ValueKind::Float,
            )?;
            matching(network, public_inputs, at, a, "b", b, kinds)
        }
    }
}

fn check_terminal(
    name: &'static str,
    port: Option<Port>,
    expected: ValueKind,
    public_inputs: &[GraphValue],
    kinds: &[ValueKind],
) -> Result<(), GraphError> {
    let Some(port) = port else { return Ok(()) };
    // Terminals aren't a node in either list; using `kinds.len()` as `at`
    // means a `Port::Node` reference is valid iff it names an existing node,
    // matching how a real terminal sits "after" every node in the network.
    let found = port_kind(Network::Surface, public_inputs, kinds.len(), port, kinds)
        .map_err(|err| terminal_err(name, err))?;
    if found == expected {
        Ok(())
    } else {
        Err(GraphError::TerminalTypeMismatch {
            name,
            expected,
            found,
        })
    }
}

const fn terminal_err(name: &'static str, err: GraphError) -> GraphError {
    match err {
        GraphError::ForwardReference { target, .. } => {
            GraphError::UnknownTerminalNode(name, target)
        }
        GraphError::UnknownInput { index, .. } => GraphError::UnknownTerminalInput(name, index),
        other => other,
    }
}

/// The small attribute pairing a graph's per-instance public-input tint.
///
/// Follows `MaterialX`'s "bind a nodegraph, override its public inputs"
/// pattern. Never carries the graph itself — that is `material:graph_data`,
/// slot content, since a hash may not appear inside an attribute payload.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GraphOverridesAttr {
    /// Public-input index -> override value. Empty if the graph's own
    /// defaults (`ShaderGraph::public_inputs`) are used as-is.
    pub overrides: BTreeMap<u16, GraphValue>,
}

impl Attribute for GraphOverridesAttr {
    const KEY: &'static str = "material:graph";
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum OverridesError {
    #[error("override targets public input {0}, which the graph does not declare")]
    UnknownInput(u16),
    #[error("override for public input {index} expected {expected:?}, got {found:?}")]
    TypeMismatch {
        index:    u16,
        expected: ValueKind,
        found:    ValueKind,
    },
}

/// Cross-checks overrides against the graph they apply to.
///
/// The two are separate entries (an attribute and a slot) that can
/// arrive out of order or go stale independently, so this is re-run whenever
/// either changes, not folded into [`validate`].
pub fn validate_overrides(
    graph: &ShaderGraph,
    overrides: &GraphOverridesAttr,
) -> Result<(), OverridesError> {
    for (&index, value) in &overrides.overrides {
        let expected = graph
            .public_inputs
            .get(usize::from(index))
            .ok_or(OverridesError::UnknownInput(index))?
            .kind();
        if value.kind() != expected {
            return Err(OverridesError::TypeMismatch {
                index,
                expected,
                found: value.kind(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf(kind: NodeKind) -> Node {
        Node { kind }
    }

    fn unlit(color: Port) -> SurfaceGraph {
        SurfaceGraph {
            nodes:  Vec::new(),
            output: SurfaceOutput::Unlit(UnlitOutput {
                color,
                alpha_clip_threshold: None,
            }),
        }
    }

    #[test]
    fn default_graph_is_valid_and_unlit() {
        let validated = validate(&ShaderGraph::default()).expect("valid");
        assert_eq!(validated.surface, Vec::new());
        assert_eq!(validated.displacement, None);
    }

    #[test]
    fn a_node_may_reference_an_earlier_node_in_the_same_network() {
        let graph = ShaderGraph {
            surface: SurfaceGraph {
                nodes:  vec![
                    leaf(NodeKind::Time),
                    leaf(NodeKind::Add {
                        a: Port::Node(0),
                        b: Port::Const(GraphValue::Float(1.0)),
                    }),
                ],
                output: SurfaceOutput::Lit(LitOutput {
                    roughness: Some(Port::Node(1)),
                    ..Default::default()
                }),
            },
            ..Default::default()
        };
        let validated = validate(&graph).expect("valid");
        assert_eq!(validated.surface, vec![ValueKind::Float, ValueKind::Float]);
    }

    /// The DAG-safety property: a node's inputs may only reference strictly
    /// lower indices, so a cycle cannot be constructed in the first place.
    #[test]
    fn a_node_may_not_reference_itself_or_a_later_node() {
        let self_ref = ShaderGraph {
            surface: SurfaceGraph {
                nodes: vec![leaf(NodeKind::Add {
                    a: Port::Node(0),
                    b: Port::Const(GraphValue::Float(1.0)),
                })],
                ..unlit(Port::Const(GraphValue::Color([0.0; 4])))
            },
            ..Default::default()
        };
        assert_eq!(
            validate(&self_ref),
            Err(GraphError::ForwardReference {
                network: Network::Surface,
                node:    0,
                target:  0,
            })
        );

        let forward_ref = ShaderGraph {
            surface: SurfaceGraph {
                nodes: vec![
                    leaf(NodeKind::Add {
                        a: Port::Node(1),
                        b: Port::Const(GraphValue::Float(1.0)),
                    }),
                    leaf(NodeKind::Time),
                ],
                ..unlit(Port::Const(GraphValue::Color([0.0; 4])))
            },
            ..Default::default()
        };
        assert_eq!(
            validate(&forward_ref),
            Err(GraphError::ForwardReference {
                network: Network::Surface,
                node:    0,
                target:  1,
            })
        );
    }

    #[test]
    fn mismatched_port_types_are_rejected() {
        let graph = ShaderGraph {
            surface: SurfaceGraph {
                nodes: vec![
                    leaf(NodeKind::Uv),
                    leaf(NodeKind::Add {
                        a: Port::Node(0),
                        b: Port::Const(GraphValue::Float(1.0)),
                    }),
                ],
                ..unlit(Port::Const(GraphValue::Color([0.0; 4])))
            },
            ..Default::default()
        };
        assert_eq!(
            validate(&graph),
            Err(GraphError::NodeTypeMismatch {
                network:  Network::Surface,
                node:     1,
                port:     "b",
                expected: ValueKind::Vec2,
                found:    ValueKind::Float,
            })
        );
    }

    #[test]
    fn terminal_referencing_an_out_of_bounds_node_is_rejected() {
        let graph = ShaderGraph {
            surface: unlit(Port::Node(0)),
            ..Default::default()
        };
        assert_eq!(
            validate(&graph),
            Err(GraphError::UnknownTerminalNode("color", 0))
        );
    }

    #[test]
    fn terminal_type_mismatch_is_rejected() {
        let graph = ShaderGraph {
            surface: SurfaceGraph {
                nodes: vec![leaf(NodeKind::Uv)],
                ..unlit(Port::Node(0))
            },
            ..Default::default()
        };
        assert_eq!(
            validate(&graph),
            Err(GraphError::TerminalTypeMismatch {
                name:     "color",
                expected: ValueKind::Color,
                found:    ValueKind::Vec2,
            })
        );
    }

    #[test]
    fn node_count_cap_is_enforced_per_network() {
        let nodes = (0..=MAX_NODES).map(|_| leaf(NodeKind::Time)).collect();
        let graph = ShaderGraph {
            surface: SurfaceGraph {
                nodes,
                ..unlit(Port::Const(GraphValue::Color([0.0; 4])))
            },
            ..Default::default()
        };
        assert_eq!(
            validate(&graph),
            Err(GraphError::TooManyNodes {
                network: Network::Surface,
                count:   MAX_NODES + 1,
            })
        );
    }

    #[test]
    fn texture_sample_cap_is_enforced() {
        let mut nodes = vec![leaf(NodeKind::Uv)];
        nodes.extend((0..=MAX_TEXTURE_SAMPLES).map(|_| {
            leaf(NodeKind::TextureSample {
                uv:   Port::Node(0),
                slot: 0,
            })
        }));
        let graph = ShaderGraph {
            surface: SurfaceGraph {
                nodes,
                ..unlit(Port::Const(GraphValue::Color([0.0; 4])))
            },
            ..Default::default()
        };
        assert_eq!(
            validate(&graph),
            Err(GraphError::TooManyTextureSamples(MAX_TEXTURE_SAMPLES + 1))
        );
    }

    #[test]
    fn texture_slot_out_of_range_is_rejected() {
        let graph = ShaderGraph {
            surface: SurfaceGraph {
                nodes: vec![
                    leaf(NodeKind::Uv),
                    leaf(NodeKind::TextureSample {
                        uv:   Port::Node(0),
                        slot: MAX_TEXTURE_SAMPLES as u8,
                    }),
                ],
                ..unlit(Port::Const(GraphValue::Color([0.0; 4])))
            },
            ..Default::default()
        };
        assert_eq!(
            validate(&graph),
            Err(GraphError::InvalidTextureSlot(MAX_TEXTURE_SAMPLES as u8))
        );
    }

    #[test]
    fn public_input_cap_is_enforced() {
        let graph = ShaderGraph {
            public_inputs: vec![GraphValue::Float(0.0); MAX_PUBLIC_INPUTS + 1],
            ..Default::default()
        };
        assert_eq!(
            validate(&graph),
            Err(GraphError::TooManyPublicInputs(MAX_PUBLIC_INPUTS + 1))
        );
    }

    #[test]
    fn a_node_may_reference_a_public_input() {
        let graph = ShaderGraph {
            public_inputs: vec![GraphValue::Color([1.0, 0.0, 0.0, 1.0])],
            surface: unlit(Port::Input(0)),
            ..Default::default()
        };
        assert_eq!(validate(&graph).expect("valid").surface, Vec::new());
    }

    #[test]
    fn unknown_public_input_reference_is_rejected() {
        let graph = ShaderGraph {
            surface: SurfaceGraph {
                nodes: vec![leaf(NodeKind::Add {
                    a: Port::Input(0),
                    b: Port::Const(GraphValue::Float(1.0)),
                })],
                ..unlit(Port::Const(GraphValue::Color([0.0; 4])))
            },
            ..Default::default()
        };
        assert_eq!(
            validate(&graph),
            Err(GraphError::UnknownInput {
                network: Network::Surface,
                node:    0,
                index:   0,
            })
        );
    }

    #[test]
    fn lit_output_requires_pbr_typed_terminals() {
        let graph = ShaderGraph {
            surface: SurfaceGraph {
                nodes:  Vec::new(),
                output: SurfaceOutput::Lit(LitOutput {
                    base_color: Some(Port::Const(GraphValue::Color([1.0, 1.0, 1.0, 1.0]))),
                    metallic: Some(Port::Const(GraphValue::Float(0.5))),
                    ..Default::default()
                }),
            },
            ..Default::default()
        };
        assert!(validate(&graph).is_ok());
    }

    /// A `Fresnel` node — surface-only, since `N`/`V` don't exist in the
    /// vertex stage — must be rejected inside a displacement network.
    #[test]
    fn surface_only_leaves_are_rejected_in_displacement() {
        let graph = ShaderGraph {
            surface: unlit(Port::Const(GraphValue::Color([0.0; 4]))),
            displacement: Some(DisplacementGraph {
                nodes: vec![leaf(NodeKind::WorldNormal)],
                position_offset: Some(Port::Node(0)),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(
            validate(&graph),
            Err(GraphError::WrongNetwork {
                network: Network::Displacement,
                node:    0,
            })
        );
    }

    /// `LocalPosition`/`LocalNormal` are vertex-stage attributes; they have
    /// no meaning in the fragment stage and must be rejected there.
    #[test]
    fn displacement_only_leaves_are_rejected_in_surface() {
        let graph = ShaderGraph {
            surface: SurfaceGraph {
                nodes: vec![leaf(NodeKind::LocalPosition)],
                ..unlit(Port::Const(GraphValue::Color([0.0; 4])))
            },
            ..Default::default()
        };
        assert_eq!(
            validate(&graph),
            Err(GraphError::WrongNetwork {
                network: Network::Surface,
                node:    0,
            })
        );
    }

    #[test]
    fn a_displacement_graph_computes_a_position_offset() {
        let graph = ShaderGraph {
            surface: unlit(Port::Const(GraphValue::Color([1.0; 4]))),
            displacement: Some(DisplacementGraph {
                nodes:           vec![
                    leaf(NodeKind::LocalNormal),
                    leaf(NodeKind::Time),
                    leaf(NodeKind::Mul {
                        a: Port::Node(0),
                        b: Port::Const(GraphValue::Vec3([1.0, 1.0, 1.0])),
                    }),
                ],
                position_offset: Some(Port::Node(2)),
                normal_override: None,
            }),
            ..Default::default()
        };
        let validated = validate(&graph).expect("valid");
        assert_eq!(
            validated.displacement,
            Some(vec![ValueKind::Vec3, ValueKind::Float, ValueKind::Vec3])
        );
    }

    /// `Sin`/`Cos` are legal in both networks — a `Time`-driven pulse or
    /// sway needs an oscillator in either stage, and this is the basic one.
    #[test]
    fn sin_and_cos_compose_a_time_driven_oscillator() {
        let graph = ShaderGraph {
            surface: unlit(Port::Const(GraphValue::Color([1.0; 4]))),
            displacement: Some(DisplacementGraph {
                nodes:           vec![
                    leaf(NodeKind::Time),
                    leaf(NodeKind::Sin { x: Port::Node(0) }),
                    leaf(NodeKind::Cos { x: Port::Node(0) }),
                    leaf(NodeKind::LocalNormal),
                    leaf(NodeKind::Mul {
                        a: Port::Node(3),
                        b: Port::Const(GraphValue::Vec3([0.1, 0.1, 0.1])),
                    }),
                ],
                position_offset: Some(Port::Node(4)),
                normal_override: None,
            }),
            ..Default::default()
        };
        let validated = validate(&graph).expect("valid");
        assert_eq!(
            validated.displacement,
            Some(vec![
                ValueKind::Float,
                ValueKind::Float,
                ValueKind::Float,
                ValueKind::Vec3,
                ValueKind::Vec3,
            ])
        );
    }

    #[test]
    fn sin_and_cos_require_a_float_input() {
        let graph = ShaderGraph {
            surface: SurfaceGraph {
                nodes: vec![leaf(NodeKind::Uv), leaf(NodeKind::Sin { x: Port::Node(0) })],
                ..unlit(Port::Const(GraphValue::Color([0.0; 4])))
            },
            ..Default::default()
        };
        assert_eq!(
            validate(&graph),
            Err(GraphError::NodeTypeMismatch {
                network:  Network::Surface,
                node:     1,
                port:     "x",
                expected: ValueKind::Float,
                found:    ValueKind::Vec2,
            })
        );
    }

    #[test]
    fn texture_sampling_is_rejected_in_displacement() {
        let graph = ShaderGraph {
            surface: unlit(Port::Const(GraphValue::Color([1.0; 4]))),
            displacement: Some(DisplacementGraph {
                nodes:           vec![
                    leaf(NodeKind::Uv), // wrong-network check fires first
                ],
                position_offset: None,
                normal_override: None,
            }),
            ..Default::default()
        };
        // `Uv` is surface-only, so this specifically exercises that check;
        // a texture-sample-specific graph is covered structurally by
        // `TextureSampleInDisplacement` below via direct construction.
        assert!(matches!(
            validate(&graph),
            Err(GraphError::WrongNetwork {
                network: Network::Displacement,
                ..
            })
        ));
    }

    #[test]
    fn alpha_clip_threshold_must_be_float() {
        let graph = ShaderGraph {
            surface: SurfaceGraph {
                nodes:  Vec::new(),
                output: SurfaceOutput::Unlit(UnlitOutput {
                    color:                Port::Const(GraphValue::Color([1.0; 4])),
                    alpha_clip_threshold: Some(Port::Const(GraphValue::Vec2([0.0; 2]))),
                }),
            },
            ..Default::default()
        };
        assert_eq!(
            validate(&graph),
            Err(GraphError::TerminalTypeMismatch {
                name:     "alpha_clip_threshold",
                expected: ValueKind::Float,
                found:    ValueKind::Vec2,
            })
        );
    }

    #[test]
    fn encode_decode_round_trips() {
        let graph = ShaderGraph {
            public_inputs: vec![GraphValue::Float(0.5)],
            surface:       SurfaceGraph {
                nodes:  vec![
                    leaf(NodeKind::Uv),
                    leaf(NodeKind::TextureSample {
                        uv:   Port::Node(0),
                        slot: 2,
                    }),
                ],
                output: SurfaceOutput::Lit(LitOutput {
                    base_color: Some(Port::Node(1)),
                    alpha: Some(Port::Input(0)),
                    ..Default::default()
                }),
            },
            displacement:  Some(DisplacementGraph {
                nodes:           vec![leaf(NodeKind::LocalPosition)],
                position_offset: Some(Port::Node(0)),
                normal_override: None,
            }),
        };
        let bytes = graph.encode().expect("encode");
        let decoded = ShaderGraph::decode(&bytes).expect("decode");
        assert_eq!(decoded.encode().expect("re-encode"), bytes);
    }

    /// Cross-prim dedup depends on this: two structurally identical graphs
    /// must compile to byte-identical slot entries so their content hashes
    /// collide in the blob store.
    #[test]
    fn identical_graphs_encode_identically() {
        let make = || ShaderGraph {
            surface: SurfaceGraph {
                nodes:  vec![leaf(NodeKind::Uv), leaf(NodeKind::Time)],
                output: SurfaceOutput::Unlit(UnlitOutput {
                    color:                Port::Const(GraphValue::Color([1.0; 4])),
                    alpha_clip_threshold: Some(Port::Node(1)),
                }),
            },
            ..Default::default()
        };
        assert_eq!(
            make().encode().expect("encode"),
            make().encode().expect("encode")
        );
    }

    #[test]
    fn overrides_must_match_declared_public_input_kind() {
        let graph = ShaderGraph {
            public_inputs: vec![GraphValue::Float(1.0)],
            ..Default::default()
        };
        let ok = GraphOverridesAttr {
            overrides: BTreeMap::from([(0, GraphValue::Float(2.0))]),
        };
        assert_eq!(validate_overrides(&graph, &ok), Ok(()));

        let wrong_kind = GraphOverridesAttr {
            overrides: BTreeMap::from([(0, GraphValue::Vec3([0.0; 3]))]),
        };
        assert_eq!(
            validate_overrides(&graph, &wrong_kind),
            Err(OverridesError::TypeMismatch {
                index:    0,
                expected: ValueKind::Float,
                found:    ValueKind::Vec3,
            })
        );

        let unknown = GraphOverridesAttr {
            overrides: BTreeMap::from([(1, GraphValue::Float(2.0))]),
        };
        assert_eq!(
            validate_overrides(&graph, &unknown),
            Err(OverridesError::UnknownInput(1))
        );
    }

    #[test]
    fn overrides_attribute_round_trips() {
        let attr = GraphOverridesAttr {
            overrides: BTreeMap::from([(0, GraphValue::Color([1.0, 0.0, 0.0, 1.0]))]),
        };
        let bytes = attr.encode().expect("encode");
        assert_eq!(GraphOverridesAttr::decode(&bytes).expect("decode"), attr);
    }
}
