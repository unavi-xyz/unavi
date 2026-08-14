use super::super::{
    ctx::Ctx,
    error::GraphError,
};
use crate::attributes::material_graph::{
    node::Node,
    value::ValueKind,
};

/// The host-provided terms: their rules come from how the shader host wires
/// them, not from pure WGSL semantics.
pub(super) fn kind(ctx: &Ctx, node: &Node) -> Result<ValueKind, GraphError> {
    match *node {
        Node::Fresnel { power } => {
            ctx.require("power", power, ValueKind::Float)?;
            Ok(ValueKind::Float)
        }
        Node::Noise { uv } => {
            ctx.require("uv", uv, ValueKind::Vec2)?;
            Ok(ValueKind::Float)
        }
        Node::TextureSample { uv, .. } | Node::SceneColor { uv } => {
            ctx.require("uv", uv, ValueKind::Vec2)?;
            Ok(ValueKind::Color)
        }
        Node::Select { cond, a, b } => {
            ctx.require("cond", cond, ValueKind::Float)?;
            ctx.matching(a, "b", b)
        }
        _ => unreachable!("only the dispatch match in rules/mod.rs reaches here"),
    }
}
