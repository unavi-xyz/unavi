use std::fmt::{
    Result,
    Write,
};

use hsd::attributes::shader::{
    node::{
        Node,
        Port,
    },
    value::GraphValue,
};

use super::port_expr;

/// WGSL's own `+ - * /` broadcast a `Float` across a vector's components, so
/// this family costs codegen nothing beyond a parenthesized infix pair.
pub(super) fn emit(out: &mut String, public_inputs: &[GraphValue], node: &Node) -> Result {
    match *node {
        Node::Add { a, b } => binary(out, public_inputs, "+", a, b),
        Node::Sub { a, b } => binary(out, public_inputs, "-", a, b),
        Node::Mul { a, b } => binary(out, public_inputs, "*", a, b),
        Node::Div { a, b } => binary(out, public_inputs, "/", a, b),
        _ => unreachable!("only the dispatch match in expr/mod.rs reaches here"),
    }
}

fn binary(out: &mut String, public_inputs: &[GraphValue], op: &str, a: Port, b: Port) -> Result {
    out.push('(');
    port_expr(out, public_inputs, a)?;
    write!(out, " {op} ")?;
    port_expr(out, public_inputs, b)?;
    out.push(')');
    Ok(())
}
