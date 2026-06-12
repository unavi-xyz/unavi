use std::f32::consts::FRAC_PI_2;

use avian3d::prelude::*;
use bevy::{
    camera::visibility::RenderLayers,
    light::light_consts::lux,
    math::Affine2,
    prelude::*,
};
use bevy_panorbit_camera::{
    PanOrbitCamera,
    PanOrbitCameraPlugin,
};
use unavi_manifold::{
    GluedTo,
    ManifoldBody,
    ManifoldPlugin,
    ManifoldViewer,
    PrevTranslation,
    Seam,
    SeamSize,
    transition::CrossedSeam,
    visuals::SEAM_RENDER_LAYER,
};

#[derive(Component)]
struct RollingBall {
    start:      Vec3,
    respawn_at: Option<f32>,
}

const RESPAWN_DELAY: f32 = 2.5;
const BALL_RADIUS: f32 = 0.4;
const BALL_START: Vec3 = Vec3::new(12.0, 3.0, 0.0);

const SEAM_X: f32 = 3.0;
const SEAM_WIDTH: f32 = 2.5;
const SEAM_HEIGHT: f32 = 3.5;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        name: Some("unavi".to_string()),
                        title: "UNAVI".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: concat!(env!("CARGO_MANIFEST_DIR"), "/../unavi-client/assets")
                        .to_string(),
                    ..default()
                }),
            PanOrbitCameraPlugin,
            PhysicsPlugins::default(),
            ManifoldPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.52, 0.62, 0.74)))
        .insert_resource(Gravity(Vec3::NEG_Y * 9.81))
        .insert_resource(GlobalAmbientLight {
            brightness: 350.0,
            ..default()
        })
        .add_systems(Startup, setup_scene)
        .add_systems(Update, respawn_ball)
        .add_observer(schedule_respawn)
        .run();
}

fn schedule_respawn(event: On<CrossedSeam>, time: Res<Time>, mut balls: Query<&mut RollingBall>) {
    if let Ok(mut ball) = balls.get_mut(event.entity)
        && ball.respawn_at.is_none()
    {
        ball.respawn_at = Some(time.elapsed_secs() + RESPAWN_DELAY);
    }
}

fn respawn_ball(
    time: Res<Time>,
    mut balls: Query<(
        &mut RollingBall,
        &mut Transform,
        &mut LinearVelocity,
        &mut AngularVelocity,
        &mut PrevTranslation,
    )>,
) {
    let now = time.elapsed_secs();
    for (mut ball, mut transform, mut linear, mut angular, mut prev) in &mut balls {
        if ball.respawn_at.is_some_and(|t| now >= t) {
            transform.translation = ball.start;
            transform.rotation = Quat::IDENTITY;
            linear.0 = Vec3::ZERO;
            angular.0 = Vec3::ZERO;
            // Reset the manifold's previous translation so the jump back to the
            // ramp top is not mistaken for a seam crossing.
            prev.0 = ball.start;
            ball.respawn_at = None;
        }
    }
}

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        Camera3d::default(),
        PanOrbitCamera {
            focus: Vec3::new(3.0, 1.6, 0.0),
            radius: Some(23.0),
            yaw: Some(0.95),
            pitch: Some(0.42),
            ..default()
        },
        RenderLayers::from_layers(&[0, SEAM_RENDER_LAYER]),
        ManifoldViewer,
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: lux::FULL_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(6.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Checkerboard ground, tiled and tinted.
    let ground_texture = asset_server.load("image/dev-white.png");
    let ground_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.40, 0.48, 0.58),
        base_color_texture: Some(ground_texture),
        uv_transform: Affine2::from_scale(Vec2::splat(20.0)),
        perceptual_roughness: 0.85,
        metallic: 0.0,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(20.0)))),
        MeshMaterial3d(ground_material),
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
    ));

    spawn_ramp(&mut commands, &mut meshes, &mut materials);
    spawn_seams(&mut commands, &mut meshes, &mut materials);
    spawn_landmarks(&mut commands, &mut meshes, &mut materials);
    spawn_ball(&mut commands, &mut meshes, &mut materials);
}

