#define_import_path unavi_manifold::seam_clip

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var<uniform> clip_plane: vec4<f32>;

// True for fragments behind the seam plane, which a straddling mesh must not
// render so it does not protrude out a portal's back side.
fn seam_clipped(world_position: vec3<f32>) -> bool {
    return dot(clip_plane.xyz, world_position) + clip_plane.w < 0.0;
}

// Outward normal of the flat cap exposed at the cut; shading interior back
// faces with it makes the sliced mesh read as solid.
fn seam_cap_normal() -> vec3<f32> {
    return -normalize(clip_plane.xyz);
}
