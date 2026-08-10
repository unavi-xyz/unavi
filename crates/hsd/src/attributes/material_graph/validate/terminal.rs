use super::{
    ctx::Ctx,
    error::GraphError,
};
use crate::attributes::material_graph::{
    graph::{
        DisplacementGraph,
        LitOutput,
        UnlitOutput,
    },
    node::{
        Network,
        Port,
    },
    value::{
        GraphValue,
        ValueKind,
    },
};

/// The `Lit` terminals as `(name, port, expected kind)` triples, mirroring
/// the codegen-side terminal list.
pub(super) fn lit_terminals(
    lit: &LitOutput,
) -> impl Iterator<Item = (&'static str, Port, ValueKind)> {
    [
        (lit.base_color, "base_color", ValueKind::Color),
        (lit.emissive, "emissive", ValueKind::Vec3),
        (lit.metallic, "metallic", ValueKind::Float),
        (lit.roughness, "roughness", ValueKind::Float),
        (lit.normal, "normal", ValueKind::Vec3),
        (lit.alpha, "alpha", ValueKind::Float),
        (
            lit.alpha_clip_threshold,
            "alpha_clip_threshold",
            ValueKind::Float,
        ),
    ]
    .into_iter()
    .filter_map(|(port, name, expected)| port.map(|port| (name, port, expected)))
}

/// The `Unlit` terminals as `(name, port, expected kind)` triples.
pub(super) fn unlit_terminals(
    unlit: &UnlitOutput,
) -> impl Iterator<Item = (&'static str, Port, ValueKind)> {
    [
        (Some(unlit.color), "color", ValueKind::Color),
        (
            unlit.alpha_clip_threshold,
            "alpha_clip_threshold",
            ValueKind::Float,
        ),
    ]
    .into_iter()
    .filter_map(|(port, name, expected)| port.map(|port| (name, port, expected)))
}

/// The displacement terminals as `(name, port, expected kind)` triples.
pub(super) fn displacement_terminals(
    displacement: &DisplacementGraph,
) -> impl Iterator<Item = (&'static str, Port, ValueKind)> {
    [
        (
            displacement.position_offset,
            "position_offset",
            ValueKind::Vec3,
        ),
        (
            displacement.normal_override,
            "normal_override",
            ValueKind::Vec3,
        ),
        (
            displacement.world_position_offset,
            "world_position_offset",
            ValueKind::Vec3,
        ),
    ]
    .into_iter()
    .filter_map(|(port, name, expected)| port.map(|port| (name, port, expected)))
}

pub(super) fn check_terminal(
    network: Network,
    name: &'static str,
    port: Port,
    expected: ValueKind,
    public_inputs: &[GraphValue],
    kinds: &[ValueKind],
) -> Result<(), GraphError> {
    let ctx = Ctx {
        network,
        public_inputs,
        at: kinds.len(),
        kinds,
    };
    let found = ctx.port_kind(port).map_err(|err| terminal_err(name, err))?;
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
