fn graph_object_scale(m: mat4x4<f32>) -> vec3<f32> {
    return vec3<f32>(length(m[0].xyz), length(m[1].xyz), length(m[2].xyz));
}

// Bevy assigns instance indices while batching, so this is stable only for as
// long as the batch is. A prim whose entity outlives the effect keeps its
// value. One that is despawned and respawned may not.
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

// 1.0 where `level` exceeds the 4x4 Bayer threshold at the pixel `coord` falls
// in. The matrix is the one the website and the cell art dither with.
fn graph_dither(coord: vec2<f32>, level: f32) -> f32 {
    var bayer = array<f32, 16>(0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0);
    let x = u32(i32(floor(coord.x)) & 3);
    let y = u32(i32(floor(coord.y)) & 3);
    let threshold = (bayer[y * 4u + x] + 0.5) / 16.0;
    return select(0.0, 1.0, level > threshold);
}
