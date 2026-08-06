mod common;

use common::{
    const_f,
    displaced,
    graph,
    graph_with_output,
    input,
    node,
    unlit,
};
use hsd::attributes::material_graph::{
    MAX_NODES,
    MAX_PUBLIC_INPUTS,
    MAX_TEXTURE_SAMPLES,
    ShaderGraph,
    graph::{
        LitOutput,
        SurfaceGraph,
        SurfaceOutput,
    },
    node::{
        Network,
        Node,
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

#[test]
fn default_graph_is_valid_and_unlit() {
    let validated = validate(&ShaderGraph::default()).expect("valid");
    assert_eq!(validated.surface(), Vec::new());
    assert_eq!(validated.displacement(), None);
}

/// The DAG-safety property: a node's inputs may only reference strictly
/// lower indices, so a cycle cannot be constructed in the first place. A
/// self-reference is just the tightest forward reference.
#[test]
fn a_node_may_not_reference_itself_or_a_later_node() {
    let self_ref = graph(vec![Node::Add {
        a: node(0),
        b: const_f(1.0),
    }]);
    assert_eq!(
        validate(&self_ref),
        Err(GraphError::ForwardReference {
            network: Network::Surface,
            node:    0,
            target:  0,
        })
    );

    let forward_ref = graph(vec![
        Node::Add {
            a: node(1),
            b: const_f(1.0),
        },
        Node::Time,
    ]);
    assert_eq!(
        validate(&forward_ref),
        Err(GraphError::ForwardReference {
            network: Network::Surface,
            node:    0,
            target:  1,
        })
    );
}

#[test]
fn a_node_may_reference_an_earlier_node_in_the_same_network() {
    let graph = graph_with_output(
        vec![
            Node::Time,
            Node::Add {
                a: const_f(1.0),
                b: node(0),
            },
        ],
        SurfaceOutput::Lit(LitOutput {
            roughness: Some(node(1)),
            ..Default::default()
        }),
    );
    let validated = validate(&graph).expect("valid");
    assert_eq!(
        validated.surface(),
        vec![ValueKind::Float, ValueKind::Float]
    );
}

#[test]
fn sin_and_cos_compose_a_time_driven_oscillator() {
    let graph = displaced(
        vec![
            Node::Time,
            Node::Sin { x: node(0) },
            Node::Cos { x: node(0) },
            Node::LocalNormal,
            Node::Mul {
                a: node(3),
                b: const_f(0.1),
            },
        ],
        Some(node(4)),
    );
    let validated = validate(&graph).expect("valid");
    assert_eq!(
        validated.displacement(),
        Some(
            [
                ValueKind::Float,
                ValueKind::Float,
                ValueKind::Float,
                ValueKind::Vec3,
                ValueKind::Vec3,
            ]
            .as_slice()
        )
    );
}

#[test]
fn a_node_may_reference_a_public_input() {
    let graph = ShaderGraph {
        public_inputs: vec![GraphValue::Color([1.0, 0.0, 0.0, 1.0])],
        surface:       SurfaceGraph {
            nodes:  Vec::new(),
            output: unlit(input(0)),
        },
        displacement:  None,
    };
    assert!(validate(&graph).is_ok());
}

#[test]
fn node_count_cap_is_enforced_per_network() {
    let graph = graph(vec![Node::Time; MAX_NODES + 1]);
    assert_eq!(
        validate(&graph),
        Err(GraphError::TooManyNodes {
            network: Network::Surface,
            count:   MAX_NODES + 1,
        })
    );
}

#[test]
fn texture_sample_cap_is_enforced() {
    let mut nodes = vec![Node::Uv];
    nodes.extend(std::iter::repeat_n(
        Node::TextureSample {
            uv:   node(0),
            slot: 0,
        },
        MAX_TEXTURE_SAMPLES + 1,
    ));
    let graph = graph(nodes);
    assert_eq!(
        validate(&graph),
        Err(GraphError::TooManyTextureSamples(MAX_TEXTURE_SAMPLES + 1))
    );
}

#[test]
fn texture_slot_out_of_range_is_rejected() {
    let graph = graph(vec![
        Node::Uv,
        Node::TextureSample {
            uv:   node(0),
            slot: MAX_TEXTURE_SAMPLES as u8,
        },
    ]);
    assert_eq!(
        validate(&graph),
        Err(GraphError::InvalidTextureSlot(MAX_TEXTURE_SAMPLES as u8))
    );
}

#[test]
fn public_input_cap_is_enforced() {
    let graph = ShaderGraph {
        public_inputs: vec![GraphValue::Float(0.0); MAX_PUBLIC_INPUTS + 1],
        ..Default::default()
    };
    assert_eq!(
        validate(&graph),
        Err(GraphError::TooManyPublicInputs(MAX_PUBLIC_INPUTS + 1))
    );
}
