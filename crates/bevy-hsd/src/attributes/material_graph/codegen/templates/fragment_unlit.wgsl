#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    mesh_functions,
    mesh_view_bindings::{view, globals, view_transmission_texture, view_transmission_sampler},
    pbr_functions::calculate_view,
}

//#PREAMBLE
@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
    let is_orthographic = view.clip_from_view[3].w == 1.0;
    let N = normalize(in.world_normal);
    let V = calculate_view(in.world_position, is_orthographic);
    let graph_world_normal = in.world_normal;
    // `frag_coord_to_uv`, inlined rather than imported: it is two operations,
    // and the import path is one more thing to keep in step with bevy_render.
    let graph_screen_uv = (in.position.xy - view.viewport.xy) / view.viewport.zw;
    let graph_instance_index = in.instance_index;
    let world_from_local = mesh_functions::get_world_from_local(graph_instance_index);

//#BODY
    var out: FragmentOutput;
    out.color = out_color;
    return out;
}
