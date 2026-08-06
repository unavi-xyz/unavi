use thiserror::Error;

use crate::attributes::material_graph::{
    DisplacementGraph,
    GraphValue,
    MAX_NODES,
    MAX_PUBLIC_INPUTS,
    MAX_TEXTURE_SAMPLES,
    Network,
    Node,
    Port,
    ShaderGraph,
    SurfaceGraph,
    SurfaceOutput,
    ValueKind,
};

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
    #[error("{network:?} network node {node} holds a non-finite constant")]
    NonFiniteConst { network: Network, node: usize },
    #[error("public input {0} holds a non-finite value")]
    NonFinitePublicInput(usize),
    #[error("terminal {0} holds a non-finite constant")]
    NonFiniteTerminal(&'static str),
}

/// The per-node-index output kinds of a validated network, one entry per
/// network present.
///
/// Fields are private and there is no public constructor, so holding one is
/// proof [`validate`] accepted the graph it came from. Codegen takes this
/// rather than a loose `&[ValueKind]`, which a caller could otherwise pair
/// with the wrong graph and index out of bounds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Validated {
    surface:      Vec<ValueKind>,
    displacement: Option<Vec<ValueKind>>,
}

impl Validated {
    #[must_use]
    pub fn surface(&self) -> &[ValueKind] {
        &self.surface
    }

    #[must_use]
    pub fn displacement(&self) -> Option<&[ValueKind]> {
        self.displacement.as_deref()
    }
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
    for (index, value) in graph.public_inputs.iter().enumerate() {
        if !is_finite(*value) {
            return Err(GraphError::NonFinitePublicInput(index));
        }
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

fn validate_surface(
    surface: &SurfaceGraph,
    public_inputs: &[GraphValue],
) -> Result<Vec<ValueKind>, GraphError> {
    let kinds = validate_network(Network::Surface, &surface.nodes, public_inputs)?;
    match &surface.output {
        SurfaceOutput::Lit(lit) => {
            check_terminal(
                Network::Surface,
                "base_color",
                lit.base_color,
                ValueKind::Color,
                public_inputs,
                &kinds,
            )?;
            check_terminal(
                Network::Surface,
                "emissive",
                lit.emissive,
                ValueKind::Vec3,
                public_inputs,
                &kinds,
            )?;
            check_terminal(
                Network::Surface,
                "metallic",
                lit.metallic,
                ValueKind::Float,
                public_inputs,
                &kinds,
            )?;
            check_terminal(
                Network::Surface,
                "roughness",
                lit.roughness,
                ValueKind::Float,
                public_inputs,
                &kinds,
            )?;
            check_terminal(
                Network::Surface,
                "normal",
                lit.normal,
                ValueKind::Vec3,
                public_inputs,
                &kinds,
            )?;
            check_terminal(
                Network::Surface,
                "alpha",
                lit.alpha,
                ValueKind::Float,
                public_inputs,
                &kinds,
            )?;
            check_terminal(
                Network::Surface,
                "alpha_clip_threshold",
                lit.alpha_clip_threshold,
                ValueKind::Float,
                public_inputs,
                &kinds,
            )?;
        }
        SurfaceOutput::Unlit(unlit) => {
            check_terminal(
                Network::Surface,
                "color",
                Some(unlit.color),
                ValueKind::Color,
                public_inputs,
                &kinds,
            )?;
            check_terminal(
                Network::Surface,
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

fn validate_displacement(
    displacement: &DisplacementGraph,
    public_inputs: &[GraphValue],
) -> Result<Vec<ValueKind>, GraphError> {
    let kinds = validate_network(Network::Displacement, &displacement.nodes, public_inputs)?;
    check_terminal(
        Network::Displacement,
        "position_offset",
        displacement.position_offset,
        ValueKind::Vec3,
        public_inputs,
        &kinds,
    )?;
    check_terminal(
        Network::Displacement,
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
        check_network_leaf(network, index, node)?;

        if let Node::TextureSample { slot, .. } = node {
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
            node,
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
const fn check_network_leaf(network: Network, index: usize, node: &Node) -> Result<(), GraphError> {
    let surface_only = matches!(
        node,
        Node::Uv | Node::WorldNormal | Node::WorldPosition | Node::VertexColor
    ) || matches!(node, Node::Fresnel { .. });
    let displacement_only = matches!(node, Node::LocalPosition | Node::LocalNormal);

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

/// Whether every component is a real number.
///
/// `NaN` and the infinities have no WGSL literal — `f32`'s own formatting
/// renders them `NaN`/`inf`, which no shader compiler accepts — so they are
/// refused here rather than in any one backend.
#[must_use]
pub const fn is_finite(value: GraphValue) -> bool {
    match value {
        GraphValue::Float(v) => v.is_finite(),
        GraphValue::Vec2([x, y]) => x.is_finite() && y.is_finite(),
        GraphValue::Vec3([x, y, z]) => x.is_finite() && y.is_finite() && z.is_finite(),
        GraphValue::Color([r, g, b, a]) => {
            r.is_finite() && g.is_finite() && b.is_finite() && a.is_finite()
        }
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
        Port::Const(value) if !is_finite(value) => {
            Err(GraphError::NonFiniteConst { network, node: at })
        }
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
    node: &Node,
    kinds: &[ValueKind],
) -> Result<ValueKind, GraphError> {
    match *node {
        Node::Uv => Ok(ValueKind::Vec2),
        Node::WorldNormal | Node::WorldPosition | Node::LocalPosition | Node::LocalNormal => {
            Ok(ValueKind::Vec3)
        }
        Node::VertexColor => Ok(ValueKind::Color),
        Node::Time => Ok(ValueKind::Float),
        Node::Add { a, b } | Node::Mul { a, b } => {
            matching(network, public_inputs, at, a, "b", b, kinds)
        }
        Node::Lerp { a, b, t } => {
            let value_kind = matching(network, public_inputs, at, a, "b", b, kinds)?;
            require(network, public_inputs, at, "t", t, kinds, ValueKind::Float)?;
            Ok(value_kind)
        }
        Node::Dot { a, b } => {
            matching(network, public_inputs, at, a, "b", b, kinds)?;
            Ok(ValueKind::Float)
        }
        Node::Sin { x } | Node::Cos { x } => {
            require(network, public_inputs, at, "x", x, kinds, ValueKind::Float)?;
            Ok(ValueKind::Float)
        }
        Node::Fresnel { power } => {
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
        Node::Noise { uv } => {
            require(network, public_inputs, at, "uv", uv, kinds, ValueKind::Vec2)?;
            Ok(ValueKind::Float)
        }
        Node::TextureSample { uv, .. } => {
            require(network, public_inputs, at, "uv", uv, kinds, ValueKind::Vec2)?;
            Ok(ValueKind::Color)
        }
        Node::Select { cond, a, b } => {
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
    network: Network,
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
    let found = port_kind(network, public_inputs, kinds.len(), port, kinds)
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
        GraphError::NonFiniteConst { .. } => GraphError::NonFiniteTerminal(name),
        other => other,
    }
}
