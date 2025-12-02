use std::f32::consts::{FRAC_PI_2, FRAC_PI_3};

use avian3d::prelude::*;
use bevy::{
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
            maximum_distance: 50.0,
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
        Transform::from_xyz(-0.9, 10.0, 3.8).looking_at(Vec3::ZERO, Vec3::Y),
        MtoonSun,
    ));
}

pub fn spawn_scene(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let player = LocalPlayerSpawner::default().spawn(&mut commands, &asset_server);

    // commands
    //     .entity(player.external_root)
    //     .insert(Transform::from_rotation(Quat::from_rotation_y(-FRAC_PI_2)));

    let portal_width = 2.0;
    let portal_height = 3.0;

    let portal_a = space_a(
        &asset_server,
        &mut commands,
        &mut materials,
        &mut meshes,
        portal_width,
        portal_height,
    );
    let portal_b = space_b(
        &asset_server,
        &mut commands,
        &mut materials,
        &mut meshes,
        portal_width,
        portal_height,
    );

    commands.entity(portal_a).queue(CreatePortal {
        destination: Some(portal_b),
        tracked_camera: Some(player.camera),
        height: portal_height,
        width: portal_width,
        ..default()
    });
    commands.entity(portal_b).queue(CreatePortal {
        destination: Some(portal_a),
        tracked_camera: Some(player.camera),
        height: portal_height,
        width: portal_width,
        ..default()
    });
}

fn space_a(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    portal_width: f32,
    portal_height: f32,
) -> Entity {
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/demo.glb")),
    ));

    commands.spawn((
        Collider::half_space(Vec3::Y),
        RigidBody::Static,
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    let portal_transform = Transform::from_xyz(2.0, portal_height / 2.0, -15.765);

    spawn_portal_frame(
        commands,
        meshes,
        materials,
        portal_transform,
        portal_width,
        portal_height,
        Color::BLACK,
    );

    commands.spawn(portal_transform).id()
}

fn space_b(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    portal_width: f32,
    portal_height: f32,
) -> Entity {
    let space_offset = 1000.0;

    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/mc-room.glb"))),
        Transform::from_xyz(space_offset, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));

    commands.spawn((
        Collider::half_space(Vec3::Y),
        RigidBody::Static,
        Transform::from_xyz(space_offset, 0.0, 0.0),
    ));

    let portal_transform = Transform::from_xyz(space_offset - 2.0, portal_height / 2.0, -4.5);

    spawn_portal_frame(
        commands,
        meshes,
        materials,
        portal_transform,
        portal_width,
        portal_height,
        Color::BLACK,
    );

    commands.spawn(portal_transform).id()
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
    let depth = 0.1;

    let frame_material = materials.add(StandardMaterial {
        base_color: color,
        perceptual_roughness: 0.9,
        metallic: 0.8,
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
