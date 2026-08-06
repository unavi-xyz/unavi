mod common;

use common::{
    const_color,
    const_f,
    const_v2,
    graph_with_output,
    node,
    unlit,
};
use hsd::attributes::material_graph::{
    graph::{
        LitOutput,
        SurfaceOutput,
        UnlitOutput,
    },
    node::Node,
    validate::{
        error::GraphError,
        validate,
    },
    value::ValueKind,
};

#[test]
fn terminal_referencing_an_out_of_bounds_node_is_rejected() {
    let graph = graph_with_output(Vec::new(), unlit(node(0)));
    assert_eq!(
        validate(&graph),
        Err(GraphError::UnknownTerminalNode("color", 0))
    );
}

#[test]
fn terminal_type_mismatch_is_rejected() {
    let graph = graph_with_output(vec![Node::Uv], unlit(node(0)));
    assert_eq!(
        validate(&graph),
        Err(GraphError::TerminalTypeMismatch {
            name:     "color",
            expected: ValueKind::Color,
            found:    ValueKind::Vec2,
        })
    );
}

#[test]
fn alpha_clip_threshold_must_be_float() {
    let graph = graph_with_output(
        Vec::new(),
        SurfaceOutput::Unlit(UnlitOutput {
            color:                const_color([1.0, 1.0, 1.0, 1.0]),
            alpha_clip_threshold: Some(const_v2([0.0, 0.0])),
        }),
    );
    assert_eq!(
        validate(&graph),
        Err(GraphError::TerminalTypeMismatch {
            name:     "alpha_clip_threshold",
            expected: ValueKind::Float,
            found:    ValueKind::Vec2,
        })
    );
}

#[test]
fn lit_output_requires_pbr_typed_terminals() {
    let graph = graph_with_output(
        Vec::new(),
        SurfaceOutput::Lit(LitOutput {
            base_color: Some(const_color([1.0, 1.0, 1.0, 1.0])),
            metallic: Some(const_f(0.5)),
            ..Default::default()
        }),
    );
    assert!(validate(&graph).is_ok());
}

/// A texture channel driving a `Float` terminal — the mask/ORM-texture
/// workflow, which `Extract` is what makes expressible.
#[test]
fn a_texture_channel_can_drive_a_float_terminal() {
    let graph = graph_with_output(
        vec![
            Node::Uv,
            Node::TextureSample {
                uv:   node(0),
                slot: 0,
            },
            Node::Extract {
                v:       node(1),
                channel: 1,
            },
        ],
        SurfaceOutput::Lit(LitOutput {
            roughness: Some(node(2)),
            ..Default::default()
        }),
    );
    assert!(validate(&graph).is_ok());
}
