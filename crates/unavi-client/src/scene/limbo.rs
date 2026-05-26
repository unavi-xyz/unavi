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
use bevy_hsd::Hsd;
use unavi_space::Space;

use crate::scene::{
    SceneState,
    respawn::Respawn,
};

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

pub fn exit_limbo_on_space_join(
    trigger: On<Add, Hsd>,
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

    // TODO track asset loads within space to ensure all blobs are downloaded

    info!("Space loaded, exiting limbo");
    next.set(SceneState::Space);

    commands.trigger(Respawn);
}
