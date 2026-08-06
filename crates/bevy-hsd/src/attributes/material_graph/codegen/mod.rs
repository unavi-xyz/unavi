//! Compiles a validated [`ShaderGraph`] to WGSL, client-side, at load time.
//!
//! Never ships pre-generated WGSL over the network: the graph is the only
//! thing that travels; every peer that loads it generates its own source
//! text from the same validated data. A graph compiles to up to two shader
//! stages — a fragment body from `SurfaceGraph` (in a `Lit` or `Unlit`
//! shape) and, if present, a vertex body from `DisplacementGraph` — never
//! one fixed PBR-only body, which is what makes this a genuine WGSL
//! replacement rather than a parameter tinter.

pub mod body;

mod expr;

use std::fmt::Write;

use body::{
    generate_displacement_body,
    generate_surface_body,
};
use hsd::attributes::material_graph::{
    MAX_PUBLIC_INPUTS,
    MAX_TEXTURE_SAMPLES,
    ShaderGraph,
    graph::SurfaceOutput,
    validate::Validated,
};

/// Value-noise helpers the generated body calls into; defined once per
/// shader rather than inlined per `Noise` node.
const NOISE_FUNCTIONS: &str = include_str!("templates/noise.wgsl");

fn texture_bindings() -> String {
    let mut out = String::new();
    for slot in 0..MAX_TEXTURE_SAMPLES {
        let tex_binding = 1 + slot * 2;
        let sampler_binding = tex_binding + 1;
        let _ = writeln!(
            out,
            "@group(#{{MATERIAL_BIND_GROUP}}) @binding({tex_binding})\nvar tex_{slot}: texture_2d<f32>;\n@group(#{{MATERIAL_BIND_GROUP}}) @binding({sampler_binding})\nvar samp_{slot}: sampler;"
        );
    }
    out
}

fn uniform_block() -> String {
    format!(
        "struct GraphParams {{\n    inputs: array<vec4<f32>, {MAX_PUBLIC_INPUTS}>,\n}};\n\n@group(#{{MATERIAL_BIND_GROUP}}) @binding(0)\nvar<uniform> params: GraphParams;\n"
    )
}

/// Splices a generated body and uniform preamble into a `.wgsl` template at
/// its `//#PREAMBLE`/`//#BODY` line-comment markers, keeping the templates
/// readable and diffable.
fn splice(template: &str, body: &str, preamble: &str) -> String {
    template
        .replace("//#PREAMBLE", preamble)
        .replace("//#BODY", body)
}

/// The full fragment shader.
///
/// Bevy's own `PbrInput` construction and lighting
/// (`pbr_input_from_vertex_output`/`apply_pbr_lighting`) wrap the generated
/// body for `Lit`; `Unlit` skips both and writes `out_color` straight to
/// the fragment output — mirroring Unreal's Unlit shading model ("only
/// outputs Emissive Color") and Unity's Unlit Master Stack target. Not
/// naga-tested standalone — the `#import`s are Bevy's shader-preprocessor
/// syntax, not plain WGSL; the crate's integration tests wrap the bodies in
/// a bare harness instead.
#[must_use]
pub fn generate_fragment_shader(graph: &ShaderGraph, validated: &Validated) -> String {
    let body = generate_surface_body(graph, validated);
    let preamble = format!(
        "{uniform}\n{textures}\n{noise}",
        uniform = uniform_block(),
        textures = texture_bindings(),
        noise = NOISE_FUNCTIONS,
    );

    match &graph.surface.output {
        SurfaceOutput::Lit(_) => splice(
            include_str!("templates/fragment_lit.wgsl"),
            &body,
            &preamble,
        ),
        SurfaceOutput::Unlit(_) => splice(
            include_str!("templates/fragment_unlit.wgsl"),
            &body,
            &preamble,
        ),
    }
}

/// The full vertex shader, generated only when a graph has a
/// `DisplacementGraph`.
///
/// Modeled directly on `bevy_pbr`'s own default `mesh.wgsl` vertex function
/// — fetch `vertex.position`/`vertex.normal`, splice the displacement body
/// in, then run the same `mesh_functions::mesh_position_local_to_world` /
/// `view_transformations::position_world_to_clip` calls it already does —
/// rather than reimplementing mesh-transform logic. Skinning/morph targets
/// are out of scope for v1; this targets the static-mesh path only.
///
/// `None` for a graph with no displacement network, where the mesh pipeline's
/// own vertex shader runs unmodified.
#[must_use]
pub fn generate_vertex_shader(graph: &ShaderGraph, validated: &Validated) -> Option<String> {
    let body = generate_displacement_body(graph, validated)?;
    let preamble = format!(
        "{uniform}\n{noise}",
        uniform = uniform_block(),
        noise = NOISE_FUNCTIONS,
    );
    Some(splice(
        include_str!("templates/vertex.wgsl"),
        &body,
        &preamble,
    ))
}
