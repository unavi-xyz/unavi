use avian3d::prelude::*;
use bevy::{prelude::*, render::mesh::VertexAttributeValues};

use crate::{WorldRecord, WorldServer, WorldState};

pub fn set_loading_state(
    mut next_state: ResMut<NextState<WorldState>>,
    worlds: Query<Entity, (With<Handle<Scene>>, With<WorldRecord>, With<WorldServer>)>,
) {
    if worlds.is_empty() {
        next_state.set(WorldState::Loading);
    } else {
        next_state.set(WorldState::InWorld);
    }
}

#[derive(Component)]
pub struct LoadingScene;

pub fn spawn_loading_scene(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut entity = commands.spawn((LoadingScene, SceneBundle::default()));

    entity.with_children(|builder| {
        let ground_size = 50.0;
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

pub fn despawn_loading_scene(mut commands: Commands, scenes: Query<Entity, With<LoadingScene>>) {
    if scenes.is_empty() {
        warn!("No loading scene to despawn.");
    }

    for entity in scenes.iter() {
        commands.entity(entity).despawn();
    }
}
