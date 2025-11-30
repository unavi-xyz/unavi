use std::f32::consts::FRAC_PI_2;

use avian3d::prelude::*;
use bevy::{
    color::palettes::tailwind::{BLUE_400, BLUE_500, ORANGE_500},
    light::{CascadeShadowConfigBuilder, light_consts::lux},
    prelude::*,
};
use bevy_vrm::mtoon::MtoonSun;
use unavi_player::LocalPlayerSpawner;
use unavi_portal::create::CreatePortal;

pub fn spawn_lights(mut commands: Commands, mut ambient: ResMut<AmbientLight>) {
    ambient.brightness = lux::OVERCAST_DAY;
    commands.spawn((
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 5.0,
            maximum_distance: SIZE * 1.2,
            minimum_distance: 0.1,
            num_cascades: 3,
            ..Default::default()
        }
        .build(),
        DirectionalLight {
            illuminance: lux::DIRECT_SUNLIGHT / 2.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(0.4, 1.0, 0.1).looking_at(Vec3::ZERO, Vec3::Y),
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
    let player = LocalPlayerSpawner::default().spawn(&mut commands, &asset_server);

    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/demo.glb")),
    ));

    commands.spawn((
        Collider::half_space(Vec3::Y),
        RigidBody::Static,
        Transform::from_xyz(0.0, 0.0, 0.0),
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

    // Portals
    let portal_distance = 6.0;

    let portal_width = 2.0;
    let portal_height = 3.0;

    let portal_left_transform = Transform::from_xyz(-portal_distance, portal_height / 2.0, 0.0)
        .with_rotation(Quat::from_rotation_y(FRAC_PI_2));
    let portal_right_transform = Transform::from_xyz(portal_distance, portal_height / 2.0, 0.0)
        .with_rotation(Quat::from_rotation_y(-FRAC_PI_2));

    let id_left = commands.spawn(portal_left_transform).id();
    let id_right = commands.spawn(portal_right_transform).id();

    commands.entity(id_left).queue(CreatePortal {
        destination: Some(id_right),
        tracked_camera: Some(player.camera),
        height: portal_height,
        width: portal_width,
        ..default()
    });
    commands.entity(id_right).queue(CreatePortal {
        destination: Some(id_left),
        tracked_camera: Some(player.camera),
        height: portal_height,
        width: portal_width,
        ..default()
    });

    spawn_portal_frame(
        &mut commands,
        &mut meshes,
        &mut materials,
        portal_left_transform,
        portal_width,
        portal_height,
        Color::Srgba(BLUE_500),
    );
    spawn_portal_frame(
        &mut commands,
        &mut meshes,
        &mut materials,
        portal_right_transform,
        portal_width,
        portal_height,
        Color::Srgba(ORANGE_500),
    );
}

fn spawn_portal_frame(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    portal_transform: Transform,
    width: f32,
    height: f32,
    color: Color,
) {
    let thickness = 0.1;
    let depth = 0.05;

    let frame_material = materials.add(StandardMaterial {
        base_color: color,
        emissive: color.into(),
        ..default()
    });

    let top_bar = meshes.add(Cuboid::new(width + thickness * 2.0, thickness, depth));
    commands.spawn((
        Mesh3d(top_bar),
        MeshMaterial3d(frame_material.clone()),
        portal_transform.with_translation(
            portal_transform.translation
                + portal_transform.rotation.mul_vec3(Vec3::new(
                    0.0,
                    height / 2.0 + thickness / 2.0,
                    0.0,
                )),
        ),
    ));

    let left_bar = meshes.add(Cuboid::new(thickness, height, depth));
    commands.spawn((
        Mesh3d(left_bar),
        MeshMaterial3d(frame_material.clone()),
        portal_transform.with_translation(
            portal_transform.translation
                + portal_transform.rotation.mul_vec3(Vec3::new(
                    -(width / 2.0 + thickness / 2.0),
                    0.0,
                    0.0,
                )),
        ),
    ));

    let right_bar = meshes.add(Cuboid::new(thickness, height, depth));
    commands.spawn((
        Mesh3d(right_bar),
        MeshMaterial3d(frame_material),
        portal_transform.with_translation(
            portal_transform.translation
                + portal_transform.rotation.mul_vec3(Vec3::new(
                    width / 2.0 + thickness / 2.0,
                    0.0,
                    0.0,
                )),
        ),
    ));
}
