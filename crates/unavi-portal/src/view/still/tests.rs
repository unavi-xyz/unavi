use std::f32::consts::FRAC_PI_2;

use bevy::{
    camera::CameraProjection,
    prelude::*,
};

use super::{
    FACES,
    NEAR,
};

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
    (
        face,
        Vec2::new(f32::midpoint(x, 1.0), f32::midpoint(1.0, -y)),
    )
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
            assert_eq!(
                got_face, face,
                "pixel {uv} of face {face} looked up on {got_face}"
            );
            assert!(
                got_uv.distance(uv) < 1.0e-5,
                "pixel {uv} of face {face} looked up at {got_uv}"
            );
        }
    }
}

/// `still_distance.wgsl`'s depth-to-distance conversion, line for line.
fn shader_distance(depth: f32, ndc: Vec2) -> f32 {
    NEAR / depth * Vec3::new(ndc.x, ndc.y, 1.0).length()
}

#[test]
fn the_distance_pass_recovers_distance_from_a_face_cameras_depth() {
    let clip_from_view = PerspectiveProjection {
        fov: FRAC_PI_2,
        aspect_ratio: 1.0,
        near: NEAR,
        ..default()
    }
    .get_clip_from_view();

    for point in [
        Vec3::new(0.3, -0.2, -2.0),
        Vec3::new(-5.0, 4.0, -9.0),
        Vec3::new(0.0, 0.0, -0.5),
        Vec3::new(40.0, -40.0, -80.0),
    ] {
        let clip = clip_from_view * point.extend(1.0);
        let ndc = clip.xy() / clip.w;
        let depth = clip.z / clip.w;
        let got = shader_distance(depth, ndc);
        assert!(
            (got - point.length()).abs() < 1.0e-3 * point.length(),
            "{point} read back as {got}"
        );
    }
}
