//! Compiles a validated [`ShaderGraph`] to WGSL, client-side, at load time.
//!
//! Never ships pre-generated WGSL over the network: the graph is the only
//! thing that travels; every peer that loads it generates its own source
//! text from the same validated data. A graph compiles to up to two shader
//! stages — a fragment body from [`SurfaceGraph`] (in a `Lit` or `Unlit`
//! shape) and, if present, a vertex body from [`DisplacementGraph`] — never
//! one fixed PBR-only body, which is what makes this a genuine WGSL
//! replacement rather than a parameter tinter.

use std::fmt::Write;

use hsd::attributes::material_graph::{
    DisplacementGraph,
    NodeKind,
    Port,
    ShaderGraph,
    SurfaceOutput,
    ValueKind,
};

pub const MAX_TEXTURE_SAMPLES: usize = hsd::attributes::material_graph::MAX_TEXTURE_SAMPLES;
pub const MAX_PUBLIC_INPUTS: usize = hsd::attributes::material_graph::MAX_PUBLIC_INPUTS;

const fn wgsl_type(kind: ValueKind) -> &'static str {
    match kind {
        ValueKind::Float => "f32",
        ValueKind::Vec2 => "vec2<f32>",
        ValueKind::Vec3 => "vec3<f32>",
        ValueKind::Color => "vec4<f32>",
    }
}

/// `{:?}` rather than `{}`: `f32`'s `Debug` always prints a decimal point
/// (`2.0`, not `2`), which a bare integer is not in WGSL.
fn literal(value: hsd::attributes::material_graph::GraphValue) -> String {
    use hsd::attributes::material_graph::GraphValue;
    match value {
        GraphValue::Float(v) => format!("{v:?}"),
        GraphValue::Vec2([x, y]) => format!("vec2<f32>({x:?}, {y:?})"),
        GraphValue::Vec3([x, y, z]) => format!("vec3<f32>({x:?}, {y:?}, {z:?})"),
        GraphValue::Color([r, g, b, a]) => format!("vec4<f32>({r:?}, {g:?}, {b:?}, {a:?})"),
    }
}

/// A public input is stored as a full `vec4` slot; only the components its
/// declared kind actually uses are meaningful, so a lower-kind reference
/// swizzles down to them.
fn port_expr(public_inputs: &[hsd::attributes::material_graph::GraphValue], port: Port) -> String {
    match port {
        Port::Const(value) => literal(value),
        Port::Input(index) => {
            let kind = public_inputs[usize::from(index)].kind();
            let slot = format!("params.inputs[{index}]");
            match kind {
                ValueKind::Float => format!("{slot}.x"),
                ValueKind::Vec2 => format!("{slot}.xy"),
                ValueKind::Vec3 => format!("{slot}.xyz"),
                ValueKind::Color => slot,
            }
        }
        Port::Node(index) => format!("n{index}"),
    }
}

