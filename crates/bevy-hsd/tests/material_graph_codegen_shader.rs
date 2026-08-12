mod common;

use bevy_hsd::attributes::material_graph::codegen::{
    body::{
        generate_displacement_body,
        generate_surface_body,
    },
    generate_fragment_shader,
    generate_vertex_shader,
};
use common::{
    const_color,
    const_f,
    const_v3,
    displaced,
    displaced_world,
    graph_with_output,
    input,
    node,
    unlit,
};
use hsd::attributes::material_graph::{
    ShaderGraph,
    graph::{
        DisplacementGraph,
        LitOutput,
        SurfaceGraph,
        SurfaceOutput,
        UnlitOutput,
    },
    node::Node,
    validate::validate,
    value::GraphValue,
};
use naga::front::wgsl;

/// Resolves the `#ifdef`/`#else`/`#endif` a generated body may carry for
/// optional mesh attributes, since plain `naga` has no preprocessor.
/// Generated bodies never nest these, so one flag suffices.
fn expand_ifdefs(source: &str, defined: bool) -> String {
    let mut out = String::new();
    let mut keep = true;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("#ifdef ") {
            keep = defined;
        } else if trimmed == "#else" {
            keep = !defined;
        } else if trimmed == "#endif" {
            keep = true;
        } else if keep {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Wraps a generated surface body in a bare WGSL module with no Bevy
/// preprocessor syntax, so plain `naga` can parse it and malformed WGSL fails
/// `cargo test`. Parses both branches of every `#ifdef`: `VertexOutput.uv` /
/// `.color` do not exist without their shader defs, and an HSD prim supplies
/// mesh attributes individually.
fn assert_surface_valid(body: &str, out_expr: &str) {
    for defined in [true, false] {
        let module = include_str!("harness/surface.wgsl")
            .replace("//#BODY", &expand_ifdefs(body, defined))
            .replace("{OUT_EXPR}", out_expr);

        if let Err(err) = wgsl::parse_str(&module) {
            panic!(
                "generated surface WGSL failed to parse (attributes \
                 defined={defined}):\n{module}\n\nerror: {err}"
            );
        }
    }
}

/// Same idea as [`assert_surface_valid`] for a displacement body.
fn assert_displacement_valid(body: &str) {
    for defined in [true, false] {
        let module = include_str!("harness/displacement.wgsl")
            .replace("//#BODY", &expand_ifdefs(body, defined));
        // The generated body reads `vertex.position`/`vertex.normal` fields,
        // but the placeholder is a bare `vec3<f32>`; rewrite those accesses so
        // the harness stays a single flat local instead of a full `Vertex`
        // redeclaration.
        let module = module
            .replace("vertex.position", "vertex")
            .replace("vertex.normal", "vertex");

        if let Err(err) = wgsl::parse_str(&module) {
            panic!(
                "generated displacement WGSL failed to parse (attributes \
                 defined={defined}):\n{module}\n\nerror: {err}"
            );
        }
    }
}

#[test]
fn empty_unlit_graph_generates_valid_wgsl() {
    let graph = ShaderGraph::default();
    let validated = validate(&graph).expect("valid");
    let body = generate_surface_body(&graph, &validated);
    assert_surface_valid(&body, "out_color");
}

#[test]
fn lit_output_with_every_terminal_generates_valid_wgsl() {
    let graph = graph_with_output(
        vec![Node::Uv, Node::WorldNormal],
        SurfaceOutput::Lit(LitOutput {
            base_color:           Some(const_color([1.0, 1.0, 1.0, 1.0])),
            emissive:             Some(node(1)),
            metallic:             Some(const_f(0.5)),
            roughness:            Some(const_f(0.5)),
            normal:               Some(node(1)),
            alpha:                Some(const_f(1.0)),
            alpha_clip_threshold: Some(const_f(0.1)),
        }),
    );
    let validated = validate(&graph).expect("valid");
    let body = generate_surface_body(&graph, &validated);
    assert!(body.contains("discard"), "{body}");
    assert_surface_valid(&body, "out_base_color");
}

/// Every surface node kind in one graph: a bad emission shows up as a naga
/// parse failure.
#[test]
fn every_surface_node_kind_generates_valid_wgsl() {
    let graph = ShaderGraph {
        public_inputs: vec![GraphValue::Color([1.0, 0.0, 0.0, 1.0])],
        surface:       SurfaceGraph {
            nodes: vec![
                Node::Uv,
                Node::WorldNormal,
                Node::WorldPosition,
                Node::VertexColor,
                Node::Time,
                Node::Add {
                    a: const_f(1.0),
                    b: node(4),
                },
                Node::Mul {
                    a: const_f(2.0),
                    b: node(4),
                },
                Node::Lerp {
                    a: const_f(0.0),
                    b: const_f(1.0),
                    t: node(4),
                },
                Node::Dot {
                    a: node(1),
                    b: node(1),
                },
                Node::Sin { x: node(4) },
                Node::Cos { x: node(4) },
                Node::Fresnel {
                    power: const_f(2.0),
                },
                Node::Noise { uv: node(0) },
                Node::TextureSample {
                    uv:   node(0),
                    slot: 3,
                },
                Node::Select {
                    cond: node(4),
                    a:    input(0),
                    b:    input(0),
                },
            ],
            output: SurfaceOutput::Unlit(UnlitOutput {
                color:                node(14),
                alpha_clip_threshold: Some(node(4)),
            }),
            ..Default::default()
        },
        displacement:  None,
    };
    let validated = validate(&graph).expect("valid");
    let body = generate_surface_body(&graph, &validated);
    assert_surface_valid(&body, "out_color");
}

/// `VertexOutput.uv` only exists under `VERTEX_UVS_A`, and an HSD prim need
/// not supply `UV_0`, so a `Uv` node must degrade rather than fail to compile.
#[test]
fn optional_mesh_attributes_are_guarded_by_their_shader_defs() {
    let graph = graph_with_output(vec![Node::Uv, Node::VertexColor], unlit(node(1)));
    let validated = validate(&graph).expect("valid");
    let body = generate_surface_body(&graph, &validated);
    assert!(body.contains("#ifdef VERTEX_UVS_A"), "{body}");
    assert!(body.contains("#ifdef VERTEX_COLORS"), "{body}");
    assert_surface_valid(&body, "out_color");
}

/// `Time` reads the view-wide `globals` uniform Bevy already updates, not a
/// per-material slot needing a re-upload every frame.
#[test]
fn time_reads_the_view_globals_uniform() {
    let graph = graph_with_output(
        vec![Node::Time],
        SurfaceOutput::Unlit(UnlitOutput {
            color:                const_color([1.0, 1.0, 1.0, 1.0]),
            alpha_clip_threshold: Some(node(0)),
        }),
    );
    let validated = validate(&graph).expect("valid");
    let body = generate_surface_body(&graph, &validated);
    assert!(body.contains("globals.time"), "{body}");
    assert!(!body.contains("params.time"), "{body}");
    assert_surface_valid(&body, "out_color");
}

#[test]
fn public_input_swizzles_down_to_its_declared_kind() {
    let graph = ShaderGraph {
        public_inputs: vec![GraphValue::Float(1.0)],
        surface:       SurfaceGraph {
            nodes: Vec::new(),
            output: SurfaceOutput::Unlit(UnlitOutput {
                color:                const_color([1.0, 1.0, 1.0, 1.0]),
                alpha_clip_threshold: Some(input(0)),
            }),
            ..Default::default()
        },
        displacement:  None,
    };
    let validated = validate(&graph).expect("valid");
    let body = generate_surface_body(&graph, &validated);
    assert!(body.contains("params.inputs[0].x"), "{body}");
    assert_surface_valid(&body, "out_color");
}

#[test]
fn fragment_shader_splices_the_body_and_declares_bevy_imports() {
    let graph = ShaderGraph::default();
    let validated = validate(&graph).expect("valid");
    let source = generate_fragment_shader(&graph, &validated);
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
    let graph = graph_with_output(Vec::new(), SurfaceOutput::Lit(LitOutput::default()));
    let validated = validate(&graph).expect("valid");
    let source = generate_fragment_shader(&graph, &validated);
    assert!(source.contains("apply_pbr_lighting"));
    assert!(source.contains("pbr_input_from_vertex_output"));
}

#[test]
fn displacement_body_generates_valid_wgsl() {
    let graph = displaced(vec![Node::LocalNormal, Node::Time], Some(node(0)));
    let validated = validate(&graph).expect("valid");
    let body = generate_displacement_body(&graph, &validated).expect("has displacement");
    assert_displacement_valid(&body);
}

#[test]
fn a_sin_driven_displacement_body_generates_valid_wgsl() {
    let graph = displaced(
        vec![
            Node::Time,
            Node::Sin { x: node(0) },
            Node::Mul {
                a: node(1),
                b: const_f(0.15),
            },
            Node::LocalNormal,
            Node::Lerp {
                a: const_v3([0.0, 0.0, 0.0]),
                b: node(3),
                t: node(2),
            },
        ],
        Some(node(4)),
    );
    let validated = validate(&graph).expect("valid");
    let body = generate_displacement_body(&graph, &validated).expect("has displacement");
    assert!(body.contains("sin("), "{body}");
    assert_displacement_valid(&body);
}

#[test]
fn vertex_shader_splices_the_displacement_body() {
    let graph = displaced(vec![Node::LocalPosition], Some(node(0)));
    let validated = validate(&graph).expect("valid");
    let source = generate_vertex_shader(&graph, &validated).expect("has displacement");
    assert!(source.contains("#import bevy_pbr"));
    assert!(source.contains("vertex.position += out_position_offset;"));
    assert!(source.contains("mesh_position_local_to_world"));
}

/// The physgun beam's rope sag: a parabola in local `y` driving a constant
/// world-down offset. A local-space offset could not express it — the beam
/// prim is stretched and rotated between two points, so no local vector
/// stays world-down.
#[test]
fn a_world_space_sag_body_generates_valid_wgsl() {
    let graph = displaced_world(
        vec![
            Node::LocalPosition,
            Node::Extract {
                v:       node(0),
                channel: 1,
            },
            Node::Mul {
                a: node(1),
                b: node(1),
            },
            Node::Mul {
                a: node(2),
                b: const_f(4.0),
            },
            Node::OneMinus { x: node(3) },
            Node::Saturate { x: node(4) },
            Node::Mul {
                a: node(5),
                b: const_f(-0.35),
            },
            Node::Combine3 {
                x: const_f(0.0),
                y: node(6),
                z: const_f(0.0),
            },
        ],
        Some(node(7)),
    );
    let validated = validate(&graph).expect("valid");
    let body = generate_displacement_body(&graph, &validated).expect("has displacement");
    assert!(body.contains("out_world_position_offset = n7;"), "{body}");
    assert_displacement_valid(&body);
}

/// World and local offsets compose: the beam sags in world space while
/// wavering in its own local space.
#[test]
fn vertex_shader_applies_the_world_offset_after_the_mesh_transform() {
    let graph = ShaderGraph {
        displacement: Some(DisplacementGraph {
            nodes:                 vec![Node::LocalNormal],
            position_offset:       Some(node(0)),
            normal_override:       None,
            world_position_offset: Some(node(0)),
        }),
        ..Default::default()
    };
    let validated = validate(&graph).expect("valid");
    let source = generate_vertex_shader(&graph, &validated).expect("has displacement");

    let transform = source
        .find("mesh_position_local_to_world")
        .expect("mesh transform");
    let world_offset = source
        .find("+ out_world_position_offset")
        .expect("world offset applied");
    let clip = source
        .find("out.position = position_world_to_clip")
        .expect("clip transform");
    assert!(
        transform < world_offset && world_offset < clip,
        "world offset must land between the mesh and clip transforms:\n{source}"
    );
}
