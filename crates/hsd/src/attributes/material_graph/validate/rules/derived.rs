use super::super::{
    ctx::Ctx,
    error::GraphError,
};
use crate::attributes::material_graph::{
    node::Node,
    value::ValueKind,
};

/// The nodes that compile to a small fixed expression rather than to one WGSL
/// builtin. Every one of them is expressible with the arithmetic and channel
/// nodes already in the format; each is here because the hand-built form runs
/// to five or more nodes of a 128-node budget and reads as arithmetic rather
/// than as the thing it means.
pub(super) fn kind(ctx: &Ctx, node: &Node) -> Result<ValueKind, GraphError> {
    match *node {
        Node::Remap {
            x,
            from_low,
            from_high,
            to_low,
            to_high,
        } => ctx.all_matching(
            x,
            &[
                ("from-low", from_low),
                ("from-high", from_high),
                ("to-low", to_low),
                ("to-high", to_high),
            ],
        ),
        Node::TriangleWave { x } => {
            ctx.require("x", x, ValueKind::Float)?;
            Ok(ValueKind::Float)
        }
        Node::Luminance { color } => {
            let found = ctx.port_kind(color)?;
            if matches!(found, ValueKind::Vec3 | ValueKind::Color) {
                Ok(ValueKind::Float)
            } else {
                Err(ctx.mismatch("color", ValueKind::Vec3, found))
            }
        }
        Node::PolarCoords { uv, center } => {
            ctx.require("uv", uv, ValueKind::Vec2)?;
            ctx.require("center", center, ValueKind::Vec2)?;
            Ok(ValueKind::Vec2)
        }
        Node::RotateUv {
            uv,
            center,
            radians,
        } => {
            ctx.require("uv", uv, ValueKind::Vec2)?;
            ctx.require("center", center, ValueKind::Vec2)?;
            ctx.require("radians", radians, ValueKind::Float)?;
            Ok(ValueKind::Vec2)
        }
        _ => unreachable!("only the dispatch match in rules/mod.rs reaches here"),
    }
}