fn node_expr(
    public_inputs: &[hsd::attributes::material_graph::GraphValue],
    kind: &NodeKind,
) -> String {
    match *kind {
        NodeKind::Uv => "in.uv".to_owned(),
        NodeKind::WorldNormal => "pbr_input.world_normal".to_owned(),
        NodeKind::WorldPosition => "in.world_position.xyz".to_owned(),
        NodeKind::VertexColor => "in.color".to_owned(),
        NodeKind::LocalPosition => "vertex.position".to_owned(),
        NodeKind::LocalNormal => "vertex.normal".to_owned(),
        NodeKind::Time => "params.time".to_owned(),
        NodeKind::Add { a, b } => format!(
            "({} + {})",
            port_expr(public_inputs, a),
            port_expr(public_inputs, b)
        ),
        NodeKind::Mul { a, b } => format!(
            "({} * {})",
            port_expr(public_inputs, a),
            port_expr(public_inputs, b)
        ),
        NodeKind::Lerp { a, b, t } => format!(
            "mix({}, {}, {})",
            port_expr(public_inputs, a),
            port_expr(public_inputs, b),
            port_expr(public_inputs, t)
        ),
        NodeKind::Dot { a, b } => format!(
            "dot({}, {})",
            port_expr(public_inputs, a),
            port_expr(public_inputs, b)
        ),
        NodeKind::Sin { x } => format!("sin({})", port_expr(public_inputs, x)),
        NodeKind::Cos { x } => format!("cos({})", port_expr(public_inputs, x)),
        // `N`/`V` are plain locals declared by both fragment templates
        // (`generate_fragment_shader`), not `pbr_input.N`/`.V` — `Unlit`
        // never constructs a `PbrInput` at all, so Fresnel needs a normal/
        // view pair that exists independent of the PBR lighting path.
        NodeKind::Fresnel { power } => format!(
            "pow(clamp(1.0 - dot(N, V), 0.0, 1.0), {})",
            port_expr(public_inputs, power)
        ),
        NodeKind::Noise { uv } => format!("graph_noise({})", port_expr(public_inputs, uv)),
        NodeKind::TextureSample { uv, slot } => format!(
            "textureSample(tex_{slot}, samp_{slot}, {})",
            port_expr(public_inputs, uv)
        ),
        NodeKind::Select { cond, a, b } => format!(
            "select({}, {}, {} > 0.5)",
            port_expr(public_inputs, b),
            port_expr(public_inputs, a),
            port_expr(public_inputs, cond)
        ),
    }
}

fn emit_nodes(
    out: &mut String,
    public_inputs: &[hsd::attributes::material_graph::GraphValue],
    nodes: &[hsd::attributes::material_graph::Node],
    kinds: &[ValueKind],
) {
    for (index, node) in nodes.iter().enumerate() {
        let ty = wgsl_type(kinds[index]);
        let expr = node_expr(public_inputs, &node.kind);
        let _ = writeln!(out, "    let n{index}: {ty} = {expr};");
    }
}

fn emit_alpha_clip(
    out: &mut String,
    public_inputs: &[hsd::attributes::material_graph::GraphValue],
    alpha_expr: &str,
    threshold: Option<Port>,
) {
    if let Some(threshold) = threshold {
        let _ = writeln!(
            out,
            "    if {alpha_expr} < {} {{\n        discard;\n    }}",
            port_expr(public_inputs, threshold)
        );
    }
}

/// The fragment-stage body.
///
/// Node `let`s from [`SurfaceGraph::nodes`], then either the six `out_*`
/// PBR locals a caller assembles into a `PbrInput` (`Lit`) or a single
/// `out_color` written straight to the fragment output (`Unlit`) — see
/// `SurfaceOutput`'s docs for why there are two shapes rather than one.
#[must_use]
pub fn generate_surface_body(graph: &ShaderGraph, kinds: &[ValueKind]) -> String {
    let mut out = String::new();
    emit_nodes(&mut out, &graph.public_inputs, &graph.surface.nodes, kinds);

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
                    let _ = writeln!(
                        out,
                        "    {name} = {};",
                        port_expr(&graph.public_inputs, port)
                    );
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
            let _ = writeln!(
                out,
                "    var out_color: vec4<f32> = {};",
                port_expr(&graph.public_inputs, unlit.color)
            );
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
/// Node `let`s from [`DisplacementGraph::nodes`], then
/// `out_position_offset`/`out_normal_override` locals a caller adds to
/// `vertex.position`/replaces `vertex.normal` with, before the standard
/// mesh transform runs.
#[must_use]
pub fn generate_displacement_body(
    displacement: &DisplacementGraph,
    public_inputs: &[hsd::attributes::material_graph::GraphValue],
    kinds: &[ValueKind],
) -> String {
    let mut out = String::new();
    emit_nodes(&mut out, public_inputs, &displacement.nodes, kinds);

    out.push_str("    var out_position_offset: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);\n");
    out.push_str("    var out_normal_override: vec3<f32> = vertex.normal;\n");

    if let Some(port) = displacement.position_offset {
        let _ = writeln!(
            out,
            "    out_position_offset = {};",
            port_expr(public_inputs, port)
        );
    }
    if let Some(port) = displacement.normal_override {
        let _ = writeln!(
            out,
            "    out_normal_override = {};",
            port_expr(public_inputs, port)
        );
    }

    out
}

/// Value-noise helpers the generated body calls into; defined once per
/// shader rather than inlined per [`NodeKind::Noise`] node.
const NOISE_FUNCTIONS: &str = "\
fn graph_hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453123);
}

