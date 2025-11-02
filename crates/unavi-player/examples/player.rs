use std::num::NonZero;

use avian3d::{
    PhysicsPlugins,
    prelude::{Collider, PhysicsDebugPlugin, RigidBody},
};
use bevy::{
    color::palettes::tailwind::{
        BLUE_400, EMERALD_400, ORANGE_400, PURPLE_400, RED_400, YELLOW_400,
    },
    core_pipeline::{auto_exposure::AutoExposure, bloom::Bloom},
    pbr::{Atmosphere, AtmosphereSettings, CascadeShadowConfigBuilder, light_consts::lux},
    prelude::*,
    render::{camera::Exposure, mesh::VertexAttributeValues, view::RenderLayers},
};
use bevy_rich_text3d::{Text3d, Text3dPlugin, Text3dStyling, TextAtlas};
use bevy_vrm::first_person::{FirstPersonFlag, RENDER_LAYERS};
use unavi_input::InputPlugin;
use unavi_player::{PlayerPlugin, PlayerSpawner};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                file_path: "../unavi-client/assets".to_string(),
                ..Default::default()
            }),
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            InputPlugin,
            PlayerPlugin,
            Text3dPlugin {
                asynchronous_load: true,
                load_system_fonts: true,
                sync_scale_factor_with_main_window: true,
                ..default()
            },
        ))
        .insert_resource(AmbientLight {
            brightness: lux::OVERCAST_DAY,
            ..default()
        })
        .add_systems(Startup, setup_scene)
        .add_systems(Update, handle_input)
        .run();
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

const SIZE: f32 = 64.0;

struct SceneSpawner<'w, 's, 'a> {
    commands: &'a mut Commands<'w, 's>,
    meshes: &'a mut Assets<Mesh>,
    materials: &'a mut Assets<StandardMaterial>,
}

impl<'w, 's, 'a> SceneSpawner<'w, 's, 'a> {
    fn text(&mut self, text: &str, transform: Transform) {
        let mat = self.materials.add(StandardMaterial {
            base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
            alpha_mode: AlphaMode::Mask(0.5),
            unlit: true,
            cull_mode: None,
            ..Default::default()
        });

        self.commands.spawn((
            Text3d::parse_raw(text).unwrap(),
            Text3dStyling {
                font: "monospace".into(),
                layer_offset: 0.001,
                size: 64.0,
                stroke: NonZero::new(8),
                stroke_color: Srgba::BLACK,
                world_scale: Some(Vec2::splat(0.3)),
                ..default()
            },
            Mesh3d::default(),
            MeshMaterial3d(mat),
            transform,
        ));
    }

    fn slope(&mut self, position: Vec3, width: f32, height: f32, depth: f32, color: Color) {
        let angle = (height / depth).atan();
        let hypotenuse = (height.powi(2) + depth.powi(2)).sqrt();

        let slope_mesh = self.meshes.add(Cuboid::new(width, 0.2, hypotenuse));
        let rotation = Quat::from_rotation_y(std::f32::consts::PI) * Quat::from_rotation_x(-angle);

        // Position the slope so it touches the ground at the start and reaches height at the end
        let offset_y = hypotenuse / 2.0 * angle.sin() - 0.1;
        let offset_z = -hypotenuse / 2.0 * angle.cos();

        self.commands.spawn((
            Collider::cuboid(width, 0.2, hypotenuse),
            Mesh3d(slope_mesh),
            MeshMaterial3d(self.materials.add(color)),
            RigidBody::Static,
            Transform::from_translation(position + Vec3::new(0.0, offset_y, offset_z))
                .with_rotation(rotation),
        ));

        let angle_deg = angle.to_degrees();
        let text = format!("{:.0}°", angle_deg);
        self.text(
            &text,
            Transform::from_translation(position + Vec3::new(0.0, height + 0.8, -depth / 2.0)),
        );
    }

    fn steps(
        &mut self,
        start_position: Vec3,
        width: f32,
        height: f32,
        depth: f32,
        count: usize,
        color: Color,
    ) {
        for i in 0..count {
            let step_mesh = self.meshes.add(Cuboid::new(width, height, depth));
            let y = height * (i as f32 + 0.5);
            let z = depth * i as f32;

            self.commands.spawn((
                Collider::cuboid(width, height, depth),
                Mesh3d(step_mesh),
                MeshMaterial3d(self.materials.add(color)),
                RigidBody::Static,
                Transform::from_translation(start_position + Vec3::new(0.0, y, z)),
            ));
        }

        let total_height = height * count as f32;
        let total_depth = depth * count as f32;
        let text = format!("{:.2}m", height);
        self.text(
            &text,
            Transform::from_translation(
                start_position + Vec3::new(0.0, total_height + 0.5, total_depth / 2.0),
            )
            .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        );
    }
}

fn setup_scene(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    PlayerSpawner::default().spawn(&mut commands, &asset_server);

    commands.spawn((
        SkyCamera,
        Camera {
            hdr: true,
            is_active: false,
            ..Default::default()
        },
        Camera3d::default(),
        Transform::from_translation(Vec3::splat(8.0)).looking_at(Vec3::ZERO, Vec3::Y),
        Atmosphere::EARTH,
        AtmosphereSettings::default(),
        AutoExposure {
            range: -4.0..=8.0,
            ..Default::default()
        },
        Exposure::SUNLIGHT,
        Bloom::OLD_SCHOOL,
        Msaa::default(),
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

    let mut spawner = SceneSpawner {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
    };

    let depth = 6.0;

    // 15° slope
    spawner.slope(
        Vec3::new(-12.0, 0.0, -10.0),
        4.0,
        depth * 15f32.to_radians().tan(),
        depth,
        Color::from(BLUE_400),
    );

    // 30° slope
    spawner.slope(
        Vec3::new(-6.0, 0.0, -10.0),
        4.0,
        depth * 30f32.to_radians().tan(),
        depth,
        Color::from(PURPLE_400),
    );

    // 45° slope
    spawner.slope(
        Vec3::new(0.0, 0.0, -10.0),
        4.0,
        depth * 45f32.to_radians().tan(),
        depth,
        Color::from(RED_400),
    );

    // 60° slope
    spawner.slope(
        Vec3::new(6.0, 0.0, -10.0),
        4.0,
        depth * 60f32.to_radians().tan(),
        depth,
        Color::from(ORANGE_400),
    );

    // Small steps (easy)
    spawner.steps(
        Vec3::new(-12.0, 0.0, 2.0),
        3.0,
        0.2,
        0.5,
        10,
        Color::from(YELLOW_400),
    );

    // Medium steps
    spawner.steps(
        Vec3::new(-6.0, 0.0, 2.0),
        3.0,
        0.35,
        0.6,
        8,
        Color::from(EMERALD_400),
    );

    // Large steps (challenging)
    spawner.steps(
        Vec3::new(0.0, 0.0, 2.0),
        3.0,
        0.5,
        0.8,
        6,
        Color::from(BLUE_400),
    );
}
