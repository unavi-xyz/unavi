#import bevy_pbr::{
    forward_io::VertexOutput,
    mesh_view_bindings::view,
}

struct SkyParams {
    zenith: vec4<f32>,
    horizon: vec4<f32>,
    haze: vec4<f32>,
    line: vec4<f32>,
    glow: vec4<f32>,
    drift: vec2<f32>,
    graticule: f32,
    spin: f32,
    pulse: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> params: SkyParams;

const PI: f32 = 3.14159265;
const TAU: f32 = 6.28318531;
// Matches the drift wrap on the CPU. Each fbm octave doubles exactly, so the
// whole sum repeats with the lattice.
const CLOUD_PERIOD: i32 = 256;

fn pcg(v: u32) -> u32 {
    let s = v * 747796405u + 2891336453u;
    let w = ((s >> ((s >> 28u) + 4u)) ^ s) * 277803737u;
    return (w >> 22u) ^ w;
}

fn hash(v: u32) -> f32 {
    return f32(pcg(v)) / 4294967295.0;
}

fn cell_hash(cell: vec2<i32>) -> f32 {
    let c = ((cell % CLOUD_PERIOD) + CLOUD_PERIOD) % CLOUD_PERIOD;
    return hash(pcg(u32(c.x)) ^ (u32(c.y) * 2654435761u));
}

fn value_noise(p: vec2<f32>) -> f32 {
    let i = vec2<i32>(floor(p));
    var f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    let a = cell_hash(i);
    let b = cell_hash(i + vec2(1, 0));
    let c = cell_hash(i + vec2(0, 1));
    let d = cell_hash(i + vec2(1, 1));
    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

fn fbm(p_in: vec2<f32>) -> f32 {
    var p = p_in;
    var sum = 0.0;
    var amp = 0.5;
    for (var i = 0; i < 5; i++) {
        sum += amp * value_noise(p);
        p = p * 2.0 + 17.0;
        amp *= 0.5;
    }
    return sum;
}

// Coverage of a line at every integer of `x`, where `w` is `fwidth(x)`.
fn grid_line(x: f32, w: f32, width: f32) -> f32 {
    let d = abs(fract(x + 0.5) - 0.5);
    return 1.0 - smoothstep(w * width * 0.5, w * (width * 0.5 + 1.0), d);
}

fn overcast(d: vec3<f32>) -> vec3<f32> {
    let up = clamp(d.y, 0.0, 1.0);
    var c = mix(params.horizon.rgb, params.zenith.rgb, sqrt(up));
    c *= 1.0 - 0.3 * smoothstep(0.0, 0.4, -d.y);
    c += params.haze.rgb * exp(-abs(d.y) * 16.0);

    let q = d.xz / (up + 0.15);
    let n = fbm(q * 0.9 + params.drift);
    let deck = smoothstep(0.42, 0.78, n) * smoothstep(0.0, 0.35, d.y);
    return mix(c, c * 1.1 + params.haze.rgb * 0.2, deck);
}

fn graticule(d: vec3<f32>) -> vec3<f32> {
    let el = asin(clamp(d.y, -1.0, 1.0));
    // Nudged off the pole, where atan2(0, 0) is undefined.
    let flat = vec2(d.x + 1e-7, d.z);
    let lon = atan2(flat.y, flat.x) / TAU * 48.0 + params.spin;
    // A second longitude with its seam on the far side, so the line width
    // does not blow up where atan2 wraps.
    let lon_far = (atan2(-flat.y, -flat.x) + PI) / TAU * 48.0 + params.spin;
    let lon_w = min(fwidth(lon), fwidth(lon_far));
    let lat = el / (PI * 0.5) * 9.0;
    let lat_w = fwidth(lat);

    let above = step(0.0, d.y);
    let meridian = grid_line(lon, lon_w, 1.0) * (1.0 - smoothstep(0.35, 0.82, d.y)) * above;
    let parallel = grid_line(lat, lat_w, 1.0) * above;
    let fade = exp(-max(d.y, 0.0) * 2.4);
    var c = params.line.rgb * max(meridian, parallel) * 0.32 * fade;

    let hz = abs(d.y);
    c += params.glow.rgb * (exp(-hz / 0.0035) * 1.4 + exp(-hz / 0.05) * 0.12);

    // A few meridians carry a pulse climbing toward the zenith.
    // Wrapped to 48 so the meridian on the atan2 seam has one key.
    let k = u32(((i32(floor(lon + 0.5)) % 48) + 48) % 48);
    let multiple = 1.0 + floor(hash(k + 5000u) * 3.0);
    let phase = fract(hash(k + 1000u) + params.pulse * multiple);
    let wide = grid_line(lon, lon_w, 2.5) * (1.0 - smoothstep(0.5, 0.9, d.y)) * above;
    let along = (el / (PI * 0.5) - phase) * 38.0;
    let pulse = exp(-along * along) * step(0.6, hash(k + 77u));
    c += params.glow.rgb * pulse * wide * 2.2;
    return c;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let d = normalize(in.world_position.xyz - view.world_position);
    let c = overcast(d) + graticule(d) * params.graticule;
    return vec4(c, 1.0);
}
