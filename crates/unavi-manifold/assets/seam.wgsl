#import bevy_pbr::{
    forward_io::{Vertex, VertexOutput},
    mesh_view_bindings::view,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1)
var texture_sampler: sampler;

struct PortalParams {
    world_from_seam: mat4x4<f32>,
    half_size:       vec2<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(2)
var<uniform> params: PortalParams;

struct FragmentOutput {
    @builtin(frag_depth) depth: f32,
    @location(0) color:        vec4<f32>,
}

// The portal is drawn as a full-screen pass and composited per pixel by
// intersecting the view ray with the seam plane. Nothing is rasterized in the
// seam plane, so there is no surface for the near plane or w-clipping to slice;
// the result is correct from any angle, including with the eye in the plane.
@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(sign(vertex.position.x), sign(vertex.position.y), 1.0, 1.0);
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
    let center = params.world_from_seam[3].xyz;
    let normal = params.world_from_seam[2].xyz;
    let x_axis = params.world_from_seam[0].xyz;
    let y_axis = params.world_from_seam[1].xyz;

    let uv = (in.position.xy - view.viewport.xy) / view.viewport.zw;
    let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
    // Reverse-z near plane is `z = 1`; unproject it for a finite ray point. The
    // far plane (`z = 0`) is at infinity for Bevy's infinite projection and
    // unprojects to `w = 0`, which would make the ray direction NaN.
    let near_point = view.world_from_clip * vec4<f32>(ndc, 1.0, 1.0);
    let ray_origin = view.world_position;
    let ray_dir = normalize(near_point.xyz / near_point.w - ray_origin);

    let denom = dot(ray_dir, normal);
    if abs(denom) < 1.0e-6 {
        discard;
    }
    let t = dot(center - ray_origin, normal) / denom;
    if t <= 0.0 {
        discard;
    }

    let hit = ray_origin + ray_dir * t;
    let offset = hit - center;
    if abs(dot(offset, x_axis)) > params.half_size.x
        || abs(dot(offset, y_axis)) > params.half_size.y {
        discard;
    }

    let clip = view.clip_from_world * vec4<f32>(hit, 1.0);
    let dims = vec2<f32>(textureDimensions(texture));
    let screen_uv = (in.position.xy - view.viewport.xy) / dims;

    var out: FragmentOutput;
    out.depth = clip.z / clip.w;
    out.color = vec4<f32>(textureSample(texture, texture_sampler, screen_uv).rgb, 1.0);
    return out;
}
