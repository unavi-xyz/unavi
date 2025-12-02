use std::f32::consts::{FRAC_PI_2, FRAC_PI_3, TAU};

use bevy::{
    camera::visibility::RenderLayers,
    color::palettes::tailwind::{BLUE_500, ORANGE_500},
    light::light_consts::lux,
    prelude::*,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use unavi_constants::PORTAL_RENDER_LAYER;
use unavi_portal::{PortalPlugin, PortalTraveler, create::CreatePortal};

#[derive(Component)]
struct MovingSinusoid {
    amplitude: f32,
    period: f32,
    start_time: f32,
}

fn move_sinusoid(time: Res<Time>, mut query: Query<(&mut Transform, &mut MovingSinusoid)>) {
    let delta = time.delta_secs();

    for (mut transform, sinusoid) in &mut query {
        let frequency = TAU / sinusoid.period;
        let velocity = sinusoid.amplitude * frequency;

        // Calculate phase for current time.
        let elapsed = time.elapsed_secs() - sinusoid.start_time;
        let phase = elapsed * frequency;

        // Velocity is derivative of sin: v = amplitude * frequency * cos(phase).
        let current_velocity = velocity * phase.cos();

        // Apply relative movement along forward axis.
        let forward = transform.rotation * Vec3::Y;
        transform.translation += forward * current_velocity * delta;
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    name: Some("unavi".to_string()),
                    title: "UNAVI".to_string(),
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
        .add_systems(Update, move_sinusoid)
        .run();
}

#[allow(clippy::too_many_lines)]
fn setup_scene(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
) {
    let portal_distance = 6.0;

    let portal_width = 3.0;
    let portal_height = 4.0;

    // Spawn camera with panorbit controls and portal traveler.
    let camera_distance = 8.0;
    let tracked_camera = commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(
                portal_distance / 3.0 * camera_distance,
                portal_height * 0.8 * camera_distance,
                portal_distance / 2.0 * camera_distance,
            )
            .looking_at(Vec3::ZERO, Vec3::Y),
            PanOrbitCamera {
                focus: Vec3::new(-portal_distance * 0.8, portal_height / 3.0, 0.0),
                ..default()
            },
            RenderLayers::from_layers(&[0, PORTAL_RENDER_LAYER]),
            PortalTraveler,
        ))
        .id();

    // Spawn linked portal pair.
    let portal_left_transform = Transform::from_xyz(-portal_distance, portal_height / 2.0, 0.0)
        .with_rotation(Quat::from_rotation_y(FRAC_PI_3));
    let portal_right_transform = Transform::from_xyz(portal_distance, portal_height / 2.0, 0.0)
        .with_rotation(Quat::from_rotation_y(-FRAC_PI_2));

    let id_left = commands.spawn(portal_left_transform).id();
    let id_right = commands.spawn(portal_right_transform).id();

    commands.entity(id_left).queue(CreatePortal {
        destination: Some(id_right),
        tracked_camera: Some(tracked_camera),
        height: portal_height,
        width: portal_width,
        ..Default::default()
    });
    commands.entity(id_right).queue(CreatePortal {
        destination: Some(id_left),
        tracked_camera: Some(tracked_camera),
        height: portal_height,
        width: portal_width,
        ..Default::default()
    });

    // Spawn moving test traveler.
    let traveler_mesh = meshes.add(Cone::new(0.25, 0.5));
    let traveler_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.5, 0.0),
        metallic: 0.5,
        perceptual_roughness: 0.3,
        ..default()
    });

    commands.spawn((
        Mesh3d(traveler_mesh),
        MeshMaterial3d(traveler_material),
        Transform::from_xyz(portal_distance + 2.0, portal_height / 3.0, -1.0)
            .with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
        PortalTraveler,
        MovingSinusoid {
            amplitude: portal_distance,
            period: 8.0,
            start_time: time.elapsed_secs(),
        },
    ));

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
        Mesh3d(cube_mesh.clone()),
        MeshMaterial3d(cube_material_c),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // Cube behind portal A (left portal).
    let cube_behind_a = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.2),
        ..default()
    });
    commands.spawn((
        Mesh3d(cube_mesh.clone()),
        MeshMaterial3d(cube_behind_a),
        Transform::from_xyz(-portal_distance - 2.0, 0.5, 0.0),
    ));

    // Cube behind portal B (right portal).
    let cube_behind_b = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.2, 0.8),
        ..default()
    });
    commands.spawn((
        Mesh3d(cube_mesh),
        MeshMaterial3d(cube_behind_b),
        Transform::from_xyz(portal_distance + 2.0, 0.5, 0.0),
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