fn graph_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let a = graph_hash(i);
    let b = graph_hash(i + vec2<f32>(1.0, 0.0));
    let c = graph_hash(i + vec2<f32>(0.0, 1.0));
    let d = graph_hash(i + vec2<f32>(1.0, 1.0));
    let u = f * f * (3.0 - 2.0 * f);
    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}
";

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
        "struct GraphParams {{\n    inputs: array<vec4<f32>, {MAX_PUBLIC_INPUTS}>,\n    time: f32,\n}};\n\n@group(#{{MATERIAL_BIND_GROUP}}) @binding(0)\nvar<uniform> params: GraphParams;\n"
    )
}

/// The full fragment shader.
///
/// Bevy's own `PbrInput` construction and lighting
/// (`pbr_input_from_vertex_output`/`apply_pbr_lighting`) wrap the generated
/// body for `Lit`; `Unlit` skips both and writes `out_color` straight to
/// the fragment output — mirroring Unreal's Unlit shading model ("only
/// outputs Emissive Color") and Unity's Unlit Master Stack target. Not
/// naga-tested standalone — the `#import`s are Bevy's shader-preprocessor
/// syntax, not plain WGSL — see [`generate_surface_body`]'s tests for the
/// part that is.
#[must_use]
pub fn generate_fragment_shader(graph: &ShaderGraph, kinds: &[ValueKind]) -> String {
    let body = generate_surface_body(graph, kinds);
    let preamble = format!(
        "{uniform}\n{textures}\n{noise}",
        uniform = uniform_block(),
        textures = texture_bindings(),
        noise = NOISE_FUNCTIONS,
    );

    match &graph.surface.output {
        SurfaceOutput::Lit(_) => format!(
            "#import bevy_pbr::{{\n    forward_io::{{VertexOutput, FragmentOutput}},\n    pbr_fragment::pbr_input_from_vertex_output,\n    pbr_functions::{{apply_pbr_lighting, main_pass_post_lighting_processing, alpha_discard}},\n}}\n\n\
             {preamble}\n\
             @fragment\n\
             fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {{\n\
             \x20   var pbr_input = pbr_input_from_vertex_output(in, is_front, false);\n\
             \x20   let N = pbr_input.N;\n\
             \x20   let V = pbr_input.V;\n\
             \n\
             {body}\n\
             \x20   pbr_input.material.base_color = vec4<f32>(out_base_color.rgb, out_alpha);\n\
             \x20   pbr_input.material.emissive = vec4<f32>(out_emissive, 1.0);\n\
             \x20   pbr_input.material.metallic = out_metallic;\n\
             \x20   pbr_input.material.perceptual_roughness = out_roughness;\n\
             \x20   pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);\n\
             \x20   pbr_input.world_normal = out_normal;\n\
             \x20   pbr_input.N = normalize(out_normal);\n\
             \n\
             \x20   var out: FragmentOutput;\n\
             \x20   out.color = apply_pbr_lighting(pbr_input);\n\
             \x20   out.color = main_pass_post_lighting_processing(pbr_input, out.color);\n\
             \x20   return out;\n\
             }}\n"
        ),
        // No `PbrInput`/lighting pass, but `Fresnel` still needs a normal/
        // view pair — computed directly from `VertexOutput` rather than
        // via the (unconstructed) `PbrInput`, the same values
        // `pbr_input_from_vertex_output` would have produced.
        SurfaceOutput::Unlit(_) => format!(
            "#import bevy_pbr::{{\n    forward_io::{{VertexOutput, FragmentOutput}},\n    mesh_view_bindings::view,\n    pbr_functions::calculate_view,\n}}\n\n\
             {preamble}\n\
             @fragment\n\
             fn fragment(in: VertexOutput) -> FragmentOutput {{\n\
             \x20   let is_orthographic = view.clip_from_view[3].w == 1.0;\n\
             \x20   let N = normalize(in.world_normal);\n\
             \x20   let V = calculate_view(in.world_position, is_orthographic);\n\
             \n\
             {body}\n\
             \x20   var out: FragmentOutput;\n\
             \x20   out.color = out_color;\n\
             \x20   return out;\n\
             }}\n"
        ),
    }
}

