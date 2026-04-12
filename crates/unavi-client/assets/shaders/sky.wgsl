#import bevy_pbr::forward_io::VertexOutput

struct SkyParams {
    top_color: vec4<f32>,
    bottom_color: vec4<f32>,
    horizon_softness: f32,
    radial_falloff: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> params: SkyParams;

fn hash(p: vec3<f32>) -> f32 {
    return fract(sin(dot(p, vec3<f32>(127.1, 311.7, 74.7))) * 43758.5453);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let dir = normalize(in.world_position.xyz);

    let y = clamp(dir.y * 0.5 + 0.5, 0.0, 1.0);

    let zenith = params.top_color.xyz;
    let horizon = params.bottom_color.xyz;
    let mid = mix(horizon, zenith, 0.5);

    var color = mix(
        mix(horizon, mid, smoothstep(0.0, 0.6, y)),
        mix(mid, zenith, smoothstep(0.4, 1.0, y)),
        y
    );

    let bias = dot(dir, normalize(vec3<f32>(0.2, 1.0, 0.1)));
    color *= 0.92 + 0.08 * bias;

    let depth = exp(-length(in.world_position.xyz) * 0.01);
    color *= mix(0.75, 1.0, depth);

    let vignette = 1.0 - params.radial_falloff * (1.0 - y * y);
    color *= vignette;

    let star_uv = dir * 120.0;

    let cell = floor(star_uv);
    let rand = hash(cell);

    let star = step(0.994, rand);

    let local = fract(star_uv) - 0.5;
    let dist2 = dot(local, local);

    let core = exp(-dist2 * 140.0);

    let sky_gate = smoothstep(0.2, 1.0, dir.y);

    let stars = star * core * sky_gate;

    color += vec3<f32>(stars * 2.0);

    return vec4<f32>(color, 1.0);
}
