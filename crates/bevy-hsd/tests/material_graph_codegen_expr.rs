mod common;

use bevy_hsd::attributes::material_graph::codegen::body::{
    generate_displacement_body,
    generate_surface_body,
};
use common::{
    const_color,
    const_f,
    const_v2,
    const_v3,
    displaced,
    graph,
    node,
};
use hsd::attributes::material_graph::{
    node::Node,
    validate::validate,
    value::ValueKind,
};
use rstest::rstest;

/// The RHS of one node's `let n{index}: … = …;` statement in a generated
/// body — the exact WGSL expression codegen emits for that node.
fn rhs_of(body: &str, index: usize) -> String {
    let needle = format!("let n{index}:");
    let line = body
        .lines()
        .find(|line| line.trim_start().starts_with(&needle))
        .expect("node let statement");
    line.split_once('=')
        .expect("let with a rhs")
        .1
        .trim()
        .trim_end_matches(';')
        .to_owned()
}

fn surface_rhs(nodes: &[Node], index: usize) -> String {
    let graph = graph(nodes.to_vec());
    let validated = validate(&graph).expect("valid");
    let body = generate_surface_body(&graph, &validated);
    rhs_of(&body, index)
}

fn displacement_rhs(node: Node) -> String {
    let graph = displaced(vec![node], None);
    let validated = validate(&graph).expect("valid");
    let body = generate_displacement_body(&graph, &validated).expect("has displacement");
    rhs_of(&body, 0)
}

/// The exact RHS each surface node kind emits, at per-node granularity: a
/// wrong but well-formed builtin would otherwise pass.
#[rstest]
#[case(&[Node::Uv], "in.uv")]
#[case(&[Node::WorldNormal], "pbr_input.world_normal")]
#[case(&[Node::WorldPosition], "in.world_position.xyz")]
#[case(&[Node::VertexColor], "in.color")]
#[case(&[Node::Time], "globals.time")]
#[case(&[Node::Add { a: const_f(1.0), b: const_f(2.0) }], "(1.0 + 2.0)")]
#[case(&[Node::Sub { a: const_f(1.0), b: const_f(2.0) }], "(1.0 - 2.0)")]
#[case(&[Node::Mul { a: const_f(1.0), b: const_f(2.0) }], "(1.0 * 2.0)")]
#[case(&[Node::Div { a: const_f(1.0), b: const_f(2.0) }], "(1.0 / 2.0)")]
#[case(
    &[Node::Lerp { a: const_f(0.0), b: const_f(1.0), t: const_f(0.5) }],
    "mix(0.0, 1.0, 0.5)"
)]
#[case(
    &[Node::Dot { a: const_v3([0.0, 0.0, 1.0]), b: const_v3([0.0, 1.0, 0.0]) }],
    "dot(vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(0.0, 1.0, 0.0))"
)]
#[case(&[Node::Sin { x: const_f(1.0) }], "sin(1.0)")]
#[case(&[Node::Cos { x: const_f(1.0) }], "cos(1.0)")]
#[case(
    &[Node::Fresnel { power: const_f(2.0) }],
    "pow(clamp(1.0 - dot(N, V), 0.0, 1.0), 2.0)"
)]
#[case(
    &[Node::Noise { uv: const_v2([0.5, 0.5]) }],
    "graph_noise(vec2<f32>(0.5, 0.5))"
)]
#[case(
    &[Node::TextureSample { uv: const_v2([0.5, 0.5]), slot: 2 }],
    "textureSample(tex_2, samp_2, vec2<f32>(0.5, 0.5))"
)]
#[case(
    &[Node::Select { cond: const_f(1.0), a: const_f(2.0), b: const_f(3.0) }],
    "select(3.0, 2.0, 1.0 > 0.5)"
)]
#[case(&[Node::OneMinus { x: const_f(0.25) }], "(1.0 - 0.25)")]
#[case(&[Node::Abs { x: const_f(-1.0) }], "abs(-1.0)")]
#[case(&[Node::Floor { x: const_f(1.5) }], "floor(1.5)")]
#[case(&[Node::Fract { x: const_f(1.5) }], "fract(1.5)")]
#[case(&[Node::Saturate { x: const_f(2.0) }], "saturate(2.0)")]
#[case(&[Node::Sqrt { x: const_f(4.0) }], "sqrt(max(4.0, 0.0))")]
#[case(
    &[Node::Pow { x: const_f(2.0), y: const_f(3.0) }],
    "pow(max(2.0, 0.0), 3.0)"
)]
#[case(&[Node::Min { a: const_f(1.0), b: const_f(2.0) }], "min(1.0, 2.0)")]
#[case(&[Node::Max { a: const_f(1.0), b: const_f(2.0) }], "max(1.0, 2.0)")]
#[case(
    &[Node::Clamp { x: const_f(2.0), low: const_f(0.0), high: const_f(1.0) }],
    "clamp(2.0, 0.0, 1.0)"
)]
#[case(
    &[Node::Step { edge: const_f(0.5), x: const_f(0.75) }],
    "step(0.5, 0.75)"
)]
#[case(
    &[Node::Smoothstep { low: const_f(0.0), high: const_f(1.0), x: const_f(0.5) }],
    "smoothstep(0.0, 1.0, 0.5)"
)]
#[case(
    &[Node::Length { v: const_v3([1.0, 2.0, 3.0]) }],
    "length(vec3<f32>(1.0, 2.0, 3.0))"
)]
#[case(
    &[Node::Normalize { v: const_v3([1.0, 2.0, 3.0]) }],
    "normalize(vec3<f32>(1.0, 2.0, 3.0))"
)]
#[case(
    &[Node::Cross { a: const_v3([1.0, 0.0, 0.0]), b: const_v3([0.0, 1.0, 0.0]) }],
    "cross(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0))"
)]
#[case(
    &[Node::Extract { v: const_v3([1.0, 0.0, 0.0]), channel: 1 }],
    "(vec3<f32>(1.0, 0.0, 0.0)).y"
)]
#[case(
    &[Node::Combine2 { x: const_f(1.0), y: const_f(2.0) }],
    "vec2<f32>(1.0, 2.0)"
)]
#[case(
    &[Node::Combine3 { x: const_f(1.0), y: const_f(2.0), z: const_f(3.0) }],
    "vec3<f32>(1.0, 2.0, 3.0)"
)]
#[case(
    &[Node::Combine4 { x: const_f(1.0), y: const_f(2.0), z: const_f(3.0), w: const_f(4.0) }],
    "vec4<f32>(1.0, 2.0, 3.0, 4.0)"
)]
fn node_emits(#[case] nodes: &[Node], #[case] expected: &str) {
    assert_eq!(surface_rhs(nodes, 0), expected);
}

