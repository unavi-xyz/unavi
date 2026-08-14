use hsd::attributes::material_graph::{
    node::Node,
    value::GraphValue,
};

use super::port_expr;

/// The nodes that compile to a small fixed expression rather than to one WGSL
/// builtin. Each is a handful of bounded ALU ops, so the cost argument is
/// unchanged: no loop, no branch, no unbounded work.
pub(super) fn emit(out: &mut String, public_inputs: &[GraphValue], node: &Node) {
    match *node {
        Node::Remap {
            x,
            from_low,
            from_high,
            to_low,
            to_high,
        } => {
            out.push('(');
            port_expr(out, public_inputs, to_low);
            out.push_str(" + (");
            port_expr(out, public_inputs, x);
            out.push_str(" - ");
            port_expr(out, public_inputs, from_low);
            out.push_str(") * (");
            port_expr(out, public_inputs, to_high);
            out.push_str(" - ");
            port_expr(out, public_inputs, to_low);
            out.push_str(") / (");
            port_expr(out, public_inputs, from_high);
            out.push_str(" - ");
            port_expr(out, public_inputs, from_low);
            out.push_str("))");
        }
        Node::TriangleWave { x } => {
            out.push_str("(1.0 - abs(fract(");
            port_expr(out, public_inputs, x);
            out.push_str(") * 2.0 - 1.0))");
        }
        Node::Luminance { color } => {
            out.push_str("dot(");
            port_expr(out, public_inputs, color);
            out.push_str(".rgb, vec3<f32>(0.2126, 0.7152, 0.0722))");
        }
        Node::PolarCoords { uv, center } => {
            out.push_str("graph_polar(");
            port_expr(out, public_inputs, uv);
            out.push_str(", ");
            port_expr(out, public_inputs, center);
            out.push(')');
        }
        Node::RotateUv {
            uv,
            center,
            radians,
        } => {
            out.push_str("graph_rotate_uv(");
            port_expr(out, public_inputs, uv);
            out.push_str(", ");
            port_expr(out, public_inputs, center);
            out.push_str(", ");
            port_expr(out, public_inputs, radians);
            out.push(')');
        }
        _ => unreachable!("only the dispatch match in expr/mod.rs reaches here"),
    }
}
