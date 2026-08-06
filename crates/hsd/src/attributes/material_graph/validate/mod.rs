mod ctx;
pub mod error;
mod rules;
mod terminal;

use self::{
    ctx::Ctx,
    error::GraphError,
    rules::{
        check_network_leaf,
        node_output_kind,
    },
    terminal::{
        check_terminal,
        displacement_terminals,
        lit_terminals,
        unlit_terminals,
    },
};
use crate::attributes::material_graph::{
    MAX_NODES,
    MAX_PUBLIC_INPUTS,
    MAX_TEXTURE_SAMPLES,
    ShaderGraph,
    graph::{
        DisplacementGraph,
        SurfaceGraph,
        SurfaceOutput,
    },
    node::{
        Network,
        Node,
    },
    value::{
        GraphValue,
        ValueKind,
        is_finite,
    },
};

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
            for (name, port, expected) in lit_terminals(lit) {
                check_terminal(
                    Network::Surface,
                    name,
                    port,
                    expected,
                    public_inputs,
                    &kinds,
                )?;
            }
        }
        SurfaceOutput::Unlit(unlit) => {
            for (name, port, expected) in unlit_terminals(unlit) {
                check_terminal(
                    Network::Surface,
                    name,
                    port,
                    expected,
                    public_inputs,
                    &kinds,
                )?;
            }
        }
    }
    Ok(kinds)
}

fn validate_displacement(
    displacement: &DisplacementGraph,
    public_inputs: &[GraphValue],
) -> Result<Vec<ValueKind>, GraphError> {
    let kinds = validate_network(Network::Displacement, &displacement.nodes, public_inputs)?;
    for (name, port, expected) in displacement_terminals(displacement) {
        check_terminal(
            Network::Displacement,
            name,
            port,
            expected,
            public_inputs,
            &kinds,
        )?;
    }
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

        let ctx = Ctx {
            network,
            public_inputs,
            at: index,
            kinds: &kinds,
        };
        kinds.push(node_output_kind(&ctx, node)?);
    }

    if texture_samples > MAX_TEXTURE_SAMPLES {
        return Err(GraphError::TooManyTextureSamples(texture_samples));
    }

    Ok(kinds)
}