/// The full vertex shader, generated only when a graph has a
/// [`DisplacementGraph`].
///
/// Modeled directly on `bevy_pbr`'s own default `mesh.wgsl` vertex function
/// — fetch `vertex.position`/`vertex.normal`, splice the displacement body
/// in, then run the same `mesh_functions::mesh_position_local_to_world` /
/// `view_transformations::position_world_to_clip` calls it already does —
/// rather than reimplementing mesh-transform logic. Skinning/morph targets
/// are out of scope for v1; this targets the static-mesh path only.
#[must_use]
pub fn generate_vertex_shader(
    displacement: &DisplacementGraph,
    public_inputs: &[hsd::attributes::material_graph::GraphValue],
    kinds: &[ValueKind],
) -> String {
    let body = generate_displacement_body(displacement, public_inputs, kinds);
    format!(
        "#import bevy_pbr::{{\n    forward_io::{{Vertex, VertexOutput}},\n    mesh_functions,\n    view_transformations::position_world_to_clip,\n}}\n\
         \n\
         {uniform}\n\
         {noise}\n\
         @vertex\n\
         fn vertex(vertex_in: Vertex) -> VertexOutput {{\n\
         \x20   var vertex = vertex_in;\n\
         \x20   var out: VertexOutput;\n\
         \n\
         {body}\n\
         \x20   vertex.position += out_position_offset;\n\
         \x20   vertex.normal = out_normal_override;\n\
         \n\
         \x20   let world_from_local = mesh_functions::get_world_from_local(vertex_in.instance_index);\n\
         \x20   out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal, vertex_in.instance_index);\n\
         \x20   out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));\n\
         \x20   out.position = position_world_to_clip(out.world_position.xyz);\n\
         \x20   out.uv = vertex.uv;\n\
         \n\
         \x20   return out;\n\
         }}\n",
        uniform = uniform_block(),
        noise = NOISE_FUNCTIONS,
    )
}

#[cfg(test)]
mod tests {
    use hsd::attributes::material_graph::{
        GraphValue,
        LitOutput,
        Node,
        SurfaceGraph,
        UnlitOutput,
        validate,
    };
    use naga::front::wgsl;

    use super::*;

