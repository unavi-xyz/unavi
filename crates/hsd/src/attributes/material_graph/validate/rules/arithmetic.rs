use super::super::{
    ctx::Ctx,
    error::GraphError,
};
use crate::attributes::material_graph::{
    node::Node,
    value::ValueKind,
};

/// `Add`/`Sub`/`Mul`/`Div` either pair two operands of one kind or broadcast a
/// `Float` against a vector.
pub(super) fn kind(ctx: &Ctx, node: &Node) -> Result<ValueKind, GraphError> {
    match *node {
        Node::Add { a, b } | Node::Sub { a, b } | Node::Mul { a, b } | Node::Div { a, b } => {
            ctx.arithmetic(a, b)
        }
        _ => unreachable!("only the dispatch match in rules/mod.rs reaches here"),
    }
}
