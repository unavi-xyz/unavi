#import bevy_pbr::{
    forward_io::{Vertex, VertexOutput},
    mesh_view_bindings::{globals, view},
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var live_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1)
var live_sampler: sampler;

struct PortalParams {
    world_from_portal: mat4x4<f32>,
    rim_color:         vec4<f32>,
    ring:              vec4<f32>,
    half_size:         vec2<f32>,
    mode:              u32,
    // 1 once the distance atlas is bound.
    parallax:          u32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(2)
var<uniform> params: PortalParams;

// The arrival still: `still::FACES` in tiles of `ATLAS_COLUMNS` per row.
@group(#{MATERIAL_BIND_GROUP}) @binding(3)
var still_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4)
var still_sampler: sampler;
// Each texel's distance from its face's eye, in the still's layout. Read
// unfiltered: a blend across a depth edge is a distance that is nowhere.
@group(#{MATERIAL_BIND_GROUP}) @binding(5)
var distance_texture: texture_2d<f32>;

const MODE_LIVE: u32 = 0u;
const MODE_STILL: u32 = 1u;
const MODE_LOADING: u32 = 2u;
const MODE_VEIL: u32 = 3u;

const ATLAS_COLUMNS: u32 = 3u;
const ATLAS_ROWS: u32 = 2u;
const STILL_TILES: vec2<f32> = vec2<f32>(f32(ATLAS_COLUMNS), f32(ATLAS_ROWS));

// Parallax ray march through the still. Steps are spaced quadratically, fine
// near the portal and coarse far off, where parallax is small.
const STILL_STEPS: u32 = 32u;
const STILL_REFINE_STEPS: u32 = 5u;
// Metres the march reaches. A ray that hits nothing in this range shows the
// still as infinitely far.
const STILL_RANGE: f32 = 64.0;
// The furthest a ray may start from the capture eye, in metres. A larger
// portal gets less parallax rather than more stretching at disocclusions.
const STILL_MAX_START: f32 = 1.5;

// Rim core and glow widths, in metres.
const RIM_CORE: f32 = 0.012;
const RIM_GLOW: f32 = 0.08;
const RIM_GAIN: f32 = 1.3;
// Viewer distances, in metres, over which approach effects fade in.
const APPROACH_NEAR: f32 = 0.4;
const APPROACH_FAR: f32 = 1.6;
const RING_SECS: f32 = 0.5;
const RING_SPEED: f32 = 5.0;
// Live-view refraction along the rim on approach, in pixels.
const RIPPLE_PX: f32 = 4.0;

struct FragmentOutput {
    @builtin(frag_depth) depth: f32,
    @location(0) color:        vec4<f32>,
}

// The portal is composited per pixel by intersecting the view ray with the
// portal plane, so the rasterized quad only bounds the pixels to test. While
// every corner is in front of the eye the quad is the portal's own corners,
// pushed to the far plane so near-plane clipping cannot slice it. Otherwise
// it is the whole screen. Every vertex makes the same choice, so the quad is
// never mixed.
@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let corner = sign(vertex.position.xy);
    var in_front = true;
    for (var i = 0u; i < 4u; i += 1u) {
        let c = vec2<f32>(select(-1.0, 1.0, (i & 1u) != 0u), select(-1.0, 1.0, (i & 2u) != 0u));
        let world = params.world_from_portal * vec4<f32>(c * params.half_size, 0.0, 1.0);
        in_front = in_front && (view.clip_from_world * world).w > 1.0e-3;
    }

    var out: VertexOutput;
    if in_front {
        let world = params.world_from_portal * vec4<f32>(corner * params.half_size, 0.0, 1.0);
        let clip = view.clip_from_world * world;
        out.position = vec4<f32>(clip.xy, 0.0, clip.w);
    } else {
        out.position = vec4<f32>(corner, 1.0, 1.0);
    }
    return out;
}

struct StillCoord {
    // The tile in tiles, and the position in it in `[0, 1]`.
    tile: vec2<f32>,
    uv:   vec2<f32>,
}

// Face and uv selection is mirrored in `still/tests.rs`, which checks it
// against the capture cameras.
fn still_coord(d: vec3<f32>) -> StillCoord {
    let a = abs(d);
    var face = 5u;
    if a.x >= a.y && a.x >= a.z {
        face = select(1u, 0u, d.x > 0.0);
    } else if a.y >= a.z {
        face = select(3u, 2u, d.y > 0.0);
    } else if d.z > 0.0 {
        face = 4u;
    }

    var forward = vec3<f32>(0.0, 0.0, -1.0);
    var up = vec3<f32>(0.0, 1.0, 0.0);
    switch face {
        case 0u: { forward = vec3<f32>(1.0, 0.0, 0.0); }
        case 1u: { forward = vec3<f32>(-1.0, 0.0, 0.0); }
        case 2u: { forward = vec3<f32>(0.0, 1.0, 0.0); up = vec3<f32>(0.0, 0.0, 1.0); }
        case 3u: { forward = vec3<f32>(0.0, -1.0, 0.0); up = vec3<f32>(0.0, 0.0, -1.0); }
        case 4u: { forward = vec3<f32>(0.0, 0.0, 1.0); }
        default: {}
    }
    let right = cross(forward, up);
    let depth = dot(d, forward);
    let x = dot(d, right) / depth;
    let y = dot(d, up) / depth;

    // Half a texel in from the tile edge so filtering never reads the next
    // tile.
    let tile_texels = vec2<f32>(textureDimensions(still_texture)) / STILL_TILES;
    let half_texel = 0.5 / tile_texels;
    let uv = clamp(vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5), half_texel, 1.0 - half_texel);
    let tile = vec2<f32>(f32(face % ATLAS_COLUMNS), f32(face / ATLAS_COLUMNS));
    return StillCoord(tile, uv);
}