    /// Wraps a generated surface body in a bare WGSL module that declares
    /// everything the body can reference (`in.uv`/`in.world_position`/
    /// `in.color`, `pbr_input.{world_normal,N,V}`, `params.{inputs,time}`,
    /// `tex_0..3`/`samp_0..3`) without any Bevy preprocessor syntax, so
    /// plain `naga` can parse and validate it. This is the real safety net:
    /// a codegen bug that emits malformed WGSL fails a `cargo test`, not a
    /// manual visual check.
    fn assert_surface_valid(body: &str, out_expr: &str) {
        let module = format!(
            "struct In {{ uv: vec2<f32>, world_position: vec4<f32>, world_normal: vec3<f32>, color: vec4<f32> }}\n\
             struct PbrInput {{ world_normal: vec3<f32> }}\n\
             struct Params {{ inputs: array<vec4<f32>, {MAX_PUBLIC_INPUTS}>, time: f32 }}\n\
             \n\
             @group(0) @binding(0) var<uniform> params: Params;\n\
             {tex}\n\
             {NOISE_FUNCTIONS}\n\
             @fragment\n\
             fn fragment(in: In) -> @location(0) vec4<f32> {{\n\
             \x20   var pbr_input: PbrInput;\n\
             \x20   let N = in.world_normal;\n\
             \x20   let V = in.world_normal;\n\
             {body}\n\
             \x20   return {out_expr};\n\
             }}\n",
            tex = texture_bindings_with_group(0),
        );

        if let Err(err) = wgsl::parse_str(&module) {
            panic!("generated surface WGSL failed to parse:\n{module}\n\nerror: {err}");
        }
    }

    /// Same idea as [`assert_surface_valid`] for a displacement body: a bare
    /// harness declaring `vertex.{position,normal}` and `params.*`.
    fn assert_displacement_valid(body: &str) {
        let module = format!(
            "struct Params {{ inputs: array<vec4<f32>, {MAX_PUBLIC_INPUTS}>, time: f32 }}\n\
             \n\
             @group(0) @binding(0) var<uniform> params: Params;\n\
             {NOISE_FUNCTIONS}\n\
             @fragment\n\
             fn fragment() -> @location(0) vec4<f32> {{\n\
             \x20   var vertex: vec3<f32>;\n\
             {body}\n\
             \x20   return vec4<f32>(out_position_offset, 1.0) + vec4<f32>(out_normal_override, 1.0);\n\
             }}\n",
        );
        // `vertex.position`/`vertex.normal` in the generated body assume a
        // struct, not the placeholder `vec3<f32>` above; rewrite the two
        // field accesses codegen can emit into the placeholder itself so
        // this harness stays a single flat local instead of a full `Vertex`
        // redeclaration.
        let module = module
            .replace("vertex.position", "vertex")
            .replace("vertex.normal", "vertex");

        if let Err(err) = wgsl::parse_str(&module) {
            panic!("generated displacement WGSL failed to parse:\n{module}\n\nerror: {err}");
        }
    }

    /// [`texture_bindings`] emits Bevy's `#{MATERIAL_BIND_GROUP}` macro,
    /// which plain `naga` cannot expand; the test harness needs a literal
    /// group index instead.
    fn texture_bindings_with_group(group: u32) -> String {
        let mut out = String::new();
        for slot in 0..MAX_TEXTURE_SAMPLES {
            let tex_binding = 1 + slot * 2;
            let sampler_binding = tex_binding + 1;
            let _ = writeln!(
                out,
                "@group({group}) @binding({tex_binding})\nvar tex_{slot}: texture_2d<f32>;\n@group({group}) @binding({sampler_binding})\nvar samp_{slot}: sampler;"
            );
        }
        out
    }

    fn leaf(kind: NodeKind) -> Node {
        Node { kind }
    }

    #[test]
    fn empty_unlit_graph_generates_valid_wgsl() {
        let graph = ShaderGraph::default();
        let validated = validate(&graph).expect("valid");
        let body = generate_surface_body(&graph, &validated.surface);
        assert_surface_valid(&body, "out_color");
    }

    #[test]
    fn lit_output_with_every_terminal_generates_valid_wgsl() {
        let graph = ShaderGraph {
            surface: SurfaceGraph {
                nodes:  vec![leaf(NodeKind::Uv), leaf(NodeKind::WorldNormal)],
                output: hsd::attributes::material_graph::SurfaceOutput::Lit(LitOutput {
                    base_color:           Some(Port::Const(GraphValue::Color([1.0; 4]))),
                    emissive:             Some(Port::Node(1)),
                    metallic:             Some(Port::Const(GraphValue::Float(0.5))),
                    roughness:            Some(Port::Const(GraphValue::Float(0.5))),
                    normal:               Some(Port::Node(1)),
                    alpha:                Some(Port::Const(GraphValue::Float(1.0))),
                    alpha_clip_threshold: Some(Port::Const(GraphValue::Float(0.1))),
                }),
            },
            ..Default::default()
        };
        let validated = validate(&graph).expect("valid");
        let body = generate_surface_body(&graph, &validated.surface);
        assert!(body.contains("discard"), "{body}");
        assert_surface_valid(&body, "out_base_color");
    }

