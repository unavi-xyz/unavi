use super::super::{
    ctx::Ctx,
    error::GraphError,
};
use crate::attributes::material_graph::{
    node::{
        Network,
        Node,
    },
    value::ValueKind,
};

/// The zero-arity leaves, each legal in exactly one network.
pub(super) fn kind(_ctx: &Ctx, node: &Node) -> ValueKind {
    match *node {
        Node::Uv => ValueKind::Vec2,
        Node::WorldNormal | Node::WorldPosition | Node::LocalPosition | Node::LocalNormal => {
            ValueKind::Vec3
        }
        Node::VertexColor => ValueKind::Color,
        Node::Time => ValueKind::Float,
        _ => unreachable!("only the dispatch match in rules/mod.rs reaches here"),
    }
}

/// Rejects a leaf built-in used outside the one network it is defined in —
/// `Uv`/`WorldNormal`/`WorldPosition`/`VertexColor`/`Fresnel` are
/// fragment-stage varyings that do not exist in the vertex stage,
/// `LocalPosition`/`LocalNormal` are vertex-stage attributes that have no
/// meaning post-rasterization. `Time` and every non-leaf node kind are
/// legal in both.
pub(super) const fn check_network_leaf(
    network: Network,
    index: usize,
    node: &Node,
) -> Result<(), GraphError> {
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
