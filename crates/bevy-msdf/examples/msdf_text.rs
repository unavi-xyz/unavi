//! Judges the distance field by eye.
//!
//! Dolly the camera in and out: the near label and the far sign are the same
//! asset, so watch whether either goes soft or starts to shimmer. The busy
//! panel behind the middle row is there to judge the outline.

use bevy::prelude::*;
use bevy_msdf::{
    MsdfPlugin,
    billboard::Billboard,
    mesh::Anchor,
    text::{
        MsdfStyle,
        MsdfText,
        Outline,
    },
};
use bevy_panorbit_camera::{
    PanOrbitCamera,
    PanOrbitCameraPlugin,
};
use msdf::layout::Align;
use smol_str::SmolStr;

const BODY: f32 = 0.06;
const SPECIMEN: &str = "Handgloves — AVATAR 0123";

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanOrbitCameraPlugin, MsdfPlugin))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.06, 0.09)))
        .add_systems(Startup, setup)
        .run();
}

fn text(value: &str, size: f32) -> MsdfText {
    MsdfText {
        value: SmolStr::new(value),
        size,
        anchor: Anchor::Middle,
        align: Align::Center,
        ..Default::default()
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.6, 2.5).looking_at(Vec3::new(0.0, 0.6, 0.0), Vec3::Y),
        PanOrbitCamera::default(),
    ));
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // The same string at the same size, receding. One field, every distance.
    for (index, depth) in [0.0_f32, -3.0, -8.0, -20.0].into_iter().enumerate() {
        commands.spawn((
            text(&format!("{SPECIMEN} at {}m", depth.abs() as i32 + 2), BODY),
            MsdfStyle {
                color: Color::WHITE,
                ..Default::default()
            },
            Transform::from_xyz(0.0, 1.6, depth),
            Name::new(format!("distance {index}")),
        ));
    }

    // A size ladder, to find where the field runs out of texels.
    for (index, size) in [0.012_f32, 0.02, 0.04, 0.08, 0.16].into_iter().enumerate() {
        commands.spawn((
            text(&format!("{}mm  {SPECIMEN}", (size * 1000.0) as i32), size),
            Transform::from_xyz(0.0, (index as f32).mul_add(-0.16, 1.2), 0.0),
        ));
    }

    // Outline against a surface chosen to be hostile to white text.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.4, 0.5, 0.02))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.85, 0.82, 0.3),
            ..Default::default()
        })),
        Transform::from_xyz(0.0, 0.25, -0.02),
    ));
    for (offset, outline) in [
        (-0.55, None),
        (
            0.55,
            Some(Outline {
                color: Color::BLACK,
                width: 0.25,
            }),
        ),
    ] {
        commands.spawn((
            text("legible?", BODY),
            MsdfStyle {
                color: Color::WHITE,
                outline,
                ..Default::default()
            },
            Transform::from_xyz(offset, 0.25, 0.0),
        ));
    }

    // Wrapping, alignment and a billboard that turns to follow the camera.
    commands.spawn((
        MsdfText {
            value: SmolStr::new(
                "A placard wraps at the width it is given, and breaks a word \
                 too long for the box rather than letting it escape.",
            ),
            size: 0.035,
            wrap: Some(1.1),
            anchor: Anchor::Top,
            ..Default::default()
        },
        Transform::from_xyz(-1.8, 1.2, 0.0),
    ));
    commands.spawn((
        text("billboard", 0.05),
        MsdfStyle {
            color: Color::srgb(0.6, 0.85, 1.0),
            emissive: 2.0,
            ..Default::default()
        },
        Billboard::Yaw,
        Transform::from_xyz(1.8, 1.2, 0.0),
    ));
}
