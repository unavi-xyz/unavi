use avian3d::prelude::*;
use bevy::{prelude::*, render::mesh::VertexAttributeValues};
use bevy_vrm::mtoon::MtoonSun;

use crate::WorldRecord;

pub fn setup_lights(mut commands: Commands, mut ambient: ResMut<AmbientLight>) {
    ambient.brightness = 40.0;
    ambient.color = Color::linear_rgb(0.95, 0.95, 1.0);

    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                // https://github.com/awtterpip/bevy_oxr/issues/149
                // shadows_enabled: true,
                illuminance: 5000.0,
                color: Color::linear_rgb(1.0, 1.0, 0.98),
                ..default()
            },
            transform: Transform::from_xyz(-4.5, 10.0, 7.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        MtoonSun,
    ));
}

pub fn create_world_scene(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    worlds: Query<Entity, (With<WorldRecord>, Without<Handle<Scene>>)>,
) {
    for entity in worlds.iter() {
        let mut entity = commands.entity(entity);
        entity.insert(SceneBundle::default());

        entity.with_children(|builder| {
            let ground_size = 30.0;
            let ground_texture = asset_server.load("images/dev-white.png");
            let ground_texture_scale = ground_size / 4.0;

            let mut ground_mesh = Mesh::from(Cuboid {
                half_size: Vec3::new(ground_size / 2.0, 0.05, ground_size / 2.0),
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

            builder.spawn((
                RigidBody::Static,
                Collider::cuboid(ground_size, 0.05, ground_size),
                PbrBundle {
                    mesh: meshes.add(ground_mesh),
                    material: materials.add(StandardMaterial {
                        base_color_texture: Some(ground_texture.clone()),
                        ..default()
                    }),
                    transform: Transform::from_xyz(0.0, -0.1, 0.0),
                    ..default()
                },
            ));
        });
    }
}
