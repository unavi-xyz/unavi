use std::time::Duration;

use avian3d::prelude::*;
use bevy::{
    color::palettes::tailwind,
    image::{
        ImageAddressMode,
        ImageLoaderSettings,
        ImageSampler,
        ImageSamplerDescriptor,
    },
    math::Affine2,
    prelude::*,
};
use bevy_hsd::{
    Hsd,
    loaded::HsdLoaded,
};
use unavi_space::Space;

use crate::scene::{
    SceneState,
    respawn::Respawn,
};

/// Exit limbo anyway if a space never reports loaded, so a missing or broken
/// asset can't strand the local agent on the limbo floor indefinitely.
const SPACE_LOAD_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Component)]
pub struct SpaceLoadDeadline(Duration);

const PLANE_SIZE: f32 = 2048.0;
const TEXTURE_SIZE: f32 = 16.0;

#[derive(Component)]
pub struct Limbo;

pub fn spawn_limbo(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let dev_white =
        asset_server.load_with_settings::<_, ImageLoaderSettings>("image/dev-white.png", |s| {
            let mut desc = ImageSamplerDescriptor::nearest();
            desc.address_mode_u = ImageAddressMode::Repeat;
            desc.address_mode_v = ImageAddressMode::Repeat;
            desc.address_mode_w = ImageAddressMode::Repeat;
            s.sampler = ImageSampler::Descriptor(desc);
        });

    let material = StandardMaterial {
        base_color: tailwind::SKY_100.into(),
        base_color_texture: Some(dev_white),
        clearcoat: 0.4,
        clearcoat_perceptual_roughness: 0.4,
        emissive: tailwind::SKY_500.into(),
        emissive_exposure_weight: 0.4,
        metallic: 0.3,
        perceptual_roughness: 0.7,
        uv_transform: Affine2::from_scale(Vec2::splat(PLANE_SIZE / TEXTURE_SIZE)),
        ..Default::default()
    };

    let mesh = Plane3d::new(Vec3::Y, Vec2::splat(PLANE_SIZE));

    commands.spawn((
        Limbo,
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
        Transform::default(),
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
    ));
}

pub fn despawn_limbo(limbo: Query<Entity, With<Limbo>>, mut commands: Commands) {
    for entity in limbo {
        commands.entity(entity).despawn();
    }
}

pub fn track_space_load(
    trigger: On<Add, Hsd>,
    spaces: Query<(), With<Space>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    if spaces.contains(trigger.entity) {
        commands
            .entity(trigger.entity)
            .insert(SpaceLoadDeadline(time.elapsed() + SPACE_LOAD_TIMEOUT));
    }
}

pub fn exit_limbo_on_space_loaded(
    trigger: On<Add, HsdLoaded>,
    spaces: Query<(), With<Space>>,
    state: Res<State<SceneState>>,
    mut next: ResMut<NextState<SceneState>>,
    mut commands: Commands,
) {
    if !matches!(state.get(), SceneState::Limbo) {
        return;
    }
    if !spaces.contains(trigger.entity) {
        return;
    }

    info!("Space loaded, exiting limbo");
    enter_space(&mut next, &mut commands);
}

pub fn exit_limbo_on_load_timeout(
    state: Res<State<SceneState>>,
    time: Res<Time>,
    spaces: Query<&SpaceLoadDeadline, (With<Space>, Without<HsdLoaded>)>,
    mut next: ResMut<NextState<SceneState>>,
    mut commands: Commands,
) {
    if !matches!(state.get(), SceneState::Limbo) {
        return;
    }
    if spaces.iter().any(|deadline| time.elapsed() >= deadline.0) {
        warn!("Space load timed out, exiting limbo anyway");
        enter_space(&mut next, &mut commands);
    }
}

fn enter_space(next: &mut NextState<SceneState>, commands: &mut Commands) {
    next.set(SceneState::Space);
    commands.trigger(Respawn);
}
