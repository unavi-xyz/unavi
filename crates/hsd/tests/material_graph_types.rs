mod common;

use common::{
    const_f,
    graph,
    input,
    node,
    unlit,
};
use hsd::attributes::material_graph::{
    ShaderGraph,
    graph::{
        SurfaceGraph,
        SurfaceOutput,
        UnlitOutput,
    },
    node::{
        Network,
        Node,
        Port,
    },
    validate::{
        error::GraphError,
        validate,
    },
    value::{
        GraphValue,
        ValueKind,
    },
};
use rstest::rstest;

/// One flat table for the whole rejection vocabulary, instead of ~20
/// near-identical single-case functions: each pair is a graph and the exact
/// error `validate` must return for it.
#[rstest]
#[case::forward_ref(
    graph(vec![Node::Add { a: node(1), b: const_f(1.0) }, Node::Time]),
    GraphError::ForwardReference { network: Network::Surface, node: 0, target: 1 })]
#[case::extract_from_scalar(
    graph(vec![Node::Time, Node::Extract { v: node(0), channel: 0 }]),
    GraphError::NotAVector { network: Network::Surface, node: 1, port: "v", found: ValueKind::Float })]
#[case::mismatched_port_kind(
    graph(vec![Node::Uv, Node::WorldNormal, Node::Add { a: node(0), b: node(1) }]),
    GraphError::NodeTypeMismatch {
        network: Network::Surface, node: 2, port: "b", expected: ValueKind::Vec2,
        found: ValueKind::Vec3,
    })]
#[case::builtin_backed_no_broadcast(
    graph(vec![Node::Uv, Node::Min { a: node(0), b: const_f(0.5) }]),
    GraphError::NodeTypeMismatch {
        network: Network::Surface, node: 1, port: "b", expected: ValueKind::Vec2,
        found: ValueKind::Float,
    })]
#[case::channel_out_of_range(
    graph(vec![Node::Uv, Node::Extract { v: node(0), channel: 2 }]),
    GraphError::ChannelOutOfRange {
        network: Network::Surface, node: 1, channel: 2, kind: ValueKind::Vec2,
    })]
#[case::convert_scalar(
    graph(vec![Node::WorldNormal, Node::Convert { v: node(0), to: ValueKind::Float }]),
    GraphError::InvalidConversion {
        network: Network::Surface, node: 1, from: ValueKind::Vec3, to: ValueKind::Float,
    })]
#[case::sin_requires_float(
    graph(vec![Node::Uv, Node::Sin { x: node(0) }]),
    GraphError::NodeTypeMismatch {
        network: Network::Surface, node: 1, port: "x", expected: ValueKind::Float,
        found: ValueKind::Vec2,
    })]
#[case::unknown_public_input(
    graph(vec![Node::Add { a: input(0), b: const_f(1.0) }]),
    GraphError::UnknownInput { network: Network::Surface, node: 0, index: 0 })]
fn rejects(#[case] graph: ShaderGraph, #[case] expected: GraphError) {
    assert_eq!(validate(&graph), Err(expected));
}

/// Arithmetic has its own rule — broadcast a `Float` across a vector, which
/// is what WGSL's `+ - * /` already do — so scaling a vector needs no
/// same-width vector built to match it.
#[test]
fn arithmetic_broadcasts_a_float_across_a_vector() {
    for (index, node) in [
        Node::Add {
            a: node(0),
            b: const_f(0.5),
        },
        Node::Mul {
            a: const_f(0.5),
            b: node(0),
        },
        Node::Sub {
            a: node(0),
            b: const_f(0.5),
        },
        Node::Div {
            a: node(0),
            b: const_f(0.5),
        },
    ]
    .into_iter()
    .enumerate()
    {
        let graph = graph(vec![Node::WorldNormal, node]);
        let validated = validate(&graph).unwrap_or_else(|err| panic!("node {index}: {err}"));
        assert_eq!(validated.surface()[1], ValueKind::Vec3, "node {index}");
    }
}

#[test]
fn builtin_backed_nodes_do_not_broadcast() {
    let graph = graph(vec![
        Node::WorldNormal,
        Node::Min {
            a: node(0),
            b: const_f(0.5),
        },
    ]);
    assert!(
        matches!(
            validate(&graph),
            Err(GraphError::NodeTypeMismatch { node: 1, .. })
        ),
        "builtin operand kinds must match, not broadcast"
    );
}

#[test]
fn combine_and_extract_round_trip_through_a_scalar() {
    let graph = graph(vec![
        Node::Uv,
        Node::Noise { uv: node(0) },
        Node::Combine3 {
            x: node(1),
            y: node(1),
            z: node(1),
        },
        Node::Extract {
            v:       node(2),
            channel: 2,
        },
    ]);
    let validated = validate(&graph).expect("valid");
    assert_eq!(validated.surface()[2], ValueKind::Vec3);
    assert_eq!(validated.surface()[3], ValueKind::Float);
}

#[test]
fn length_returns_a_scalar_and_normalize_keeps_the_vector_kind() {
    let graph = graph(vec![
        Node::WorldNormal,
        Node::Length { v: node(0) },
        Node::Normalize { v: node(0) },
    ]);
    let validated = validate(&graph).expect("valid");
    assert_eq!(validated.surface()[1], ValueKind::Float);
    assert_eq!(validated.surface()[2], ValueKind::Vec3);
}

#[test]
fn convert_bridges_a_vec3_node_to_a_color_terminal() {
    let graph = graph(vec![
        Node::WorldNormal,
        Node::Convert {
            v:  node(0),
            to: ValueKind::Color,
        },
    ]);
    assert_eq!(
        validate(&graph).expect("valid").surface()[1],
        ValueKind::Color
    );
}

/// `f32`'s own formatting renders these `NaN`/`inf`, neither of which is a
/// WGSL literal, so a graph carrying one would compile to a shader that does
/// not parse. Every route a float can enter by is checked.
#[test]
fn non_finite_floats_are_rejected() {
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let in_a_node = ShaderGraph {
            public_inputs: Vec::new(),
            surface:       SurfaceGraph {
                nodes: vec![Node::Sin { x: const_f(bad) }],
                output: SurfaceOutput::Unlit(UnlitOutput::default()),
                ..Default::default()
            },
            displacement:  None,
        };
        assert_eq!(
            validate(&in_a_node),
            Err(GraphError::NonFiniteConst {
                network: Network::Surface,
                node:    0,
            }),
            "node constant {bad}"
        );

        let in_a_public_input = ShaderGraph {
            public_inputs: vec![GraphValue::Vec3([0.0, bad, 0.0])],
            ..Default::default()
        };
        assert_eq!(
            validate(&in_a_public_input),
            Err(GraphError::NonFinitePublicInput(0)),
            "public input {bad}"
        );

        let in_a_terminal = ShaderGraph {
            public_inputs: Vec::new(),
            surface:       SurfaceGraph {
                nodes: Vec::new(),
                output: unlit(Port::Const(GraphValue::Color([bad, 0.0, 0.0, 1.0]))),
                ..Default::default()
            },
            displacement:  None,
        };
        assert_eq!(
            validate(&in_a_terminal),
            Err(GraphError::NonFiniteTerminal("color")),
            "terminal constant {bad}"
        );
    }
}
