use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use avian3d::prelude::*;
use bevy::{
    light::{CascadeShadowConfigBuilder, NotShadowCaster, light_consts::lux},
    prelude::*,
    render::alpha::AlphaMode,
};
use bevy_av1::{PlaybackMode, VideoPlayer, VideoSink, VideoTargetAssets};
use bevy_rich_text3d::{Text3d, Text3dStyling, TextAlign, TextAnchor, TextAtlas, Weight};
use bevy_seedling::{prelude::SpatialBasicNode, sample::SamplePlayer, sample_effects};
use bevy_vrm::mtoon::MtoonSun;
use unavi_player::LocalPlayerSpawner;
use unavi_portal::create::CreatePortal;

const PORTAL_WIDTH: f32 = 1.8;
const PORTAL_HEIGHT: f32 = 2.6;

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

    commands.spawn((
        Collider::half_space(Vec3::Y),
        RigidBody::Static,
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    let portal_a = space_a(&asset_server, &mut commands, &mut materials, &mut meshes);
    let portal_b = space_b(&asset_server, &mut commands, &mut materials, &mut meshes);

    commands.entity(portal_a).queue(CreatePortal {
        destination: Some(portal_b),
        tracked_camera: Some(player.camera),
        height: PORTAL_HEIGHT,
        width: PORTAL_WIDTH,
        ..default()
    });
    commands.entity(portal_b).queue(CreatePortal {
        destination: Some(portal_a),
        tracked_camera: Some(player.camera),
        height: PORTAL_HEIGHT,
        width: PORTAL_WIDTH,
        ..default()
    });
}

fn space_a(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> Entity {
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("model/demo.glb")),
    ));

    commands
        .spawn((
            Transform::from_xyz(19.8, 5.0, -6.0).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
            Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::splat(2.2)))),
            MeshMaterial3d(materials.add(StandardMaterial::default())),
            VideoPlayer::new(asset_server.load("video/piplup.ivf"), PlaybackMode::Loop),
            SamplePlayer::new(asset_server.load("audio/piplup.wav")).looping(),
            sample_effects![SpatialBasicNode::default()],
        ))
        .observe(
            |add: On<Add, VideoSink>,
             mut sinks: Query<(
                &VideoSink,
                &MeshMaterial3d<StandardMaterial>,
                &mut Transform,
            )>,
             mut materials: ResMut<Assets<StandardMaterial>>,
             mut video_targets: ResMut<VideoTargetAssets<StandardMaterial>>| {
                let entity = add.entity;
                if let Ok((sink, mesh_material, mut transform)) = sinks.get_mut(entity)
                    && let Some(material) = materials.get_mut(&mesh_material.0)
                {
                    video_targets.add_target(sink, &mesh_material.0);
                    material.base_color_texture = Some(sink.image().clone());
                    let aspect = sink.width() as f32 / sink.height() as f32;
                    if aspect > 1.0 {
                        transform.scale = Vec3::new(aspect, 1.0, 1.0);
                    } else {
                        transform.scale = Vec3::new(1.0, aspect, 1.0);
                    }
                }
            },
        );

    let portal_transform = Transform::from_xyz(1.95, PORTAL_HEIGHT / 2.0, -15.765);

    spawn_portal_frame(
        commands,
        meshes,
        materials,
        portal_transform,
        PORTAL_WIDTH,
        PORTAL_HEIGHT,
        Color::BLACK,
    );

    spawn_portal_label(
        commands,
        meshes,
        materials,
        portal_transform,
        "Block Room",
        "server1.unavi.xyz",
    );

    commands.spawn(portal_transform).id()
}

fn space_b(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> Entity {
    let space_offset = 1000.0;

    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("model/mc-room.glb"))),
        Transform::from_xyz(space_offset, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));

    let portal_transform = Transform::from_xyz(space_offset - 2.0, PORTAL_HEIGHT / 2.0, -4.5)
        .with_rotation(Quat::from_rotation_y(FRAC_PI_4));

    spawn_portal_frame(
        commands,
        meshes,
        materials,
        portal_transform,
        PORTAL_WIDTH,
        PORTAL_HEIGHT,
        Color::BLACK,
    );

    spawn_portal_label(
        commands,
        meshes,
        materials,
        portal_transform,
        "Dev Box",
        "server2.unavi.xyz",
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

fn spawn_portal_label(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    portal_transform: Transform,
    text: &str,
    subtext: &str,
) {
    let text_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
        alpha_mode: AlphaMode::Mask(0.5),
        unlit: true,
        cull_mode: None,
        ..default()
    });

    let line_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        ..default()
    });

    let mut text_transform = portal_transform.with_translation(
        portal_transform.translation
            + portal_transform.rotation.mul_vec3(Vec3::new(
                -PORTAL_WIDTH + 0.4,
                PORTAL_HEIGHT / 2.0 - 0.2,
                0.0,
            )),
    );

    commands.spawn((
        Text3d::parse_raw(text).expect("valid text"),
        Text3dStyling {
            font: "monospace".into(),
            layer_offset: 0.001,
            size: 64.0,
            stroke: None,
            world_scale: Some(Vec2::splat(0.22)),
            align: TextAlign::Left,
            weight: Weight::SEMIBOLD,
            anchor: TextAnchor::TOP_LEFT,
            ..default()
        },
        Mesh3d::default(),
        MeshMaterial3d(text_material.clone()),
        NotShadowCaster,
        text_transform,
    ));

    text_transform.translation.y -= 0.22;

    commands.spawn((
        Text3d::parse_raw(subtext).expect("valid text"),
        Text3dStyling {
            font: "monospace".into(),
            layer_offset: 0.001,
            size: 64.0,
            stroke: None,
            world_scale: Some(Vec2::splat(0.15)),
            align: TextAlign::Left,
            weight: Weight::SEMIBOLD,
            anchor: TextAnchor::TOP_LEFT,
            ..default()
        },
        Mesh3d::default(),
        MeshMaterial3d(text_material),
        NotShadowCaster,
        text_transform,
    ));

    let line_width = 0.03;
    let line_height = PORTAL_HEIGHT * 0.175;
    let line_depth = 0.02;

    let line_mesh = meshes.add(Cuboid::new(line_width, line_height, line_depth));
    let line_transform = portal_transform.with_translation(
        portal_transform.translation
            + portal_transform.rotation.mul_vec3(Vec3::new(
                -PORTAL_WIDTH / 2.0 - 0.32,
                PORTAL_HEIGHT / 2.0 - line_height / 2.0 + 0.025,
                0.0,
            )),
    );

    commands.spawn((
        Mesh3d(line_mesh),
        MeshMaterial3d(line_material),
        NotShadowCaster,
        line_transform,
    ));
}