/// A widened color's padded alpha is 1.0, not the 0.0 every other widening
/// pads with — a zero-padded color would be invisible.
#[rstest]
#[case::widened_color_pads_alpha_opaque(
    &[Node::Convert { v: const_v3([0.0, 0.0, 0.0]), to: ValueKind::Color }],
    "vec4<f32>(vec3<f32>(0.0, 0.0, 0.0), 1.0)"
)]
#[case::widening_pads_zero(
    &[Node::Convert { v: const_v2([0.0, 0.0]), to: ValueKind::Vec3 }],
    "vec3<f32>(vec2<f32>(0.0, 0.0), 0.0)"
)]
#[case::narrowing_swizzles(
    &[Node::Convert { v: const_color([1.0, 1.0, 1.0, 1.0]), to: ValueKind::Vec3 }],
    "(vec4<f32>(1.0, 1.0, 1.0, 1.0)).xyz"
)]
fn convert_emits(#[case] nodes: &[Node], #[case] expected: &str) {
    assert_eq!(surface_rhs(nodes, 0), expected);
}

#[rstest]
#[case(Node::LocalPosition, "vertex.position")]
#[case(Node::LocalNormal, "vertex.normal")]
fn displacement_node_emits(#[case] node: Node, #[case] expected: &str) {
    assert_eq!(displacement_rhs(node), expected);
}

/// Nodes whose RHS depends on the kinds their ports reference, and the
/// `n{i}` back-references into earlier lets.
#[test]
fn node_ports_reference_earlier_lets() {
    assert_eq!(
        surface_rhs(
            &[
                Node::Uv,
                Node::Noise { uv: node(0) },
                Node::Combine3 {
                    x: node(1),
                    y: node(1),
                    z: node(1),
                },
            ],
            2
        ),
        "vec3<f32>(n1, n1, n1)"
    );
    assert_eq!(
        surface_rhs(
            &[
                Node::Uv,
                Node::Convert {
                    v:  node(0),
                    to: ValueKind::Color,
                }
            ],
            1
        ),
        "vec4<f32>(n0, 0.0, 1.0)"
    );
    assert_eq!(
        surface_rhs(
            &[
                Node::Combine2 {
                    x: const_f(1.0),
                    y: const_f(2.0),
                },
                Node::Extract {
                    v:       node(0),
                    channel: 0,
                },
            ],
            1,
        ),
        "(n0).x"
    );
    assert_eq!(
        surface_rhs(&[Node::Uv, Node::Sqrt { x: node(0) }], 1),
        "sqrt(max(n0, vec2<f32>(0.0)))"
    );
}

/// WGSL's own `*` broadcasts, so a scaled vector needs no matching vector.
#[test]
fn a_float_scales_a_vector_without_a_matching_vector() {
    assert_eq!(
        surface_rhs(
            &[
                Node::WorldNormal,
                Node::Mul {
                    a: node(0),
                    b: const_f(0.5),
                },
            ],
            1
        ),
        "(n0 * 0.5)"
    );
}
