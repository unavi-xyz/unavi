mod common;

use common::{
    const_v3,
    displaced,
    graph,
    node,
};
use hsd::attributes::material_graph::{
    node::{
        Network,
        Node,
    },
    validate::{
        error::GraphError,
        validate,
    },
    value::ValueKind,
};

#[test]
fn surface_only_leaves_are_rejected_in_displacement() {
    let graph = displaced(vec![Node::WorldNormal], Some(node(0)));
    assert_eq!(
        validate(&graph),
        Err(GraphError::WrongNetwork {
            network: Network::Displacement,
            node:    0,
        })
    );
}

#[test]
fn displacement_only_leaves_are_rejected_in_surface() {
    let graph = graph(vec![Node::LocalPosition]);
    assert_eq!(
        validate(&graph),
        Err(GraphError::WrongNetwork {
            network: Network::Surface,
            node:    0,
        })
    );
}

#[test]
fn a_displacement_graph_computes_a_position_offset() {
    let graph = displaced(
        vec![
            Node::LocalNormal,
            Node::Time,
            Node::Mul {
                a: node(0),
                b: const_v3([1.0, 1.0, 1.0]),
            },
        ],
        Some(node(2)),
    );
    let validated = validate(&graph).expect("valid");
    assert_eq!(
        validated.displacement(),
        Some([ValueKind::Vec3, ValueKind::Float, ValueKind::Vec3].as_slice())
    );
}

/// `Uv` is surface-only, so this exercises the wrong-network check first;
/// a texture sample in displacement would be caught by the same check.
#[test]
fn texture_sampling_is_rejected_in_displacement() {
    let graph = displaced(vec![Node::Uv], None);
    assert!(matches!(
        validate(&graph),
        Err(GraphError::WrongNetwork {
            network: Network::Displacement,
            ..
        })
    ));
}
