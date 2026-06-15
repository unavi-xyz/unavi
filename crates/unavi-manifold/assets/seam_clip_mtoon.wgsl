#import bevy_pbr::forward_io::VertexOutput
#import bevy_shader_mtoon::mtoon::mtoon_shade
#import unavi_manifold::seam_clip::{seam_clipped, seam_cap_normal}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> @location(0) vec4<f32> {
    if seam_clipped(in.world_position.xyz) {
        discard;
    }

    // Exposed back faces are the mesh interior; flatten them to the cut cap.
    // `mtoon_shade` re-flips double-sided back-face normals, so feed the
    // negated cap normal to land on the outward-facing cap.
    var v = in;
    if !is_front {
        v.world_normal = -seam_cap_normal();
    }

    return mtoon_shade(v, is_front);
}
