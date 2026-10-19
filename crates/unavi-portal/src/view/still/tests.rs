use bevy::prelude::*;

use super::FACES;

/// `still_sample`'s face and uv selection in `portal.wgsl`, line for line.
fn shader_face_uv(d: Vec3) -> (usize, Vec2) {
    let a = d.abs();
    let face = if a.x >= a.y && a.x >= a.z {
        usize::from(d.x <= 0.0)
    } else if a.y >= a.z {
        2 + usize::from(d.y <= 0.0)
    } else {
        4 + usize::from(d.z <= 0.0)
    };
    let (forward, up) = FACES[face];
    let right = forward.cross(up);
    let depth = d.dot(forward);
    let x = d.dot(right) / depth;
    let y = d.dot(up) / depth;
    (face, Vec2::new(f32::midpoint(x, 1.0), f32::midpoint(1.0, -y)))
}

#[test]
fn every_still_face_camera_agrees_with_the_shader_lookup() {
    for (face, (forward, up)) in FACES.iter().enumerate() {
        let camera = Transform::IDENTITY.looking_to(*forward, *up);
        for uv in [
            Vec2::new(0.5, 0.5),
            Vec2::new(0.1, 0.2),
            Vec2::new(0.8, 0.3),
            Vec2::new(0.25, 0.9),
        ] {
            // A 90° camera's pixel at `uv`, top-left origin.
            let ndc = Vec2::new(uv.x.mul_add(2.0, -1.0), uv.y.mul_add(-2.0, 1.0));
            let dir = camera.right() * ndc.x + camera.up() * ndc.y + camera.forward().as_vec3();

            let (got_face, got_uv) = shader_face_uv(dir);
            assert_eq!(got_face, face, "pixel {uv} of face {face} looked up on {got_face}");
            assert!(
                got_uv.distance(uv) < 1.0e-5,
                "pixel {uv} of face {face} looked up at {got_uv}"
            );
        }
    }
}
