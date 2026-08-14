use std::fmt::Write;

use hsd::attributes::material_graph::{
    node::Node,
    value::GraphValue,
};

use super::port_expr;

/// The host-provided terms: their WGSL comes from how the shader host wires
/// them, not from pure builtins.
pub(super) fn emit(out: &mut String, public_inputs: &[GraphValue], node: &Node) {
    match *node {
        Node::Fresnel { power } => {
            // `N`/`V` are plain locals declared by both fragment templates,
            // not `pbr_input` fields: `Unlit` never constructs a `PbrInput`,
            // so Fresnel needs a normal/view pair independent of the PBR
            // lighting path.
            out.push_str("pow(clamp(1.0 - dot(N, V), 0.0, 1.0), ");
            port_expr(out, public_inputs, power);
            out.push(')');
        }
        Node::Noise { uv } => {
            out.push_str("graph_noise(");
            port_expr(out, public_inputs, uv);
            out.push(')');
        }
        Node::TextureSample { uv, slot } => {
            let _ = write!(out, "textureSample(tex_{slot}, samp_{slot}, ");
            port_expr(out, public_inputs, uv);
            out.push(')');
        }
        Node::SceneColor { uv } => {
            // The view-wide texture holding what was drawn before this
            // surface. Bound unconditionally by `mesh_view_bindings`, but only
            // filled for a material that asked to be drawn in the transmissive
            // phase — which `ShaderGraphMaterial` does exactly when a graph
            // contains this node.
            out.push_str(
                "textureSampleLevel(view_transmission_texture, view_transmission_sampler, ",
            );
            port_expr(out, public_inputs, uv);
            out.push_str(", 0.0)");
        }
        Node::Select { cond, a, b } => {
            out.push_str("select(");
            port_expr(out, public_inputs, b);
            out.push_str(", ");
            port_expr(out, public_inputs, a);
            out.push_str(", ");
            port_expr(out, public_inputs, cond);
            out.push_str(" > 0.5)");
        }
        _ => unreachable!("only the dispatch match in expr/mod.rs reaches here"),
    }
}
