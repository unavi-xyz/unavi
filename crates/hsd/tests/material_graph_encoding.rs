mod common;

use common::{
    const_color,
    graph_with_output,
    input,
    node,
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
    value::GraphValue,
};

/// Cross-prim dedup depends on this: two structurally identical graphs
/// must compile to byte-identical slot entries so their content hashes
/// collide in the blob store.
#[test]
fn identical_graphs_encode_identically() {
    let make = || {
        graph_with_output(
            vec![Node::Uv, Node::Time],
            SurfaceOutput::Unlit(UnlitOutput {
                color:                const_color([1.0, 1.0, 1.0, 1.0]),
                alpha_clip_threshold: Some(node(1)),
            }),
        )
    };
    assert_eq!(
        make().encode().expect("encode"),
        make().encode().expect("encode")
    );
}

#[test]
fn encode_decode_round_trips() {
    let graph = ShaderGraph {
        public_inputs: vec![GraphValue::Float(0.5)],
        surface:       SurfaceGraph {
            nodes: vec![
                Node::Uv,
                Node::TextureSample {
                    uv:   node(0),
                    slot: 2,
                },
            ],
            output: SurfaceOutput::Lit(LitOutput {
                base_color: Some(node(1)),
                alpha: Some(input(0)),
                ..Default::default()
            }),
            ..Default::default()
        },
        displacement:  Some(DisplacementGraph {
            nodes:                 vec![Node::LocalPosition],
            position_offset:       Some(node(0)),
            normal_override:       None,
            world_position_offset: None,
        }),
    };
    let bytes = graph.encode().expect("encode");
    let decoded = ShaderGraph::decode(&bytes).expect("decode");
    assert_eq!(decoded.encode().expect("re-encode"), bytes);
}
