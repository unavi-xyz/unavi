use std::f32::consts::FRAC_PI_2;

use bevy::{
    color::palettes::tailwind::{BLUE_500, ORANGE_500},
    light::light_consts::lux,
    prelude::*,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use unavi_portal::{PortalPlugin, PortalTraveler, create::CreatePortal};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Portal Example".to_string(),
                    ..default()
                }),
                ..default()
            }),
            PanOrbitCameraPlugin,
            PortalPlugin,
        ))
        .insert_resource(AmbientLight {
            brightness: lux::HALLWAY,
            ..default()
        })
        .add_systems(Startup, setup_scene)
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let portal_width = 2.0;
    let portal_height = 3.0;

    // Spawn camera with panorbit controls and portal traveler.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera::default(),
        PortalTraveler,
    ));

    // Spawn linked portal pair.
    let portal_left_transform = Transform::from_xyz(-6.0, portal_height / 2.0, 0.0)
        .with_rotation(Quat::from_rotation_y(FRAC_PI_2));
    let portal_right_transform = Transform::from_xyz(6.0, portal_height / 2.0, 0.0)
        .with_rotation(Quat::from_rotation_y(-FRAC_PI_2));

    let id_left = commands.spawn(portal_left_transform).id();
    let id_right = commands.spawn(portal_right_transform).id();

    commands.entity(id_left).queue(CreatePortal {
        destination: Some(id_right),
        height: portal_height,
        width: portal_width,
        ..Default::default()
    });
    commands.entity(id_right).queue(CreatePortal {
        destination: Some(id_left),
        height: portal_height,
        width: portal_width,
        ..Default::default()
    });

    // Ground plane.
    let ground_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(20.0)));
    let ground_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.3, 0.3),
        perceptual_roughness: 0.9,
        ..default()
    });

    commands.spawn((
        Mesh3d(ground_mesh),
        MeshMaterial3d(ground_material),
        Transform::from_xyz(0.0, -0.0001, 0.0),
    ));

    // Reference cubes for spatial orientation.
    let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    // Cube near portal A.
    let cube_material_a = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.2, 0.2),
        ..default()
    });
    commands.spawn((
        Mesh3d(cube_mesh.clone()),
        MeshMaterial3d(cube_material_a),
        Transform::from_xyz(-6.0, 0.5, 3.0),
    ));

    // Cube near portal B.
    let cube_material_b = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.8, 0.2),
        ..default()
    });
    commands.spawn((
        Mesh3d(cube_mesh.clone()),
        MeshMaterial3d(cube_material_b),
        Transform::from_xyz(6.0, 0.5, 3.0),
    ));

    // Center cube for reference.
    let cube_material_c = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.2, 0.8),
        ..default()
    });
    commands.spawn((
        Mesh3d(cube_mesh),
        MeshMaterial3d(cube_material_c),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // Directional light.
    commands.spawn((
        DirectionalLight {
            illuminance: lux::FULL_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    spawn_portal_frame(
        &mut commands,
        &mut meshes,
        &mut materials,
        portal_left_transform,
        portal_width,
        portal_height,
        Color::Srgba(BLUE_500),
    );

    spawn_portal_frame(
        &mut commands,
        &mut meshes,
        &mut materials,
        portal_right_transform,
        portal_width,
        portal_height,
        Color::Srgba(ORANGE_500),
    );
}

fn spawn_portal_frame(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    portal_transform: Transform,
    width: f32,
    height: f32,
    color: Color,
) {
    let thickness = 0.1;
    let depth = 0.05;

    let frame_material = materials.add(StandardMaterial {
        base_color: color,
        emissive: color.into(),
        ..default()
    });

    let top_bar = meshes.add(Cuboid::new(width + thickness * 2.0, thickness, depth));
    commands.spawn((
        Mesh3d(top_bar),
        MeshMaterial3d(frame_material.clone()),
        portal_transform.with_translation(
            portal_transform.translation
                + portal_transform.rotation.mul_vec3(Vec3::new(
                    0.0,
                    height / 2.0 + thickness / 2.0,
                    0.0,
                )),
        ),
    ));

    let left_bar = meshes.add(Cuboid::new(thickness, height, depth));
    commands.spawn((
        Mesh3d(left_bar),
        MeshMaterial3d(frame_material.clone()),
        portal_transform.with_translation(
            portal_transform.translation
                + portal_transform.rotation.mul_vec3(Vec3::new(
                    -(width / 2.0 + thickness / 2.0),
                    0.0,
                    0.0,
                )),
        ),
    ));

    let right_bar = meshes.add(Cuboid::new(thickness, height, depth));
    commands.spawn((
        Mesh3d(right_bar),
        MeshMaterial3d(frame_material),
        portal_transform.with_translation(
            portal_transform.translation
                + portal_transform.rotation.mul_vec3(Vec3::new(
                    width / 2.0 + thickness / 2.0,
                    0.0,
                    0.0,
                )),
        ),
    ));
}
