#import bevy_pbr::{
    forward_io::{Vertex, VertexOutput},
    mesh_functions,
    mesh_view_bindings::globals,
    view_transformations::position_world_to_clip,
}

//#PREAMBLE
@vertex
fn vertex(vertex_in: Vertex) -> VertexOutput {
    var vertex = vertex_in;
    var out: VertexOutput;
    let graph_instance_index = vertex_in.instance_index;
    let world_from_local = mesh_functions::get_world_from_local(graph_instance_index);

//#BODY
#ifdef VERTEX_NORMALS
    vertex.normal = out_normal_override;
    out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal, vertex_in.instance_index);
#endif
#ifdef VERTEX_POSITIONS
    vertex.position += out_position_offset;
    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    out.world_position = vec4<f32>(out.world_position.xyz + out_world_position_offset, out.world_position.w);
    out.position = position_world_to_clip(out.world_position.xyz);
#endif
#ifdef VERTEX_UVS_A
    out.uv = vertex.uv;
#endif
#ifdef VERTEX_UVS_B
    out.uv_b = vertex.uv_b;
#endif
#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(world_from_local, vertex.tangent, vertex_in.instance_index);
#endif
#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif
// The mesh pipeline sets this def unconditionally, so `VertexOutput` always
// declares the field and a fragment reading `InstanceRandom` gets whatever
// was left in it if the vertex stage does not write it.
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    out.instance_index = graph_instance_index;
#endif

    return out;
}
