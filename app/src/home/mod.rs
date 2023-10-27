use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod skybox;

pub struct HomePlugin;

impl Plugin for HomePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_world, skybox::setup_skybox))
            .add_systems(Update, (skybox::create_skybox, skybox::process_skybox));
    }
}

const GROUND_SIZE: f32 = 40.0;
const GROUND_THICKNESS: f32 = 0.1;
const BALL_RADIUS: f32 = 0.5;

fn setup_world(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ambient: ResMut<AmbientLight>,
) {
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(GROUND_SIZE / 2.0, GROUND_THICKNESS / 2.0, GROUND_SIZE / 2.0),
        PbrBundle {
            mesh: meshes.add(
                (shape::Box {
                    min_x: -GROUND_SIZE / 2.0,
                    max_x: GROUND_SIZE / 2.0,
                    min_y: -GROUND_THICKNESS / 2.0,
                    max_y: GROUND_THICKNESS / 2.0,
                    min_z: -GROUND_SIZE / 2.0,
                    max_z: GROUND_SIZE / 2.0,
                })
                .into(),
            ),
            material: materials.add(Color::rgb(0.8, 0.8, 0.8).into()),
            transform: Transform::from_xyz(0.0, -0.1, 0.0),
            ..default()
        },
    ));

    let box_material = materials.add(Color::rgb(0.8, 0.7, 0.6).into());

    for i in 0..4 {
        for j in 0..4 {
            spawn_cube(
                &mut commands,
                Transform::from_xyz(
                    (i as f32 - 1.5) * 10.0,
                    0.5 + j as f32,
                    (j as f32 - 1.5) * 10.0,
                ),
                box_material.clone(),
                &mut meshes,
            );
        }
    }

    commands.spawn((
        RigidBody::Dynamic,
        Collider::ball(BALL_RADIUS),
        Restitution::coefficient(0.7),
        PbrBundle {
            mesh: meshes.add(
                (shape::Icosphere {
                    radius: BALL_RADIUS,
                    ..default()
                })
                .try_into()
                .unwrap(),
            ),
            material: materials.add(Color::rgb(0.9, 0.3, 0.3).into()),
            transform: Transform::from_xyz(0.0, 8.0, -5.0),
            ..default()
        },
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 10_000.0,
            color: Color::rgb(1.0, 1.0, 0.98),
            ..default()
        },
        transform: Transform::from_xyz(-4.5, 10.0, 7.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    ambient.color = Color::rgb(0.95, 0.95, 1.0);
    ambient.brightness = 0.1;
}

fn spawn_cube(
    commands: &mut Commands,
    transform: Transform,
    material: Handle<StandardMaterial>,
    meshes: &mut ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(0.5, 0.5, 0.5),
        PbrBundle {
            mesh: meshes.add(
                (shape::Box {
                    min_x: -0.5,
                    max_x: 0.5,
                    min_y: -0.5,
                    max_y: 0.5,
                    min_z: -0.5,
                    max_z: 0.5,
                })
                .into(),
            ),
            material,
            transform,
            ..default()
        },
    ));
}
