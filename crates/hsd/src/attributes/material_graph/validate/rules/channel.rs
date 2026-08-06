use super::super::{
    ctx::Ctx,
    error::GraphError,
};
use crate::attributes::material_graph::{
    node::Node,
    value::ValueKind,
};

/// The type-system crossings: assembling vectors out of scalars, reading one
/// component out, and converting between vector kinds.
pub(super) fn kind(ctx: &Ctx, node: &Node) -> Result<ValueKind, GraphError> {
    match *node {
        Node::Extract { v, channel } => ctx.extract(v, channel),
        Node::Combine2 { x, y } => ctx.combine(&[("x", x), ("y", y)], ValueKind::Vec2),
        Node::Combine3 { x, y, z } => ctx.combine(&[("x", x), ("y", y), ("z", z)], ValueKind::Vec3),
        Node::Combine4 { x, y, z, w } => {
            ctx.combine(&[("x", x), ("y", y), ("z", z), ("w", w)], ValueKind::Color)
        }
        Node::Convert { v, to } => ctx.convert(v, to),
        _ => unreachable!("only the dispatch match in rules/mod.rs reaches here"),
    }
}