    #[test]
    fn every_surface_node_kind_generates_valid_wgsl() {
        let graph = ShaderGraph {
            public_inputs: vec![GraphValue::Color([1.0, 0.0, 0.0, 1.0])],
            surface: SurfaceGraph {
                nodes:  vec![
                    leaf(NodeKind::Uv),
                    leaf(NodeKind::WorldNormal),
                    leaf(NodeKind::WorldPosition),
                    leaf(NodeKind::VertexColor),
                    leaf(NodeKind::Time),
                    leaf(NodeKind::Add {
                        a: Port::Const(GraphValue::Float(1.0)),
                        b: Port::Node(4),
                    }),
                    leaf(NodeKind::Mul {
                        a: Port::Const(GraphValue::Float(2.0)),
                        b: Port::Node(4),
                    }),
                    leaf(NodeKind::Lerp {
                        a: Port::Const(GraphValue::Float(0.0)),
                        b: Port::Const(GraphValue::Float(1.0)),
                        t: Port::Node(4),
                    }),
                    leaf(NodeKind::Dot {
                        a: Port::Node(1),
                        b: Port::Node(1),
                    }),
                    leaf(NodeKind::Sin { x: Port::Node(4) }),
                    leaf(NodeKind::Cos { x: Port::Node(4) }),
                    leaf(NodeKind::Fresnel {
                        power: Port::Const(GraphValue::Float(2.0)),
                    }),
                    leaf(NodeKind::Noise { uv: Port::Node(0) }),
                    leaf(NodeKind::TextureSample {
                        uv:   Port::Node(0),
                        slot: 3,
                    }),
                    leaf(NodeKind::Select {
                        cond: Port::Node(4),
                        a:    Port::Input(0),
                        b:    Port::Input(0),
                    }),
                ],
                output: hsd::attributes::material_graph::SurfaceOutput::Unlit(UnlitOutput {
                    color:                Port::Node(14),
                    alpha_clip_threshold: Some(Port::Node(4)),
                }),
            },
            ..Default::default()
        };
        let validated = validate(&graph).expect("valid");
        let body = generate_surface_body(&graph, &validated.surface);
        assert_surface_valid(&body, "out_color");
    }

    #[test]
    fn public_input_swizzles_down_to_its_declared_kind() {
        let graph = ShaderGraph {
            public_inputs: vec![GraphValue::Float(1.0)],
            surface: SurfaceGraph {
                nodes:  Vec::new(),
                output: hsd::attributes::material_graph::SurfaceOutput::Unlit(UnlitOutput {
                    color:                Port::Const(GraphValue::Color([1.0; 4])),
                    alpha_clip_threshold: Some(Port::Input(0)),
                }),
            },
            ..Default::default()
        };
        let validated = validate(&graph).expect("valid");
        let body = generate_surface_body(&graph, &validated.surface);
        assert!(body.contains("params.inputs[0].x"), "{body}");
        assert_surface_valid(&body, "out_color");
    }

    #[test]
    fn whole_number_floats_still_get_a_decimal_point() {
        assert_eq!(literal(GraphValue::Float(2.0)), "2.0");
    }

