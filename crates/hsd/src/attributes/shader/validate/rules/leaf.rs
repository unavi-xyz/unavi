use super::super::{
    ctx::Ctx,
    error::GraphError,
};
use crate::attributes::shader::{
    node::{
        Network,
        Node,
    },
    value::ValueKind,
};

pub(super) fn kind(_ctx: &Ctx, node: &Node) -> ValueKind {
    match *node {
        Node::Uv | Node::ScreenUv => ValueKind::Vec2,
        Node::WorldNormal
        | Node::WorldPosition
        | Node::LocalPosition
        | Node::LocalNormal
        | Node::ObjectPosition
        | Node::ObjectScale
        | Node::ViewDirection => ValueKind::Vec3,
        Node::VertexColor => ValueKind::Color,
        Node::Time | Node::InstanceRandom => ValueKind::Float,
        _ => unreachable!("only the dispatch match in rules/mod.rs reaches here"),
    }
}

/// Rejects a leaf built-in used outside the network it is defined in.
pub(super) const fn check_network_leaf(
    network: Network,
    index: usize,
    node: &Node,
) -> Result<(), GraphError> {
    let surface_only = matches!(
        node,
        Node::Uv
            | Node::WorldNormal
            | Node::WorldPosition
            | Node::VertexColor
            | Node::ViewDirection
            | Node::ScreenUv
    ) || matches!(node, Node::Fresnel { .. } | Node::SceneColor { .. });
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
