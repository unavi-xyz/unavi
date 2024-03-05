use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use unavi_app::UnaviPlugin;

fn main() {
    unavi_app::App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                file_path: "../../assets".to_string(),
                ..default()
            }),
            UnaviPlugin {
                debug_physics: true,
                ..default()
            },
        ))
        .add_systems(Startup, setup_world)
        .run();
}

fn setup_world(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn(SceneBundle {
        scene: asset_server.load("models/catbot.vrm#Scene0"),
        transform: Transform::from_xyz(-3.0, 0.0, -10.0).with_rotation(Quat::from_rotation_y(PI)),
        ..default()
    });

    // Pyramid of cubes
    // {
    //     let box_size = 0.1;
    //     let pyramid_layers = 8;
    //     let pyramid_material = materials.add(StandardMaterial::from(Color::rgb(0.7, 0.8, 0.6)));
    //     let pyramid_offset = Transform::from_xyz(5.0, 1.0, -10.0);
    //
    //     for i in 1..=pyramid_layers {
    //         for j in 0..i {
    //             for k in 0..i {
    //                 let i = i as f32;
    //                 let j = j as f32;
    //                 let k = k as f32;
    //                 let pyramid_layers = pyramid_layers as f32;
    //
    //                 let x = (j - (i / 2.0)) * box_size * 2.0;
    //                 let y = (pyramid_layers - i) * box_size * 2.0;
    //                 let z = (k - (i / 2.0)) * box_size * 2.0;
    //
    //                 commands.spawn((
    //                     PhysShapeBundle::cube(
    //                         box_size,
    //                         pyramid_offset * Transform::from_xyz(x, y, z),
    //                         pyramid_material.clone(),
    //                         &mut meshes,
    //                     ),
    //                     ColliderDensity(0.5),
    //                 ));
    //             }
    //         }
    //     }
    // }

    // Ball
    {
        let ball_material = materials.add(StandardMaterial::from(Color::rgb(0.9, 0.3, 0.3)));
        let ball_radius = 0.75;

        commands.spawn((
            PhysShapeBundle::sphere(
                ball_radius,
                Transform::from_xyz(0.0, ball_radius, -10.0),
                ball_material.clone(),
                &mut meshes,
            ),
            Mass(8.0),
        ));
    }
}

#[derive(Bundle)]
struct PhysShapeBundle {
    collider: Collider,
    pbr_bundle: PbrBundle,
    rigid_body: RigidBody,
}

impl PhysShapeBundle {
    fn cube(
        size: f32,
        transform: Transform,
        material: Handle<StandardMaterial>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> Self {
        Self {
            collider: Collider::cuboid(size, size, size),
            rigid_body: RigidBody::Dynamic,
            pbr_bundle: PbrBundle {
                transform,
                mesh: meshes.add(Cuboid {
                    half_size: Vec3::splat(size),
                }),
                material,
                ..default()
            },
        }
    }

    fn sphere(
        radius: f32,
        transform: Transform,
        material: Handle<StandardMaterial>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> Self {
        Self {
            collider: Collider::sphere(radius),
            rigid_body: RigidBody::Dynamic,
            pbr_bundle: PbrBundle {
                transform,
                mesh: meshes.add(Sphere { radius }),
                material,
                ..default()
            },
        }
    }
}
