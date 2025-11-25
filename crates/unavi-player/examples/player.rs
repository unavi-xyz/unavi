use std::{collections::HashMap, num::NonZero};

use avian3d::{
    PhysicsPlugins,
    prelude::{Collider, RigidBody},
};
use bevy::{
    camera::{Exposure, visibility::RenderLayers},
    color::palettes::tailwind::{BLUE_400, ORANGE_400, PURPLE_400, YELLOW_400},
    light::{CascadeShadowConfigBuilder, NotShadowCaster, light_consts::lux},
    mesh::VertexAttributeValues,
    pbr::{Atmosphere, AtmosphereSettings},
    post_process::{auto_exposure::AutoExposure, bloom::Bloom},
    prelude::*,
    render::view::Hdr,
};
use bevy_rich_text3d::{Text3d, Text3dPlugin, Text3dStyling, TextAtlas};
use bevy_vrm::first_person::{DEFAULT_RENDER_LAYERS, FirstPersonFlag};
use unavi_input::InputPlugin;
use unavi_player::{LocalPlayerSpawner, PlayerPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: "../unavi-client/assets".to_string(),
                    ..Default::default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        name: Some("unavi".to_string()),
                        title: "UNAVI".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
            PhysicsPlugins::default(),
            // PhysicsDebugPlugin,
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
        .add_systems(Update, (handle_input, update_platforms))
        .run();
}

#[derive(Component)]
struct SkyCamera;

#[derive(Component)]
struct HorizontalPlatform {
    speed: f32,
    range: f32,
    start_x: f32,
}

#[derive(Component)]
struct VerticalPlatform {
    speed: f32,
    range: f32,
    start_y: f32,
}

fn update_platforms(
    time: Res<Time>,
    mut horizontal: Query<(&HorizontalPlatform, &mut Transform), Without<VerticalPlatform>>,
    mut vertical: Query<(&VerticalPlatform, &mut Transform), Without<HorizontalPlatform>>,
) {
    let elapsed = time.elapsed_secs();

    for (platform, mut transform) in &mut horizontal {
        let offset = (elapsed * platform.speed).sin() * platform.range;
        transform.translation.x = platform.start_x + offset;
    }

    for (platform, mut transform) in &mut vertical {
        let offset = (elapsed * platform.speed).sin() * platform.range;
        transform.translation.y = platform.start_y + offset;
    }
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_cam: Query<&mut Camera, Without<SkyCamera>>,
    mut sky_cam: Query<&mut Camera, With<SkyCamera>>,
) {
    if keyboard.just_pressed(KeyCode::KeyP) {
        let mut pc = player_cam.iter_mut().next().expect("value expected");
        let mut sc = sky_cam.iter_mut().next().expect("value expected");

        if pc.is_active {
            pc.is_active = false;
            sc.is_active = true;
        } else {
            pc.is_active = true;
            sc.is_active = false;
        }
    }
}

struct SceneSpawner<'w, 's, 'a> {
    commands: &'a mut Commands<'w, 's>,
    meshes: &'a mut Assets<Mesh>,
    materials: &'a mut Assets<StandardMaterial>,
    dev_texture: Handle<Image>,
    material_cache: HashMap<u32, Handle<StandardMaterial>>,
}

#[derive(Default, Clone)]
struct BoxConfig {
    position: Vec3,
    rotation: Quat,
    size: Vec3,
    color: Color,
    rigidbody: Option<RigidBody>,
}

