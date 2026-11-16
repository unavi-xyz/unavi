use std::fmt::{
    Result,
    Write,
};

use hsd::attributes::shader::{
    node::Node,
    value::GraphValue,
};

use super::port_expr;

/// The host-provided terms: their WGSL comes from how the shader host wires
/// them, not from pure builtins.
pub(super) fn emit(out: &mut String, public_inputs: &[GraphValue], node: &Node) -> Result {
    match *node {
        Node::Fresnel { power } => {
            // `N`/`V` are plain locals declared by both fragment templates,
            // not `pbr_input` fields (`Unlit` never constructs one).
            out.push_str("pow(clamp(1.0 - dot(N, V), 0.0, 1.0), ");
            port_expr(out, public_inputs, power)?;
            out.push(')');
        }
        Node::Noise { uv } => {
            out.push_str("graph_noise(");
            port_expr(out, public_inputs, uv)?;
            out.push(')');
        }
        Node::TextureSample { uv, slot } => {
            write!(out, "textureSample(tex_{slot}, samp_{slot}, ")?;
            port_expr(out, public_inputs, uv)?;
            out.push(')');
        }
        Node::SceneColor { uv } => {
            // The view-wide texture holding what was drawn before this
            // surface. Filled only for materials in the transmissive phase,
            // which `ShaderGraphMaterial` requests when a graph contains this
            // node; the helper reads black where a view binds none.
            out.push_str("graph_scene_color(");
            port_expr(out, public_inputs, uv)?;
            out.push(')');
        }
        Node::Select { cond, a, b } => {
            out.push_str("select(");
            port_expr(out, public_inputs, b)?;
            out.push_str(", ");
            port_expr(out, public_inputs, a)?;
            out.push_str(", ");
            port_expr(out, public_inputs, cond)?;
            out.push_str(" > 0.5)");
        }
        _ => unreachable!("only the dispatch match in expr/mod.rs reaches here"),
    }
    Ok(())
}
