#import bevy_pbr::forward_io::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1)
var texture_sampler: sampler;

struct PortalParams {
    time: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(2)
var<uniform> params: PortalParams;

const EDGE_WIDTH: f32        = 0.08;
const CREEP_SPEED: f32       = 0.05;
const CREEP_AMOUNT: f32      = 0.02;
const CHROMA_STRENGTH: f32   = 0.002;
const COMPRESS_STRENGTH: f32 = 0.02;

fn hue_shift(c: vec3<f32>, a: f32) -> vec3<f32> {
    let k = vec3<f32>(0.57735, 0.57735, 0.57735);
    let cos_a = cos(a);
    return c * cos_a + cross(k, c) * sin(a) + k * dot(k, c) * (1.0 - cos_a);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let center = vec2<f32>(0.5, 0.5);

    let dims = textureDimensions(texture);

    let d = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));

    let t = params.time;

    let edge_width = EDGE_WIDTH + fract(t * CREEP_SPEED) * CREEP_AMOUNT;

    let influence = 1.0 - smoothstep(edge_width * 0.4, edge_width * 3.2, d);

    var screen_uv = in.position.xy / vec2<f32>(f32(dims.x), f32(dims.y));
    screen_uv = mix(screen_uv, center, influence * COMPRESS_STRENGTH);

    let delta = uv - center;
    let len = max(length(delta), 1e-5);
    let dir = delta / len;

    let hue = sin(t * 0.6 + uv.y * 2.0 + uv.x * 1.5) * 0.5;

    let c = CHROMA_STRENGTH * influence;

    let r = textureSample(texture, texture_sampler, screen_uv + dir * c).r;
    let g = textureSample(texture, texture_sampler, screen_uv).g;
    let b = textureSample(texture, texture_sampler, screen_uv - dir * c).b;

    var color = vec3<f32>(r, g, b);

    color = hue_shift(color, hue * influence * 0.7);

    return vec4<f32>(color, 1.0);
}