impl SceneSpawner<'_, '_, '_> {
    fn text(&mut self, text: &str, transform: Transform) {
        let mat = self.materials.add(StandardMaterial {
            base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
            alpha_mode: AlphaMode::Mask(0.5),
            unlit: true,
            cull_mode: None,
            ..Default::default()
        });

        self.commands.spawn((
            Text3d::parse_raw(text).expect("value expected"),
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
            NotShadowCaster,
            transform,
        ));
    }

    fn get_or_create_material(&mut self, color: Color) -> Handle<StandardMaterial> {
        let color_key = color.to_linear().as_u32();

        if let Some(handle) = self.material_cache.get(&color_key) {
            return handle.clone();
        }

        let handle = self.materials.add(StandardMaterial {
            base_color: color,
            base_color_texture: Some(self.dev_texture.clone()),
            perceptual_roughness: 0.9,
            ..Default::default()
        });
        self.material_cache.insert(color_key, handle.clone());
        handle
    }

    fn spawn_box(&mut self, config: BoxConfig) {
        let mut box_mesh = Cuboid::from_size(config.size).mesh().build();

        // Scale UVs based on box size, per face.
        if let Some(VertexAttributeValues::Float32x2(uvs)) =
            box_mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
        {
            const TEXTURE_SCALE: f32 = 4.0;

            for (i, uv) in uvs.iter_mut().enumerate() {
                let face = i / 4; // 6 faces, 4 vertices each.
                match face {
                    0 | 1 => {
                        // Front / Back
                        uv[0] *= config.size.x / TEXTURE_SCALE;
                        uv[1] *= config.size.y / TEXTURE_SCALE;
                    }
                    2 | 3 => {
                        // Left / Right
                        uv[0] *= config.size.y / TEXTURE_SCALE;
                        uv[1] *= config.size.z / TEXTURE_SCALE;
                    }
                    4 | 5 => {
                        // Top / Bottom
                        uv[0] *= config.size.x / TEXTURE_SCALE;
                        uv[1] *= config.size.z / TEXTURE_SCALE;
                    }
                    _ => {}
                }
            }
        }

        let material = self.get_or_create_material(config.color);

        let mut entity = self.commands.spawn((
            Mesh3d(self.meshes.add(box_mesh)),
            MeshMaterial3d(material),
            Transform {
                translation: config.position,
                rotation: config.rotation,
                ..Default::default()
            },
        ));

        if let Some(rigidbody) = config.rigidbody {
            entity.insert((
                Collider::cuboid(config.size.x, config.size.y, config.size.z),
                rigidbody,
            ));
        }
    }

    fn slope(&mut self, position: Vec3, width: f32, height: f32, depth: f32, color: Color) {
        let angle = (height / depth).atan();
        let hypotenuse = height.hypot(depth);

        let rotation = Quat::from_rotation_y(std::f32::consts::PI) * Quat::from_rotation_x(-angle);

        // Position the slope so it touches the ground at the start and reaches height at the end.
        let offset_y = (hypotenuse / 2.0).mul_add(angle.sin(), -0.1);
        let offset_z = -hypotenuse / 2.0 * angle.cos();

        self.spawn_box(BoxConfig {
            position: position + Vec3::new(0.0, offset_y, offset_z),
            rotation,
            size: Vec3::new(width, 0.2, hypotenuse),
            color,
            rigidbody: Some(RigidBody::Static),
        });

        let angle_deg = angle.to_degrees();
        let text = format!("{angle_deg:.0}°");
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
            let y = height * (i as f32 + 0.5);
            let z = depth * i as f32;

            self.spawn_box(BoxConfig {
                position: start_position + Vec3::new(0.0, y, z),
                size: Vec3::new(width, height, depth),
                color,
                rigidbody: Some(RigidBody::Static),
                ..Default::default()
            });
        }

        let total_height = height * count as f32;
        let total_depth = depth * count as f32;
        let text = format!("{height:.2}m");
        self.text(
            &text,
            Transform::from_translation(
                start_position + Vec3::new(0.0, total_height + 0.5, total_depth / 2.0),
            )
            .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        );
    }
}