fn spawn_seams(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let seam_size = SeamSize {
        width:  SEAM_WIDTH,
        height: SEAM_HEIGHT,
    };
    // Bottom edge sits on the ground (centre at half-height).
    let seam_a = Transform::from_xyz(SEAM_X, SEAM_HEIGHT / 2.0, 0.0)
        .with_rotation(Quat::from_rotation_y(FRAC_PI_2));
    let seam_b = Transform::from_xyz(-SEAM_X, SEAM_HEIGHT / 2.0, 0.0)
        .with_rotation(Quat::from_rotation_y(-FRAC_PI_2));

    let id_a = commands.spawn((Seam, seam_size, seam_a)).id();
    let id_b = commands.spawn((Seam, seam_size, seam_b)).id();
    commands.entity(id_a).insert(GluedTo(id_b));
    commands.entity(id_b).insert(GluedTo(id_a));

    spawn_seam_frame(
        commands,
        meshes,
        materials,
        seam_a,
        Color::srgb(1.0, 0.55, 0.1),
    );
    spawn_seam_frame(
        commands,
        meshes,
        materials,
        seam_b,
        Color::srgb(0.2, 0.65, 1.0),
    );
}

fn spawn_landmarks(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    for (xyz, rgb) in [
        ([-5.0, 0.5, 1.8], [0.90, 0.20, 0.25]),
        ([-6.5, 0.5, -1.2], [0.25, 0.80, 0.35]),
        ([-4.6, 0.5, -3.0], [0.65, 0.35, 0.90]),
        ([-7.2, 0.5, 0.8], [0.95, 0.80, 0.20]),
        ([5.0, 0.5, 3.4], [0.20, 0.80, 0.85]),
        ([5.6, 0.5, -3.2], [0.95, 0.35, 0.70]),
    ] {
        let mat = materials.add(StandardMaterial {
            base_color: Color::srgb(rgb[0], rgb[1], rgb[2]),
            perceptual_roughness: 0.5,
            ..default()
        });
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(mat),
            Transform::from_xyz(xyz[0], xyz[1], xyz[2]),
            RigidBody::Static,
            Collider::cuboid(1.0, 1.0, 1.0),
        ));
    }
}

fn spawn_ball(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let ball_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.85, 0.12, 0.12),
        metallic: 0.3,
        perceptual_roughness: 0.35,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(BALL_RADIUS))),
        MeshMaterial3d(ball_material),
        Transform::from_translation(BALL_START),
        RigidBody::Dynamic,
        Collider::sphere(BALL_RADIUS),
        LinearVelocity(Vec3::new(-1.0, 0.0, 0.0)),
        Restitution::new(0.3),
        ManifoldBody,
        RollingBall {
            start:      BALL_START,
            respawn_at: None,
        },
    ));
}

fn spawn_ramp(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let length = 7.0;
    let height = 1.0;
    let width = 4.0;
    let center_x = 11.0;

    let hl = length / 2.0;
    let hw = width / 2.0;

    // Triangle profile in XY (bottom on the ground), extruded along Z.
    let profile = Triangle2d::new(
        Vec2::new(-hl, 0.0),
        Vec2::new(hl, 0.0),
        Vec2::new(hl, height),
    );
    let mesh = meshes.add(Extrusion::new(profile, width));

    let hull = Collider::convex_hull(vec![
        Vec3::new(-hl, 0.0, -hw),
        Vec3::new(hl, 0.0, -hw),
        Vec3::new(hl, height, -hw),
        Vec3::new(-hl, 0.0, hw),
        Vec3::new(hl, 0.0, hw),
        Vec3::new(hl, height, hw),
    ])
    .expect("ramp hull");

    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.57, 0.62),
        perceptual_roughness: 0.8,
        ..default()
    });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(center_x, 0.0, 0.0),
        RigidBody::Static,
        hull,
    ));
}

fn spawn_seam_frame(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    seam_transform: Transform,
    color: Color,
) {
    let thickness = 0.12;
    let depth = 0.08;
    let width = SEAM_WIDTH;
    let height = SEAM_HEIGHT;

    let frame_material = materials.add(StandardMaterial {
        base_color: color,
        emissive: (color.to_linear() * 2.0),
        ..default()
    });

    let bar = |offset: Vec3| {
        seam_transform
            .with_translation(seam_transform.translation + seam_transform.rotation.mul_vec3(offset))
    };

    let top_bar = meshes.add(Cuboid::new(width + thickness * 2.0, thickness, depth));
    commands.spawn((
        Mesh3d(top_bar),
        MeshMaterial3d(frame_material.clone()),
        bar(Vec3::new(0.0, height / 2.0 + thickness / 2.0, 0.0)),
    ));

    let left_bar = meshes.add(Cuboid::new(thickness, height, depth));
    commands.spawn((
        Mesh3d(left_bar),
        MeshMaterial3d(frame_material.clone()),
        bar(Vec3::new(-(width / 2.0 + thickness / 2.0), 0.0, 0.0)),
    ));

    let right_bar = meshes.add(Cuboid::new(thickness, height, depth));
    commands.spawn((
        Mesh3d(right_bar),
        MeshMaterial3d(frame_material),
        bar(Vec3::new(width / 2.0 + thickness / 2.0, 0.0, 0.0)),
    ));
}
