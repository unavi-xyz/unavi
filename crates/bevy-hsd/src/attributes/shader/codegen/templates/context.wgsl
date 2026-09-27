fn graph_object_scale(m: mat4x4<f32>) -> vec3<f32> {
    return vec3<f32>(length(m[0].xyz), length(m[1].xyz), length(m[2].xyz));
}

// Bevy assigns instance indices while batching, so this is stable only for as
// long as the batch is. A prim whose entity outlives the effect keeps its
// value; one that is despawned and respawned may not.
fn graph_instance_random(index: u32) -> f32 {
    return fract(sin(f32(index) * 12.9898 + 78.233) * 43758.5453123);
}

// x is the radius, y the angle normalised to 0..1 counterclockwise from +x.
fn graph_polar(uv: vec2<f32>, center: vec2<f32>) -> vec2<f32> {
    let d = uv - center;
    return vec2<f32>(length(d), fract(atan2(d.y, d.x) * 0.15915494 + 1.0));
}

fn graph_rotate_uv(uv: vec2<f32>, center: vec2<f32>, radians: f32) -> vec2<f32> {
    let d = uv - center;
    let s = sin(radians);
    let c = cos(radians);
    return center + vec2<f32>(d.x * c - d.y * s, d.x * s + d.y * c);
}
