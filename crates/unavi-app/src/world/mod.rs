use bevy::{prelude::*, render::mesh::VertexAttributeValues};
use bevy_rapier3d::{dynamics::RigidBody, geometry::Collider};

use crate::state::AppState;

mod skybox;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_home)
            .add_systems(
                OnEnter(AppState::InWorld),
                (setup_world, skybox::create_skybox),
            )
            .add_systems(
                Update,
                (skybox::add_skybox_to_cameras, skybox::process_cubemap)
                    .run_if(in_state(AppState::InWorld)),
            );
    }
}

/// Load the user's home world
fn load_home(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::InWorld);
}

fn setup_world(
    mut commands: Commands,
    mut ambient: ResMut<AmbientLight>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    ambient.color = Color::rgb(0.95, 0.95, 1.0);
    ambient.brightness = 0.1;

    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                illuminance: 10_000.0,
                color: Color::rgb(1.0, 1.0, 0.98),
                ..default()
            },
            transform: Transform::from_xyz(-4.5, 10.0, 7.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        bevy_vrm::mtoon::MtoonSun,
    ));

    {
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

        commands.spawn((
            RigidBody::Fixed,
            Collider::cuboid(ground_size / 2.0, 0.05, ground_size / 2.0),
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
    }
}
