mod arithmetic;
mod builtin;
mod channel;
mod derived;
mod leaf;
mod sample;

use super::{
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

pub(super) const fn check_network_leaf(
    network: Network,
    index: usize,
    node: &Node,
) -> Result<(), GraphError> {
    self::leaf::check_network_leaf(network, index, node)
}

/// The per-node output-kind rule: a single exhaustive match over [`Node`] that
/// only dispatches to the family owning each variant, so adding a node makes
/// the compiler name this file.
pub(super) fn node_output_kind(ctx: &Ctx, node: &Node) -> Result<ValueKind, GraphError> {
    match *node {
        Node::Add { .. } | Node::Sub { .. } | Node::Mul { .. } | Node::Div { .. } => {
            arithmetic::kind(ctx, node)
        }
        Node::Lerp { .. }
        | Node::Dot { .. }
        | Node::Sin { .. }
        | Node::Cos { .. }
        | Node::OneMinus { .. }
        | Node::Abs { .. }
        | Node::Floor { .. }
        | Node::Fract { .. }
        | Node::Saturate { .. }
        | Node::Sqrt { .. }
        | Node::Pow { .. }
        | Node::Min { .. }
        | Node::Max { .. }
        | Node::Clamp { .. }
        | Node::Step { .. }
        | Node::Smoothstep { .. }
        | Node::Length { .. }
        | Node::Normalize { .. }
        | Node::Atan2 { .. }
        | Node::Modulo { .. }
        | Node::Distance { .. }
        | Node::Cross { .. } => builtin::kind(ctx, node),
        Node::Remap { .. }
        | Node::TriangleWave { .. }
        | Node::Luminance { .. }
        | Node::PolarCoords { .. }
        | Node::RotateUv { .. } => derived::kind(ctx, node),
        Node::Combine2 { .. }
        | Node::Combine3 { .. }
        | Node::Combine4 { .. }
        | Node::Extract { .. }
        | Node::Convert { .. } => channel::kind(ctx, node),
        Node::Uv
        | Node::WorldNormal
        | Node::WorldPosition
        | Node::VertexColor
        | Node::LocalPosition
        | Node::LocalNormal
        | Node::InstanceRandom
        | Node::ObjectPosition
        | Node::ObjectScale
        | Node::ViewDirection
        | Node::Time => Ok(leaf::kind(ctx, node)),
        Node::Fresnel { .. }
        | Node::Noise { .. }
        | Node::TextureSample { .. }
        | Node::Select { .. } => sample::kind(ctx, node),
    }
}
