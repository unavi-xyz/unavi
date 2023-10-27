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

fn setup_world(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ambient: ResMut<AmbientLight>,
) {
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

    {
        let box_size = 0.1;
        let pyramid_layers = 8;
        let pyramid_material = materials.add(Color::rgb(0.7, 0.8, 0.6).into());
        let pyramid_offset = Transform::from_xyz(5.0, GROUND_THICKNESS * 2.0, -10.0);

        for i in 1..=pyramid_layers {
            for j in 0..i {
                for k in 0..i {
                    let i = i as f32;
                    let j = j as f32;
                    let k = k as f32;
                    let pyramid_layers = pyramid_layers as f32;

                    let x = (j - (i / 2.0)) * box_size * 2.0;
                    let y = (pyramid_layers - i) * box_size * 2.0;
                    let z = (k - (i / 2.0)) * box_size * 2.0;

                    commands.spawn((
                        PhysShapeBundle::cube(
                            box_size,
                            pyramid_offset * Transform::from_xyz(x, y, z),
                            pyramid_material.clone(),
                            &mut meshes,
                        ),
                        ColliderMassProperties::Density(0.25),
                    ));
                }
            }
        }
    }

    {
        let ball_material = materials.add(Color::rgb(0.9, 0.3, 0.3).into());
        let ball_radius = 0.75;

        commands.spawn((
            PhysShapeBundle::sphere(
                ball_radius,
                Transform::from_xyz(0.0, ball_radius, -10.0),
                ball_material.clone(),
                &mut meshes,
            ),
            ColliderMassProperties::Density(8.0),
        ));
    }
}

#[derive(Bundle)]
struct PhysShapeBundle {
    rigid_body: RigidBody,
    collider: Collider,
    pbr_bundle: PbrBundle,
}

impl PhysShapeBundle {
    fn cube(
        size: Real,
        transform: Transform,
        material: Handle<StandardMaterial>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> Self {
        Self {
            collider: Collider::cuboid(size, size, size),
            rigid_body: RigidBody::Dynamic,
            pbr_bundle: PbrBundle {
                transform,
                mesh: meshes.add(
                    (shape::Box {
                        min_x: -size,
                        max_x: size,
                        min_y: -size,
                        max_y: size,
                        min_z: -size,
                        max_z: size,
                    })
                    .into(),
                ),
                material,
                ..default()
            },
        }
    }

    fn sphere(
        radius: Real,
        transform: Transform,
        material: Handle<StandardMaterial>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> Self {
        Self {
            collider: Collider::ball(radius),
            rigid_body: RigidBody::Dynamic,
            pbr_bundle: PbrBundle {
                transform,
                mesh: meshes.add(
                    (shape::Icosphere {
                        radius,
                        ..default()
                    })
                    .try_into()
                    .unwrap(),
                ),
                material,
                ..default()
            },
        }
    }
}
