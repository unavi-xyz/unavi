use std::fmt::Write;

use hsd::attributes::material_graph::{
    node::{
        Node,
        Port,
    },
    value::{
        GraphValue,
        ValueKind,
    },
};

use super::{
    port_expr,
    port_kind,
    zero_literal,
};

/// The WGSL-builtin-backed nodes: matching operand kinds, one builtin per
/// node. `Sqrt`/`Pow` additionally clamp negative operands away rather than
/// letting them produce `NaN`.
pub(super) fn emit(
    out: &mut String,
    public_inputs: &[GraphValue],
    kinds: &[ValueKind],
    node: &Node,
) {
    match *node {
        Node::Lerp { a, b, t } => ternary(out, public_inputs, "mix", a, b, t),
        Node::Dot { a, b } => binary(out, public_inputs, "dot", a, b),
        Node::Sin { x } => unary(out, public_inputs, "sin", x),
        Node::Cos { x } => unary(out, public_inputs, "cos", x),
        Node::OneMinus { x } => {
            out.push_str("(1.0 - ");
            port_expr(out, public_inputs, x);
            out.push(')');
        }
        Node::Abs { x } => unary(out, public_inputs, "abs", x),
        Node::Floor { x } => unary(out, public_inputs, "floor", x),
        Node::Fract { x } => unary(out, public_inputs, "fract", x),
        Node::Saturate { x } => unary(out, public_inputs, "saturate", x),
        Node::Sqrt { x } => {
            out.push_str("sqrt(max(");
            port_expr(out, public_inputs, x);
            out.push_str(", ");
            zero_literal(out, port_kind(public_inputs, kinds, x));
            out.push_str("))");
        }
        Node::Pow { x, y } => {
            out.push_str("pow(max(");
            port_expr(out, public_inputs, x);
            out.push_str(", ");
            zero_literal(out, port_kind(public_inputs, kinds, x));
            out.push_str("), ");
            port_expr(out, public_inputs, y);
            out.push(')');
        }
        Node::Min { a, b } => binary(out, public_inputs, "min", a, b),
        Node::Max { a, b } => binary(out, public_inputs, "max", a, b),
        Node::Clamp { x, low, high } => {
            out.push_str("clamp(");
            port_expr(out, public_inputs, x);
            out.push_str(", ");
            port_expr(out, public_inputs, low);
            out.push_str(", ");
            port_expr(out, public_inputs, high);
            out.push(')');
        }
        Node::Step { edge, x } => {
            out.push_str("step(");
            port_expr(out, public_inputs, edge);
            out.push_str(", ");
            port_expr(out, public_inputs, x);
            out.push(')');
        }
        Node::Smoothstep { low, high, x } => {
            out.push_str("smoothstep(");
            port_expr(out, public_inputs, low);
            out.push_str(", ");
            port_expr(out, public_inputs, high);
            out.push_str(", ");
            port_expr(out, public_inputs, x);
            out.push(')');
        }
        Node::Length { v } => unary(out, public_inputs, "length", v),
        Node::Normalize { v } => unary(out, public_inputs, "normalize", v),
        Node::Cross { a, b } => binary(out, public_inputs, "cross", a, b),
        _ => unreachable!("only the dispatch match in expr/mod.rs reaches here"),
    }
}

fn unary(out: &mut String, public_inputs: &[GraphValue], f: &str, x: Port) {
    let _ = write!(out, "{f}(");
    port_expr(out, public_inputs, x);
    out.push(')');
}

fn binary(out: &mut String, public_inputs: &[GraphValue], f: &str, a: Port, b: Port) {
    let _ = write!(out, "{f}(");
    port_expr(out, public_inputs, a);
    out.push_str(", ");
    port_expr(out, public_inputs, b);
    out.push(')');
}

fn ternary(out: &mut String, public_inputs: &[GraphValue], f: &str, a: Port, b: Port, t: Port) {
    let _ = write!(out, "{f}(");
    port_expr(out, public_inputs, a);
    out.push_str(", ");
    port_expr(out, public_inputs, b);
    out.push_str(", ");
    port_expr(out, public_inputs, t);
    out.push(')');
}
