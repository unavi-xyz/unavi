use hsd::attributes::material_graph::node::Node;

/// The zero-arity leaves: each reads shader-stage context directly, and each
/// is legal in exactly one network (enforced by validation, not here).
pub(super) fn emit(out: &mut String, node: &Node) {
    match *node {
        Node::Uv => out.push_str("in.uv"),
        // Bound by both fragment templates, not read off `pbr_input`: that
        // only exists on the lit path, so reaching for it made every unlit
        // graph using a normal fail to compile.
        Node::WorldNormal => out.push_str("graph_world_normal"),
        Node::WorldPosition => out.push_str("in.world_position.xyz"),
        Node::VertexColor => out.push_str("in.color"),
        Node::LocalPosition => out.push_str("vertex.position"),
        Node::LocalNormal => out.push_str("vertex.normal"),
        // Bevy's view-wide `globals` uniform, not a slot in this material's
        // bind group: a per-material `time` would need re-uploading every
        // frame.
        Node::Time => out.push_str("globals.time"),
        // `graph_instance_index` and `world_from_local` are bound by both
        // templates before the body, so these read the same in either stage.
        Node::InstanceRandom => out.push_str("graph_instance_random(graph_instance_index)"),
        Node::ObjectPosition => out.push_str("world_from_local[3].xyz"),
        Node::ObjectScale => out.push_str("graph_object_scale(world_from_local)"),
        Node::ViewDirection => out.push('V'),
        Node::ScreenUv => out.push_str("graph_screen_uv"),
        _ => unreachable!("only the dispatch match in expr/mod.rs reaches here"),
    }
}
