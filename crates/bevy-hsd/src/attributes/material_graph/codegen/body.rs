//! Emits the per-node `let`s and assembles the fragment/vertex bodies the
//! shader templates splice in.

use std::fmt::Write;

use hsd::attributes::material_graph::{
    ShaderGraph,
    graph::SurfaceOutput,
    node::{
        Node,
        Port,
    },
    validate::Validated,
    value::{
        GraphValue,
        ValueKind,
    },
};

use super::expr::{
    node_expr,
    port_expr,
    wgsl_type,
};

/// The shader def guarding a leaf's mesh attribute, and the value to use when
/// it is absent.
///
/// `VertexOutput`/`Vertex` declare `uv`, `color`, `normal` and `position`
/// behind `#ifdef`s (see `bevy_pbr::forward_io`), and an HSD prim supplies
/// mesh attributes individually — so a graph reading `Uv` on a mesh with no
/// `UV_0` would otherwise fail to compile rather than degrade.
const fn guarded_leaf(node: &Node) -> Option<(&'static str, &'static str, &'static str)> {
    match node {
        Node::Uv => Some(("VERTEX_UVS_A", "in.uv", "vec2<f32>(0.0, 0.0)")),
        Node::VertexColor => Some(("VERTEX_COLORS", "in.color", "vec4<f32>(1.0, 1.0, 1.0, 1.0)")),
        Node::LocalPosition => Some((
            "VERTEX_POSITIONS",
            "vertex.position",
            "vec3<f32>(0.0, 0.0, 0.0)",
        )),
        Node::LocalNormal => Some((
            "VERTEX_NORMALS",
            "vertex.normal",
            "vec3<f32>(0.0, 0.0, 1.0)",
        )),
        _ => None,
    }
}

fn emit_nodes(out: &mut String, public_inputs: &[GraphValue], nodes: &[Node], kinds: &[ValueKind]) {
    for (index, node) in nodes.iter().enumerate() {
        let ty = wgsl_type(kinds[index]);
        if let Some((def, present, absent)) = guarded_leaf(node) {
            let _ = writeln!(
                out,
                "#ifdef {def}\n    let n{index}: {ty} = {present};\n#else\n    let n{index}: {ty} \
                 = {absent};\n#endif"
            );
        } else {
            let _ = write!(out, "    let n{index}: {ty} = ");
            node_expr(out, public_inputs, kinds, node);
            out.push_str(";\n");
        }
    }
}

fn emit_alpha_clip(
    out: &mut String,
    public_inputs: &[GraphValue],
    alpha_expr: &str,
    threshold: Option<Port>,
) {
    if let Some(threshold) = threshold {
        let _ = write!(out, "    if {alpha_expr} < ");
        port_expr(out, public_inputs, threshold);
        out.push_str(" {\n        discard;\n    }\n");
    }
}

/// The fragment-stage body.
///
/// Node `let`s from `SurfaceGraph::nodes`, then either the six `out_*`
/// PBR locals a caller assembles into a `PbrInput` (`Lit`) or a single
/// `out_color` written straight to the fragment output (`Unlit`) — see
/// `SurfaceOutput`'s docs for why there are two shapes rather than one.
#[must_use]
pub fn generate_surface_body(graph: &ShaderGraph, validated: &Validated) -> String {
    let mut out = String::new();
    emit_nodes(
        &mut out,
        &graph.public_inputs,
        &graph.surface.nodes,
        validated.surface(),
    );

    match &graph.surface.output {
        SurfaceOutput::Lit(lit) => {
            out.push_str("    var out_base_color: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 1.0);\n");
            out.push_str("    var out_emissive: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);\n");
            out.push_str("    var out_metallic: f32 = 0.0;\n");
            out.push_str("    var out_roughness: f32 = 0.5;\n");
            out.push_str("    var out_normal: vec3<f32> = pbr_input.world_normal;\n");
            out.push_str("    var out_alpha: f32 = 1.0;\n");

            for (name, port) in [
                ("out_base_color", lit.base_color),
                ("out_emissive", lit.emissive),
                ("out_metallic", lit.metallic),
                ("out_roughness", lit.roughness),
                ("out_normal", lit.normal),
                ("out_alpha", lit.alpha),
            ] {
                if let Some(port) = port {
                    let _ = write!(out, "    {name} = ");
                    port_expr(&mut out, &graph.public_inputs, port);
                    out.push_str(";\n");
                }
            }

            emit_alpha_clip(
                &mut out,
                &graph.public_inputs,
                "out_alpha",
                lit.alpha_clip_threshold,
            );
        }
        SurfaceOutput::Unlit(unlit) => {
            let _ = write!(out, "    var out_color: vec4<f32> = ");
            port_expr(&mut out, &graph.public_inputs, unlit.color);
            out.push_str(";\n");
            emit_alpha_clip(
                &mut out,
                &graph.public_inputs,
                "out_color.a",
                unlit.alpha_clip_threshold,
            );
        }
    }

    out
}

/// The vertex-stage body.
///
/// Node `let`s from `DisplacementGraph::nodes`, then
/// `out_position_offset`/`out_normal_override` locals a caller adds to
/// `vertex.position`/replaces `vertex.normal` with, before the standard
/// mesh transform runs.
/// `None` for a graph with no displacement network.
#[must_use]
pub fn generate_displacement_body(graph: &ShaderGraph, validated: &Validated) -> Option<String> {
    let displacement = graph.displacement.as_ref()?;
    let kinds = validated.displacement()?;
    let public_inputs = &graph.public_inputs;

    let mut out = String::new();
    emit_nodes(&mut out, public_inputs, &displacement.nodes, kinds);

    out.push_str("    var out_position_offset: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);\n");
    out.push_str(
        "#ifdef VERTEX_NORMALS\n    var out_normal_override: vec3<f32> = \
         vertex.normal;\n#else\n    var out_normal_override: vec3<f32> = vec3<f32>(0.0, 0.0, \
         1.0);\n#endif\n",
    );

    if let Some(port) = displacement.position_offset {
        let _ = write!(out, "    out_position_offset = ");
        port_expr(&mut out, public_inputs, port);
        out.push_str(";\n");
    }
    if let Some(port) = displacement.normal_override {
        let _ = write!(out, "    out_normal_override = ");
        port_expr(&mut out, public_inputs, port);
        out.push_str(";\n");
    }

    Some(out)
}
