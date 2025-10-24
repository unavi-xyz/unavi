use avian3d::prelude::*;
use bevy::{
    color::palettes::tailwind::BLUE_400,
    pbr::{CascadeShadowConfigBuilder, light_consts::lux},
    prelude::*,
    render::mesh::VertexAttributeValues,
};
use unavi_player::{PlayerSpawner, bevy_vrm::mtoon::MtoonSun};

pub fn spawn_lights(mut commands: Commands, mut ambient: ResMut<AmbientLight>) {
    ambient.brightness = lux::OVERCAST_DAY;
    commands.spawn((
        CascadeShadowConfigBuilder {
            maximum_distance: SIZE * 1.2,
            ..Default::default()
        }
        .build(),
        DirectionalLight {
            illuminance: lux::DIRECT_SUNLIGHT / 2.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(1.0, 0.4, 0.1).looking_at(Vec3::ZERO, Vec3::Y),
        MtoonSun,
    ));
}

const SIZE: f32 = 128.0;

pub fn spawn_scene(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    PlayerSpawner::default().spawn(&mut commands, &asset_server);

    let ground_texture = asset_server.load("images/dev-white.png");

    let mut ground_mesh = Plane3d::default().mesh().size(SIZE, SIZE).build();
    match ground_mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0).unwrap() {
        VertexAttributeValues::Float32x2(uvs) => {
            const TEXTURE_SCALE: f32 = 4.0;
            const UV_SCALE: f32 = SIZE / TEXTURE_SCALE;
            for uv in uvs {
                uv[0] *= UV_SCALE;
                uv[1] *= UV_SCALE;
            }
        }
        _ => unreachable!(),
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

    let x = 4.0;
    let y = 4.0;
    let z = 8.0;

    commands.spawn((
        Collider::cuboid(x, y, z),
        Mesh3d(meshes.add(Cuboid::new(x, y, z))),
        MeshMaterial3d(materials.add(Color::from(BLUE_400))),
        RigidBody::Static,
        Transform::from_xyz(-x * 2.0, y / 2.0, -z * 2.0),
    ));
}
