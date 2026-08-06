use hsd::attributes::material_graph::node::Node;

/// The zero-arity leaves: each reads shader-stage context directly, and each
/// is legal in exactly one network (enforced by validation, not here).
pub(super) fn emit(out: &mut String, node: &Node) {
    match *node {
        Node::Uv => out.push_str("in.uv"),
        Node::WorldNormal => out.push_str("pbr_input.world_normal"),
        Node::WorldPosition => out.push_str("in.world_position.xyz"),
        Node::VertexColor => out.push_str("in.color"),
        Node::LocalPosition => out.push_str("vertex.position"),
        Node::LocalNormal => out.push_str("vertex.normal"),
        // Bevy's own view-wide `globals` uniform, not a slot in this
        // material's bind group: a per-material `time` would have to be
        // re-uploaded every frame to say what the view already knows.
        Node::Time => out.push_str("globals.time"),
        _ => unreachable!("only the dispatch match in expr/mod.rs reaches here"),
    }
}
