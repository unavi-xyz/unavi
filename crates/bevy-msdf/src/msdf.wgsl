#import bevy_pbr::forward_io::VertexOutput

struct MsdfSettings {
    color: vec4<f32>,
    outline_color: vec4<f32>,
    unit_range: vec2<f32>,
    outline_width: f32,
    emissive: f32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> settings: MsdfSettings;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var field_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var field_sampler: sampler;

/// Each channel holds the distance to a different edge; their median is the
/// true distance, which keeps corners sharp.
fn median(rgb: vec3<f32>) -> f32 {
    return max(min(rgb.r, rgb.g), min(max(rgb.r, rgb.g), rgb.b));
}

/// Screen pixels the baked distance range covers here, from the rate the
/// texture coordinate changes across the quad.
fn screen_px_range(uv: vec2<f32>) -> f32 {
    let texels = vec2<f32>(1.0) / fwidth(uv);
    return max(0.5 * dot(settings.unit_range, texels), 1.0);
}

fn coverage(distance: f32, range: f32, threshold: f32) -> f32 {
    return clamp(range * (distance - threshold) + 0.5, 0.0, 1.0);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSample(field_texture, field_sampler, in.uv);
    let range = screen_px_range(in.uv);

    let glyph = coverage(median(sample.rgb), range, 0.5) * settings.color.a;
    var rgb = settings.color.rgb;
    var alpha = glyph;

    if settings.outline_width > 0.0 {
        // The fourth channel is a plain signed distance rather than a third
        // corner distance, so it is the one that stays smooth out where the
        // outline lives; a median taken this far from an edge would trace the
        // corner seams.
        let outer = coverage(sample.a, range, 0.5 - settings.outline_width);
        let border = outer * settings.outline_color.a;
        alpha = glyph + border * (1.0 - glyph);
        if alpha > 0.0 {
            rgb = (settings.color.rgb * glyph
                + settings.outline_color.rgb * border * (1.0 - glyph)) / alpha;
        }
    }

    return vec4<f32>(rgb * (1.0 + settings.emissive), alpha);
}