// The still as seen from the capture eye along `d`.
fn still_sample(d: vec3<f32>) -> vec3<f32> {
    let c = still_coord(d);
    return textureSampleLevel(still_texture, still_sampler, (c.tile + c.uv) / STILL_TILES, 0.0).rgb;
}

// The distance from the capture eye to the surface along `d`.
fn still_distance_at(d: vec3<f32>) -> f32 {
    let c = still_coord(d);
    let texel = vec2<i32>((c.tile + c.uv) / STILL_TILES * vec2<f32>(textureDimensions(distance_texture)));
    return textureLoad(distance_texture, texel, 0).r;
}

// Where a ray from `start`, a point on the portal plane in the capture eye's
// frame, meets the surface the still recorded, as a vector from the capture
// eye. A point is behind the surface once it is further from the eye than the
// surface is in its direction. Nothing between the viewer and the plane is
// hit, since the march begins at the plane. A ray that hits nothing returns
// its direction: the still as infinitely far, as before parallax.
fn still_march(start: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
    var lo = 0.0;
    var hi = -1.0;
    for (var i = 1u; i <= STILL_STEPS; i += 1u) {
        let f = f32(i) / f32(STILL_STEPS);
        let t = STILL_RANGE * f * f;
        let p = start + dir * t;
        if length(p) >= still_distance_at(p) {
            hi = t;
            break;
        }
        lo = t;
    }
    if hi < 0.0 {
        return dir;
    }

    for (var i = 0u; i < STILL_REFINE_STEPS; i += 1u) {
        let mid = 0.5 * (lo + hi);
        let p = start + dir * mid;
        if length(p) >= still_distance_at(p) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    return start + dir * hi;
}

// A slow, low-contrast interference pattern in [0, 1].
fn caustic(p: vec2<f32>, t: f32) -> f32 {
    let a = sin(p.x * 6.0 + t * 0.5 + sin(p.y * 4.0 - t * 0.35) * 1.7);
    let b = sin(p.y * 5.0 - t * 0.4 + sin(p.x * 3.0 + t * 0.3) * 1.3);
    return 0.5 + 0.5 * a * b;
}

fn luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
    let center = params.world_from_portal[3].xyz;
    let normal = normalize(params.world_from_portal[2].xyz);
    let x_axis = normalize(params.world_from_portal[0].xyz);
    let y_axis = normalize(params.world_from_portal[1].xyz);

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
    let local = vec2<f32>(dot(offset, x_axis), dot(offset, y_axis));
    let half_size = params.half_size;
    if abs(local.x) > half_size.x || abs(local.y) > half_size.y {
        discard;
    }

    let time = globals.time;
    let rim_color = params.rim_color.rgb;
    let edge = min(half_size.x - abs(local.x), half_size.y - abs(local.y));

    // Distance from the viewer to the nearest point of the opening.
    let to_viewer = ray_origin - center;
    let viewer_local = clamp(
        vec2<f32>(dot(to_viewer, x_axis), dot(to_viewer, y_axis)),
        -half_size,
        half_size,
    );
    let viewer_distance = length(to_viewer - x_axis * viewer_local.x - y_axis * viewer_local.y);
    let approach = 1.0 - smoothstep(APPROACH_NEAR, APPROACH_FAR, viewer_distance);
    // Approach effects fade out again at contact so the crossing stays clean.
    let contact = smoothstep(0.05, 0.35, viewer_distance);

    let veil_pattern = caustic(local, time);
    let veil_color = rim_color * (0.28 + 0.12 * veil_pattern);

    var color = veil_color;
    if params.mode == MODE_LIVE {
        let wave = sin(edge * 45.0 - time * 4.0) * exp(-edge / 0.12) * approach * contact;
        let center_clip = view.clip_from_world * vec4<f32>(center, 1.0);
        let center_ndc = center_clip.xy / center_clip.w;
        let center_px = view.viewport.xy
            + vec2<f32>(center_ndc.x * 0.5 + 0.5, 0.5 - center_ndc.y * 0.5) * view.viewport.zw;
        let push = normalize(in.position.xy - center_px + vec2<f32>(1.0e-4));
        let dims = vec2<f32>(textureDimensions(live_texture));
        let screen_uv = (in.position.xy - view.viewport.xy + push * wave * RIPPLE_PX) / dims;
        color = textureSampleLevel(live_texture, live_sampler, screen_uv, 0.0).rgb;
    } else if params.mode == MODE_STILL {
        // The portal's frame is the arrival's, turned half around when seen
        // from behind. The capture eye is at the arrival's origin, so the
        // opening's plane passes through it, and the view ray crosses that
        // plane at `start`.
        var d = vec3<f32>(dot(ray_dir, x_axis), dot(ray_dir, y_axis), dot(ray_dir, normal));
        var start = vec3<f32>(local, 0.0);
        if dot(to_viewer, normal) < 0.0 {
            d = vec3<f32>(-d.x, d.y, -d.z);
            start = vec3<f32>(-start.x, start.y, 0.0);
        }
        var still = vec3<f32>(0.0);
        if params.parallax != 0u {
            start *= min(1.0, STILL_MAX_START / max(length(start), 1.0e-4));
            still = still_sample(still_march(start, d));
        } else {
            still = still_sample(d);
        }
        let veil = mix(0.2, 0.8, smoothstep(0.5, 3.0, viewer_distance));
        let tinted = mix(still, luminance(still) * normalize(rim_color + vec3<f32>(1.0e-4)) * 1.7, 0.5 * veil);
        color = mix(tinted, veil_color, 0.5 * veil);
    } else if params.mode == MODE_VEIL {
        color = veil_color;
    } else if params.mode == MODE_LOADING {
        let rise = fract(time * 0.25);
        let height = local.y / (2.0 * half_size.y) + 0.5;
        let front = exp(-abs(height - rise) / 0.015);
        let filled = select(0.0, 0.06, height < rise);
        color = veil_color + rim_color * (filled + 0.5 * front);
    }

    let fresnel = 1.0 - abs(denom);
    var rim = exp(-edge / RIM_CORE) + 0.35 * exp(-edge / RIM_GLOW) * (0.5 + fresnel);
    rim *= 1.0 + approach * contact;

    if params.ring.w > 0.0 {
        let age = time - params.ring.z;
        if age >= 0.0 && age < RING_SECS {
            let radius = age * RING_SPEED;
            let d = distance(local, params.ring.xy);
            rim += 1.5 * exp(-abs(d - radius) / 0.04) * (1.0 - age / RING_SECS);
        }
    }

    color += rim_color * rim * RIM_GAIN;

    let clip = view.clip_from_world * vec4<f32>(hit, 1.0);
    var out: FragmentOutput;
    out.depth = clip.z / clip.w;
    out.color = vec4<f32>(color, 1.0);
    return out;
}
