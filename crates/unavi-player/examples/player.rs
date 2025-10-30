use avian3d::{
    PhysicsPlugins,
    prelude::{Collider, PhysicsDebugPlugin, RigidBody},
};
use bevy::{
    color::palettes::tailwind::BLUE_400,
    core_pipeline::{auto_exposure::AutoExposure, bloom::Bloom},
    pbr::{Atmosphere, AtmosphereSettings, CascadeShadowConfigBuilder, light_consts::lux},
    prelude::*,
    render::{camera::Exposure, mesh::VertexAttributeValues, view::RenderLayers},
};
use bevy_vrm::first_person::{FirstPersonFlag, RENDER_LAYERS};
use unavi_input::InputPlugin;
use unavi_player::{PlayerPlugin, PlayerSpawner};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                file_path: "../unavi/assets".to_string(),
                ..Default::default()
            }),
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            InputPlugin,
            PlayerPlugin,
        ))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, handle_input)
        .run();
}

const SIZE: f32 = 64.0;

fn setup_scene(
    asset_server: Res<AssetServer>,
    mut ambient: ResMut<AmbientLight>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    ambient.brightness = lux::OVERCAST_DAY;

    PlayerSpawner::default().spawn(&mut commands, &asset_server);

    commands.spawn((
        SkyCamera,
        Camera {
            hdr: true,
            is_active: false,
            ..Default::default()
        },
        Camera3d::default(),
        Transform::from_xyz(5.0, 4.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        Atmosphere::EARTH,
        AtmosphereSettings::default(),
        AutoExposure {
            range: -4.0..=8.0,
            ..Default::default()
        },
        Exposure::SUNLIGHT,
        Bloom::OLD_SCHOOL,
        RenderLayers::layer(0).union(&RENDER_LAYERS[&FirstPersonFlag::ThirdPersonOnly]),
    ));

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
    ));

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

    commands.spawn((
        Collider::cuboid(4.0, 1.0, 4.0),
        Mesh3d(meshes.add(Cuboid::new(4.0, 1.0, 4.0))),
        MeshMaterial3d(materials.add(Color::from(BLUE_400))),
        RigidBody::Static,
        Transform::from_xyz(-3.0, 0.75, -6.0),
    ));
}

#[derive(Component)]
struct SkyCamera;

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_cam: Query<&mut Camera, Without<SkyCamera>>,
    mut sky_cam: Query<&mut Camera, With<SkyCamera>>,
) {
    if keyboard.just_pressed(KeyCode::KeyP) {
        let mut pc = player_cam.iter_mut().next().unwrap();
        let mut sc = sky_cam.iter_mut().next().unwrap();

        if pc.is_active {
            pc.is_active = false;
            sc.is_active = true;
        } else {
            pc.is_active = true;
            sc.is_active = false;
        }
    }
}
