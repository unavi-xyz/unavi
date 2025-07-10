use avian3d::{
    PhysicsPlugins,
    prelude::{Collider, RigidBody},
};
use bevy::{
    color::palettes::tailwind::BLUE_400,
    pbr::{CascadeShadowConfigBuilder, light_consts::lux},
    prelude::*,
    render::mesh::VertexAttributeValues,
};
use unavi_input::InputPlugin;
use unavi_player::{PlayerPlugin, PlayerSpawner};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                file_path: "../unavi-app/assets".to_string(),
                ..Default::default()
            }),
            PhysicsPlugins::default(),
            InputPlugin,
            PlayerPlugin,
        ))
        .add_systems(Startup, setup_scene)
        .run();
}

const SIZE: f32 = 32.0;

fn setup_scene(
    asset_server: Res<AssetServer>,
    mut ambient: ResMut<AmbientLight>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Player
    PlayerSpawner::default().spawn(&mut commands);

    // Lighting
    ambient.brightness = lux::LIVING_ROOM;
    commands.spawn((
        CascadeShadowConfigBuilder {
            maximum_distance: SIZE * 1.2,
            ..Default::default()
        }
        .build(),
        DirectionalLight {
            illuminance: lux::RAW_SUNLIGHT,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(1.0, 0.4, 0.1).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ground
    let ground_texture = asset_server.load("images/dev-white.png");

    let mut ground_mesh = Plane3d::default().mesh().size(SIZE, SIZE).build();
    match ground_mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0).unwrap() {
        VertexAttributeValues::Float32x2(uvs) => {
            const TEXTURE_SCALE: f32 = 4.0;

            let uv_scale = SIZE / TEXTURE_SCALE;

            for uv in uvs {
                uv[0] *= uv_scale;
                uv[1] *= uv_scale;
            }
        }
        _ => panic!(),
    }

    commands.spawn((
        Collider::half_space(Vec3::Y),
        Mesh3d(meshes.add(ground_mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(ground_texture),
            perceptual_roughness: 0.8,
            ..Default::default()
        })),
        RigidBody::Static,
    ));

    // Platform
    commands.spawn((
        Collider::cuboid(4.0, 1.0, 4.0),
        Mesh3d(meshes.add(Cuboid::new(4.0, 1.0, 4.0))),
        MeshMaterial3d(materials.add(Color::from(BLUE_400))),
        RigidBody::Static,
        Transform::from_xyz(-3.0, 0.75, -6.0),
    ));
}