    #[test]
    fn fragment_shader_splices_the_body_and_declares_bevy_imports() {
        let graph = ShaderGraph {
            surface: SurfaceGraph {
                nodes:  Vec::new(),
                output: hsd::attributes::material_graph::SurfaceOutput::Unlit(UnlitOutput {
                    color:                Port::Const(GraphValue::Color([1.0; 4])),
                    alpha_clip_threshold: None,
                }),
            },
            ..Default::default()
        };
        let validated = validate(&graph).expect("valid");
        let source = generate_fragment_shader(&graph, &validated.surface);
        assert!(source.contains("#import bevy_pbr"));
        assert!(source.contains("var out_color: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 1.0);"));
        assert!(source.contains("var<uniform> params: GraphParams;"));
        assert!(source.contains("tex_3: texture_2d<f32>"));
        assert!(
            !source.contains("apply_pbr_lighting"),
            "unlit shader must skip PBR lighting"
        );
    }

    #[test]
    fn lit_fragment_shader_uses_pbr_lighting() {
        let graph = ShaderGraph {
            surface: SurfaceGraph {
                nodes:  Vec::new(),
                output: hsd::attributes::material_graph::SurfaceOutput::Lit(LitOutput::default()),
            },
            ..Default::default()
        };
        let validated = validate(&graph).expect("valid");
        let source = generate_fragment_shader(&graph, &validated.surface);
        assert!(source.contains("apply_pbr_lighting"));
        assert!(source.contains("pbr_input_from_vertex_output"));
    }

    #[test]
    fn displacement_body_generates_valid_wgsl() {
        let displacement = DisplacementGraph {
            nodes:           vec![leaf(NodeKind::LocalNormal), leaf(NodeKind::Time)],
            position_offset: Some(Port::Node(0)),
            normal_override: None,
        };
        let public_inputs = Vec::new();
        let validated =
            hsd::attributes::material_graph::validate_displacement(&displacement, &public_inputs)
                .expect("valid");
        let body = generate_displacement_body(&displacement, &public_inputs, &validated);
        assert_displacement_valid(&body);
    }

    /// The oscillator a pulsing/swaying displacement effect needs — see the
    /// `bevy-hsd` example, which uses exactly this shape to animate a sphere.
    #[test]
    fn a_sin_driven_displacement_body_generates_valid_wgsl() {
        let displacement = DisplacementGraph {
            nodes:           vec![
                leaf(NodeKind::Time),
                leaf(NodeKind::Sin { x: Port::Node(0) }),
                // `Mul` needs matching kinds (no vector*scalar scaling), so
                // scale the scalar first, then drive `Lerp`'s `t` with it.
                leaf(NodeKind::Mul {
                    a: Port::Node(1),
                    b: Port::Const(GraphValue::Float(0.15)),
                }),
                leaf(NodeKind::LocalNormal),
                leaf(NodeKind::Lerp {
                    a: Port::Const(GraphValue::Vec3([0.0, 0.0, 0.0])),
                    b: Port::Node(3),
                    t: Port::Node(2),
                }),
            ],
            position_offset: Some(Port::Node(4)),
            normal_override: None,
        };
        let public_inputs = Vec::new();
        let validated =
            hsd::attributes::material_graph::validate_displacement(&displacement, &public_inputs)
                .expect("valid");
        let body = generate_displacement_body(&displacement, &public_inputs, &validated);
        assert!(body.contains("sin("), "{body}");
        assert_displacement_valid(&body);
    }

    #[test]
    fn vertex_shader_splices_the_displacement_body() {
        let displacement = DisplacementGraph {
            nodes:           vec![leaf(NodeKind::LocalPosition)],
            position_offset: Some(Port::Node(0)),
            normal_override: None,
        };
        let public_inputs = Vec::new();
        let validated =
            hsd::attributes::material_graph::validate_displacement(&displacement, &public_inputs)
                .expect("valid");
        let source = generate_vertex_shader(&displacement, &public_inputs, &validated);
        assert!(source.contains("#import bevy_pbr"));
        assert!(source.contains("vertex.position += out_position_offset;"));
        assert!(source.contains("mesh_position_local_to_world"));
    }
}
