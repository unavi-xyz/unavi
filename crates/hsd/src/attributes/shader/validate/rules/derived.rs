use super::super::{
    ctx::Ctx,
    error::GraphError,
};
use crate::attributes::shader::{
    node::Node,
    value::ValueKind,
};

/// Nodes that compile to a handful of primitive nodes rather than one WGSL
/// builtin.
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
