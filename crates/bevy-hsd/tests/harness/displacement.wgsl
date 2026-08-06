struct Params { inputs: array<vec4<f32>, 16> }
struct Globals { time: f32 }

@group(0) @binding(0) var<uniform> params: Params;
@group(1) @binding(0) var<uniform> globals: Globals;

fn graph_hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453123);
}

fn graph_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let a = graph_hash(i);
    let b = graph_hash(i + vec2<f32>(1.0, 0.0));
    let c = graph_hash(i + vec2<f32>(0.0, 1.0));
    let d = graph_hash(i + vec2<f32>(1.0, 1.0));
    let u = f * f * (3.0 - 2.0 * f);
    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

@fragment
fn fragment() -> @location(0) vec4<f32> {
    var vertex: vec3<f32>;
//#BODY
    return vec4<f32>(out_position_offset, 1.0) + vec4<f32>(out_normal_override, 1.0);
}
