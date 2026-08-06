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
    wgsl_type,
};

/// The type-system crossings: assembling vectors out of scalars, reading one
/// component out, and converting between vector kinds.
pub(super) fn emit(
    out: &mut String,
    public_inputs: &[GraphValue],
    kinds: &[ValueKind],
    node: &Node,
) {
    match *node {
        Node::Extract { v, channel } => {
            out.push('(');
            port_expr(out, public_inputs, v);
            let _ = write!(out, ").{}", channel_name(channel));
        }
        Node::Combine2 { x, y } => {
            out.push_str("vec2<f32>(");
            port_expr(out, public_inputs, x);
            out.push_str(", ");
            port_expr(out, public_inputs, y);
            out.push(')');
        }
        Node::Combine3 { x, y, z } => {
            out.push_str("vec3<f32>(");
            port_expr(out, public_inputs, x);
            out.push_str(", ");
            port_expr(out, public_inputs, y);
            out.push_str(", ");
            port_expr(out, public_inputs, z);
            out.push(')');
        }
        Node::Combine4 { x, y, z, w } => {
            out.push_str("vec4<f32>(");
            port_expr(out, public_inputs, x);
            out.push_str(", ");
            port_expr(out, public_inputs, y);
            out.push_str(", ");
            port_expr(out, public_inputs, z);
            out.push_str(", ");
            port_expr(out, public_inputs, w);
            out.push(')');
        }
        Node::Convert { v, to } => convert(out, public_inputs, kinds, v, to),
        _ => unreachable!("only the dispatch match in expr/mod.rs reaches here"),
    }
}

/// `.x`/`.xy`/`.xyz` for a narrowing [`Node::Convert`].
const fn swizzle(components: u8) -> &'static str {
    match components {
        1 => "x",
        2 => "xy",
        3 => "xyz",
        _ => "xyzw",
    }
}

const fn channel_name(channel: u8) -> &'static str {
    match channel {
        0 => "x",
        1 => "y",
        2 => "z",
        _ => "w",
    }
}

fn convert(
    out: &mut String,
    public_inputs: &[GraphValue],
    kinds: &[ValueKind],
    v: Port,
    to: ValueKind,
) {
    let (from, to_count) = (
        port_kind(public_inputs, kinds, v).components(),
        to.components(),
    );
    if to_count <= from {
        if to_count == from {
            port_expr(out, public_inputs, v);
        } else {
            out.push('(');
            port_expr(out, public_inputs, v);
            let _ = write!(out, ").{}", swizzle(to_count));
        }
    } else {
        // A widened color's alpha is 1.0, not the zero every other padded
        // component gets: a zero-padded color would be fully transparent.
        let _ = write!(out, "{}(", wgsl_type(to));
        port_expr(out, public_inputs, v);
        for pad in 0..to_count - from {
            let value = if to == ValueKind::Color && pad == to_count - from - 1 {
                "1.0"
            } else {
                "0.0"
            };
            let _ = write!(out, ", {value}");
        }
        out.push(')');
    }
}
