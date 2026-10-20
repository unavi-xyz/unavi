//! Emits the per-node `let`s and assembles the fragment/vertex bodies the
//! shader templates splice in.

use std::fmt::{
    self,
    Write,
};

use hsd::attributes::shader::{
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
/// behind `#ifdef`s, and an HSD prim supplies mesh attributes individually,
/// so a graph reading `Uv` on a mesh with no `UV_0` degrades instead of
/// failing to compile.
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

fn emit_nodes(
    out: &mut String,
    public_inputs: &[GraphValue],
    nodes: &[Node],
    kinds: &[ValueKind],
) -> fmt::Result {
    for (index, node) in nodes.iter().enumerate() {
        let ty = wgsl_type(kinds[index]);
        if let Some((def, present, absent)) = guarded_leaf(node) {
            writeln!(
                out,
                "#ifdef {def}\n    let n{index}: {ty} = {present};\n#else\n    let n{index}: {ty} \
                 = {absent};\n#endif"
            )?;
        } else {
            write!(out, "    let n{index}: {ty} = ")?;
            node_expr(out, public_inputs, kinds, node)?;
            out.push_str(";\n");
        }
    }
    Ok(())
}

fn emit_alpha_clip(
    out: &mut String,
    public_inputs: &[GraphValue],
    alpha_expr: &str,
    threshold: Option<Port>,
) -> fmt::Result {
    if let Some(threshold) = threshold {
        write!(out, "    if {alpha_expr} < ")?;
        port_expr(out, public_inputs, threshold)?;
        out.push_str(" {\n        discard;\n    }\n");
    }
    Ok(())
}

/// The fragment-stage body: node `let`s, then either the six `out_*` PBR
/// locals a caller assembles into a `PbrInput` (`Lit`) or a single `out_color`
/// (`Unlit`).
pub fn generate_surface_body(
    graph: &ShaderGraph,
    validated: &Validated,
) -> Result<String, fmt::Error> {
    let mut out = String::new();
    emit_nodes(
        &mut out,
        &graph.public_inputs,
        &graph.surface.nodes,
        validated.surface(),
    )?;

    match &graph.surface.output {
        SurfaceOutput::Lit(lit) => {
            out.push_str("    var out_base_color: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 1.0);\n");
            out.push_str("    var out_emissive: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);\n");
            out.push_str("    var out_metallic: f32 = 0.0;\n");
            out.push_str("    var out_roughness: f32 = 0.5;\n");
            out.push_str("    var out_normal: vec3<f32> = graph_world_normal;\n");
            out.push_str("    var out_alpha: f32 = 1.0;\n");
            out.push_str("    var out_specular_transmission: f32 = 0.0;\n");
            out.push_str("    var out_diffuse_transmission: f32 = 0.0;\n");
            out.push_str("    var out_thickness: f32 = 0.0;\n");
            out.push_str("    var out_ior: f32 = 1.5;\n");

            for (name, port) in [
                ("out_base_color", lit.base_color),
                ("out_emissive", lit.emissive),
                ("out_metallic", lit.metallic),
                ("out_roughness", lit.roughness),
                ("out_normal", lit.normal),
                ("out_alpha", lit.alpha),
                ("out_specular_transmission", lit.specular_transmission),
                ("out_diffuse_transmission", lit.diffuse_transmission),
                ("out_thickness", lit.thickness),
                ("out_ior", lit.ior),
            ] {
                if let Some(port) = port {
                    write!(out, "    {name} = ")?;
                    port_expr(&mut out, &graph.public_inputs, port)?;
                    out.push_str(";\n");
                }
            }

            emit_alpha_clip(
                &mut out,
                &graph.public_inputs,
                "out_alpha",
                lit.alpha_clip_threshold,
            )?;
        }
        SurfaceOutput::Unlit(unlit) => {
            write!(out, "    var out_color: vec4<f32> = ")?;
            port_expr(&mut out, &graph.public_inputs, unlit.color)?;
            out.push_str(";\n");
            emit_alpha_clip(
                &mut out,
                &graph.public_inputs,
                "out_color.a",
                unlit.alpha_clip_threshold,
            )?;
        }
    }

    Ok(out)
}

/// The vertex-stage body: node `let`s, then `out_position_offset` /
/// `out_normal_override` locals the caller applies before the mesh transform.
///
/// `None` for a graph with no displacement network.
pub fn generate_displacement_body(
    graph: &ShaderGraph,
    validated: &Validated,
) -> Result<Option<String>, fmt::Error> {
    let (Some(displacement), Some(kinds)) = (graph.displacement.as_ref(), validated.displacement())
    else {
        return Ok(None);
    };
    let public_inputs = &graph.public_inputs;

    let mut out = String::new();
    emit_nodes(&mut out, public_inputs, &displacement.nodes, kinds)?;

    out.push_str("    var out_position_offset: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);\n");
    out.push_str("    var out_world_position_offset: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);\n");
    out.push_str(
        "#ifdef VERTEX_NORMALS\n    var out_normal_override: vec3<f32> = \
         vertex.normal;\n#else\n    var out_normal_override: vec3<f32> = vec3<f32>(0.0, 0.0, \
         1.0);\n#endif\n",
    );

    for (name, port) in [
        ("out_position_offset", displacement.position_offset),
        ("out_normal_override", displacement.normal_override),
        (
            "out_world_position_offset",
            displacement.world_position_offset,
        ),
    ] {
        if let Some(port) = port {
            write!(out, "    {name} = ")?;
            port_expr(&mut out, public_inputs, port)?;
            out.push_str(";\n");
        }
    }

    Ok(Some(out))
}
