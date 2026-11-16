// A view prepares the transmission texture only when a transmissive surface is
// in it, so it reads black where none is bound rather than failing to compile.
fn graph_scene_color(uv: vec2<f32>) -> vec4<f32> {
@if(VIEW_TRANSMISSION_TEXTURE)
    return textureSampleLevel(view_transmission_texture, view_transmission_sampler, uv, 0.0);
@if(!VIEW_TRANSMISSION_TEXTURE)
    return vec4<f32>(0.0);
}
