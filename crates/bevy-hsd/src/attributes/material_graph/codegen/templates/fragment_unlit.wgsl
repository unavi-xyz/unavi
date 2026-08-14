#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    mesh_functions,
    mesh_view_bindings::{view, globals},
    pbr_functions::calculate_view,
}

//#PREAMBLE
@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
    let is_orthographic = view.clip_from_view[3].w == 1.0;
    let N = normalize(in.world_normal);
    let V = calculate_view(in.world_position, is_orthographic);
    let graph_world_normal = in.world_normal;
    let graph_instance_index = in.instance_index;
    let world_from_local = mesh_functions::get_world_from_local(graph_instance_index);

//#BODY
    var out: FragmentOutput;
    out.color = out_color;
    return out;
}
