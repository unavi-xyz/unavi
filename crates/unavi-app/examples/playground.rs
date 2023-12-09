use std::f32::consts::PI;

use bevy::{prelude::*, render::mesh::VertexAttributeValues};
use bevy_rapier3d::prelude::{Real, *};
use unavi_app::{networking::JoinWorld, UnaviPlugin};

fn main() {
    let mut unavi_plugin = UnaviPlugin::debug();
    unavi_plugin.assets_dir = "../../assets".to_string();

    unavi_app::App::new()
        .add_plugins(unavi_plugin)
        .add_systems(Startup, setup_world)
        .add_systems(Update, join_world)
        .run();
}

fn join_world(mut joined: Local<bool>, time: Res<Time>, _writer: EventWriter<JoinWorld>) {
    if *joined {
        return;
    }

    if time.elapsed_seconds() < 1.0 {
        return;
    }

    *joined = true;

    // writer.send(JoinWorld { world_id: 4 });
}

const GROUND_SIZE: f32 = 40.0;
const GROUND_THICKNESS: f32 = 0.01;

#[derive(Component, Default)]
pub struct RepeatTexture;

fn setup_world(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    {
        let ground_texture = asset_server.load("images/dev-white.png");
        let ground_texture_scale = GROUND_SIZE / 4.0;

        let mut ground_mesh = Mesh::from(shape::Plane {
            size: GROUND_SIZE,
            ..default()
        });

        match ground_mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0).unwrap() {
            VertexAttributeValues::Float32x2(uvs) => {
                for uv in uvs {
                    uv[0] *= ground_texture_scale;
                    uv[1] *= ground_texture_scale;
                }
            }
            _ => panic!(),
        }

        commands.spawn((
            RigidBody::Fixed,
            Collider::cuboid(GROUND_SIZE / 2.0, GROUND_THICKNESS / 2.0, GROUND_SIZE / 2.0),
            PbrBundle {
                mesh: meshes.add(ground_mesh),
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(ground_texture.clone()),
                    ..default()
                }),
                transform: Transform::from_xyz(0.0, -0.1, 0.0),
                ..default()
            },
            RepeatTexture,
        ));
    }

    commands.spawn(SceneBundle {
        scene: asset_server.load("models/catbot.vrm#Scene0"),
        transform: Transform::from_xyz(-3.0, 0.0, -10.0).with_rotation(Quat::from_rotation_y(PI)),
        ..default()
    });

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
                        ColliderMassProperties::Density(0.5),
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