#[allow(clippy::too_many_lines)]
fn setup_scene(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    LocalPlayerSpawner::default().spawn(&mut commands, &asset_server);

    commands.spawn((
        SkyCamera,
        Camera {
            is_active: false,
            ..Default::default()
        },
        Hdr,
        Camera3d::default(),
        Transform::from_translation(Vec3::splat(8.0)).looking_at(Vec3::ZERO, Vec3::Y),
        Atmosphere::EARTH,
        AtmosphereSettings::default(),
        AutoExposure {
            range: -3.0..=6.0,
            ..Default::default()
        },
        Exposure::SUNLIGHT,
        Bloom::OLD_SCHOOL,
        Msaa::default(),
        RenderLayers::layer(0).union(&DEFAULT_RENDER_LAYERS[&FirstPersonFlag::ThirdPersonOnly]),
    ));

    commands.spawn((
        CascadeShadowConfigBuilder {
            maximum_distance: TILE_SIZE * 2.0,
            first_cascade_far_bound: TILE_SIZE / 4.0,
            minimum_distance: 0.05,
            ..Default::default()
        }
        .build(),
        DirectionalLight {
            illuminance: lux::DIRECT_SUNLIGHT / 2.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(1.2, 2.0, 0.5).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let dev_texture = asset_server.load("images/dev-white.png");

    let mut spawner = SceneSpawner {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
        dev_texture,
        material_cache: HashMap::new(),
    };

    // Terrain
    spawn_terrain(&mut spawner, false);
    // spawn_terrain(&mut spawner, true);

    // Slopes
    for (i, angle) in [15f32, 30.0, 45.0, 60.0].into_iter().enumerate() {
        let width = 4.0;
        let depth = 6.0;

        let mut position = Vec3::new(0.0, 0.0, -16.0);
        position.x -= i as f32 * width;

        spawner.slope(
            position,
            width,
            depth * angle.to_radians().tan(),
            depth,
            Color::from(BLUE_400),
        );
    }

    // Steps
    for (i, step_size) in [0.05, 0.1, 0.2, 0.3].into_iter().enumerate() {
        let width = 4.0;
        let count = 10;

        let mut position = Vec3::new(0.0, 0.0, 16.0);
        position.x -= i as f32 * width;

        spawner.steps(
            position,
            width,
            step_size,
            0.5,
            count,
            Color::from(YELLOW_400),
        );
    }

    // Horizontal moving platform
    let h_platform_pos = Vec3::new(-16.0, 2.0, 0.0);
    let h_platform_size = Vec3::new(4.0, 0.5, 4.0);
    let h_material = spawner.get_or_create_material(Color::from(PURPLE_400));
    let h_mesh = spawner.meshes.add(Cuboid::from_size(h_platform_size));

    spawner.commands.spawn((
        HorizontalPlatform {
            speed: 1.0,
            range: 5.0,
            start_x: h_platform_pos.x,
        },
        RigidBody::Kinematic,
        Collider::cuboid(h_platform_size.x, h_platform_size.y, h_platform_size.z),
        Mesh3d(h_mesh),
        MeshMaterial3d(h_material),
        Transform::from_translation(h_platform_pos),
    ));

    // Vertical moving platform
    let range = 8.0;
    let thick = 0.5;
    let v_platform_pos = Vec3::new(-16.0, range - thick / 2.0 + 0.01, 4.0);
    let v_platform_size = Vec3::new(4.0, thick, 4.0);
    let v_material = spawner.get_or_create_material(Color::from(ORANGE_400));
    let v_mesh = spawner.meshes.add(Cuboid::from_size(v_platform_size));

    spawner.commands.spawn((
        VerticalPlatform {
            speed: 0.5,
            range,
            start_y: v_platform_pos.y,
        },
        RigidBody::Kinematic,
        Collider::cuboid(v_platform_size.x, v_platform_size.y, v_platform_size.z),
        Mesh3d(v_mesh),
        MeshMaterial3d(v_material),
        Transform::from_translation(v_platform_pos),
    ));

    // Dynamic pyramid (disabled because lag)
    // let pyramid_position = Vec3::new(16.0, 4.0, 0.0);
    // let pyramid_height = 3;
    // let cube_size = 0.5;
    //
    // for layer in 0..pyramid_height {
    //     let boxes_per_side = pyramid_height - layer;
    //     let layer_y = layer as f32 * cube_size;
    //
    //     for x in 0..boxes_per_side {
    //         for z in 0..boxes_per_side {
    //             let offset_x = (x as f32 - (boxes_per_side - 1) as f32 / 2.0) * cube_size;
    //             let offset_z = (z as f32 - (boxes_per_side - 1) as f32 / 2.0) * cube_size;
    //
    //             spawner.spawn_box(BoxConfig {
    //                 position: pyramid_position + Vec3::new(offset_x, layer_y, offset_z),
    //                 rotation: Quat::IDENTITY,
    //                 size: Vec3::splat(cube_size),
    //                 color: Color::from(RED_400),
    //                 rigidbody: Some(RigidBody::Dynamic),
    //             });
    //         }
    //     }
    // }
}

const TILE_SIZE: f32 = 32.0;
const N_ROWS: usize = 12;
const N_CENTER: usize = 4;

const PHYSICS_CUTOFF: usize = 2;
const TILE_SCALE_RATE: usize = 4;
const TILE_SCALE_OFFSET: usize = 1; // Adjust for center
const BASE_ROW_STEP: usize = 2;

const ROUNDING_FACTOR: usize = 1;
const ROW_ACCELERATION: f32 = 0.6;

const INVERSE_Y: f32 = 512.0;

fn spawn_terrain(spawner: &mut SceneSpawner, inverse: bool) {
    let y = if inverse { INVERSE_Y } else { 0.0 };

    // Ground
    let mut center_radius = (TILE_SIZE / 2.0) * N_CENTER as f32;

    spawner.spawn_box(BoxConfig {
        position: Vec3::new(0.0, y - 0.5, 0.0),
        size: Vec3::new(center_radius * 2.0, 1.0, center_radius * 2.0),
        color: Color::WHITE,
        rigidbody: if inverse {
            None
        } else {
            Some(RigidBody::Static)
        },
        ..Default::default()
    });

    // Rings
    let mut min_height = 0.0;

    for row in 1..=N_ROWS {
        let tile_scale_pow = (row - TILE_SCALE_OFFSET).max(0) / TILE_SCALE_RATE;
        let tile_scale = 2i32.pow(tile_scale_pow as u32).min(1024);
        info!("row {row}: tile scale: {tile_scale}");
        let tile_size = tile_scale as f32 * TILE_SIZE;

        let row_step = tile_scale_pow + BASE_ROW_STEP;
        let row_inc = row_step as f32 * (row as f32).powf(ROW_ACCELERATION);
        min_height += row_inc;
        let max_height = min_height + row_inc;

        spawn_ring(
            spawner,
            RingConfig {
                center_radius,
                physics: !inverse && row <= PHYSICS_CUTOFF,
                tile_size,
                min_height,
                max_height,
                inverse,
            },
        );

        center_radius += tile_size; // Add a tile to each side
    }
}

struct RingConfig {
    center_radius: f32,
    physics: bool,
    tile_size: f32,
    min_height: f32,
    max_height: f32,
    inverse: bool,
}

fn spawn_ring(spawner: &mut SceneSpawner, config: RingConfig) {
    let length = (config.center_radius * 2.0 / config.tile_size) + 2.0;
    info!("ring length: {length}");
    let length = length.ceil() as isize;

    for x in 0..length {
        for z in 0..length {
            if x != 0 && x != length - 1 && z != 0 && z != length - 1 {
                // Center region
                continue;
            }

            let mut height = rand::random_range(config.min_height..=config.max_height).round();
            height -= height % ROUNDING_FACTOR as f32;

            let x = (x as f32).mul_add(config.tile_size, -config.center_radius)
                - config.tile_size / 2.0;
            let z = (z as f32).mul_add(config.tile_size, -config.center_radius)
                - config.tile_size / 2.0;

            let y = if config.inverse {
                INVERSE_Y - height / 2.0
            } else {
                height / 2.0
            };

            spawner.spawn_box(BoxConfig {
                position: Vec3::new(x, y, z),
                size: Vec3::new(config.tile_size, height, config.tile_size),
                color: Color::WHITE,
                rigidbody: if config.physics {
                    Some(RigidBody::Static)
                } else {
                    None
                },
                ..Default::default()
            });
        }
    }
}
