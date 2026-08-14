#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    mesh_functions,
    mesh_view_bindings::{view, globals, view_transmission_texture, view_transmission_sampler},
    pbr_fragment::pbr_input_from_vertex_output,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing, alpha_discard},
}

//#PREAMBLE
@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_vertex_output(in, is_front, false);
    let N = pbr_input.N;
    let V = pbr_input.V;
    let graph_world_normal = pbr_input.world_normal;
    let graph_screen_uv = (in.position.xy - view.viewport.xy) / view.viewport.zw;
    let graph_instance_index = in.instance_index;
    let world_from_local = mesh_functions::get_world_from_local(graph_instance_index);

//#BODY
    pbr_input.material.base_color = vec4<f32>(out_base_color.rgb, out_alpha);
    pbr_input.material.emissive = vec4<f32>(out_emissive, 1.0);
    pbr_input.material.metallic = out_metallic;
    pbr_input.material.perceptual_roughness = out_roughness;
    pbr_input.material.specular_transmission = out_specular_transmission;
    pbr_input.material.diffuse_transmission = out_diffuse_transmission;
    pbr_input.material.thickness = out_thickness;
    pbr_input.material.ior = out_ior;
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);
    pbr_input.world_normal = out_normal;
    pbr_input.N = normalize(out_normal);

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
