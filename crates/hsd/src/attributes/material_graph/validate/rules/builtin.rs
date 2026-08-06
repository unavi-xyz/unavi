use super::super::{
    ctx::Ctx,
    error::GraphError,
};
use crate::attributes::material_graph::{
    node::Node,
    value::ValueKind,
};

/// The WGSL-builtin-backed nodes: matching operand kinds, with a few
/// fixed-kind or vector-required ports.
pub(super) fn kind(ctx: &Ctx, node: &Node) -> Result<ValueKind, GraphError> {
    match *node {
        Node::Lerp { a, b, t } => {
            let kind = ctx.matching(a, "b", b)?;
            ctx.require("t", t, ValueKind::Float)?;
            Ok(kind)
        }
        Node::Dot { a, b } => {
            ctx.matching(a, "b", b)?;
            Ok(ValueKind::Float)
        }
        Node::Sin { x } | Node::Cos { x } => {
            ctx.require("x", x, ValueKind::Float)?;
            Ok(ValueKind::Float)
        }
        Node::OneMinus { x }
        | Node::Abs { x }
        | Node::Floor { x }
        | Node::Fract { x }
        | Node::Saturate { x }
        | Node::Sqrt { x } => ctx.port_kind(x),
        Node::Pow { x, y } => ctx.matching(x, "y", y),
        Node::Min { a, b } | Node::Max { a, b } => ctx.matching(a, "b", b),
        Node::Clamp { x, low, high } => ctx.all_matching(x, &[("low", low), ("high", high)]),
        Node::Step { edge, x } => ctx.matching(edge, "x", x),
        Node::Smoothstep { low, high, x } => ctx.all_matching(low, &[("high", high), ("x", x)]),
        Node::Length { v } => {
            ctx.vector_port("v", v)?;
            Ok(ValueKind::Float)
        }
        Node::Normalize { v } => ctx.vector_port("v", v),
        Node::Cross { a, b } => {
            ctx.require("a", a, ValueKind::Vec3)?;
            ctx.require("b", b, ValueKind::Vec3)?;
            Ok(ValueKind::Vec3)
        }
        _ => unreachable!("only the dispatch match in rules/mod.rs reaches here"),
    }
}
