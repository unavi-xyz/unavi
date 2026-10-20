#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

// `still::FACE_SIZE` and `still::NEAR`, checked against them in
// `tests/shader.rs`.
const FACE_SIZE: f32 = 512.0;
const NEAR: f32 = 0.05;
// Written where a pixel hit nothing, which is the sky.
const FAR: f32 = 1.0e4;

// The still camera's depth, the size of the whole atlas.
@group(0) @binding(0)
var depth_texture: texture_depth_2d;

// Converts the depth under a pixel to the distance from the face's eye. The
// camera is a 90 degree, infinite, reverse-z perspective: depth is `NEAR`
// over the view-space distance along the axis, and the ray through a pixel is
// `(ndc, 1)` at unit axis distance.
@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let depth = textureLoad(depth_texture, vec2<i32>(in.position.xy), 0);
    let local = in.position.xy - floor(in.position.xy / FACE_SIZE) * FACE_SIZE;
    let ndc = local / FACE_SIZE * 2.0 - 1.0;
    let ray = length(vec3<f32>(ndc, 1.0));
    let distance = select(FAR, NEAR / max(depth, 1.0e-9) * ray, depth > 0.0);
    return vec4<f32>(min(distance, FAR), 0.0, 0.0, 1.0);
}
