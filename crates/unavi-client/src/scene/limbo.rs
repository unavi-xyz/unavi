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
use unavi_agent::{
    LocalAgent,
    LocalAgentEntities,
};
use unavi_space::{
    Space,
    anchor::SPACE_CELL_SIZE,
};
use unavi_util::async_commands::AsyncCommands;

use crate::scene::{
    SceneState,
    respawn::Respawn,
};

/// Delay after a space loads, to allow scripts to execute and spawn the scene.
const SPACE_LOAD_DELAY: Duration = Duration::from_secs(1);
/// Exit limbo anyway if a space never reports loaded, so a missing or broken
/// asset can't strand the local agent on the limbo floor indefinitely.
const SPACE_LOAD_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Component)]
pub struct SpaceLoadDeadline(Duration);

const LIMBO_OFFSET: Vec3 = Vec3::new(SPACE_CELL_SIZE, 0.0, SPACE_CELL_SIZE);
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
    let dev_white = asset_server
        .load_builder()
        .with_settings(|s: &mut ImageLoaderSettings| {
            let mut desc = ImageSamplerDescriptor::nearest();
            desc.address_mode_u = ImageAddressMode::Repeat;
            desc.address_mode_v = ImageAddressMode::Repeat;
            desc.address_mode_w = ImageAddressMode::Repeat;
            s.sampler = ImageSampler::Descriptor(desc);
        })
        .load("image/dev-white.png");

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
        Transform::from_translation(LIMBO_OFFSET),
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
    ));
}

/// Offsets a freshly spawned agent body onto the limbo floor, keeping it clear
/// of the active space loading in at the origin.
pub fn offset_agent_to_limbo(
    trigger: On<Add, LocalAgentEntities>,
    state: Res<State<SceneState>>,
    agents: Query<&LocalAgentEntities>,
    mut bodies: Query<&mut Transform>,
) {
    if !matches!(state.get(), SceneState::Limbo) {
        return;
    }
    let Ok(ents) = agents.get(trigger.entity) else {
        return;
    };
    let Ok(mut tr) = bodies.get_mut(ents.body) else {
        return;
    };
    tr.translation += LIMBO_OFFSET;
}

/// Sets an already-spawned agent body onto the limbo floor on limbo re-entry,
/// where [`offset_agent_to_limbo`]'s spawn trigger never fires.
pub fn park_agent_in_limbo(
    agents: Query<&LocalAgentEntities, With<LocalAgent>>,
    mut bodies: Query<(&mut Transform, &mut LinearVelocity, &mut AngularVelocity)>,
) {
    let Ok(ents) = agents.single() else {
        return;
    };
    let Ok((mut tr, mut vel, mut ang_vel)) = bodies.get_mut(ents.body) else {
        return;
    };
    tr.translation = LIMBO_OFFSET;
    *vel = LinearVelocity::default();
    *ang_vel = AngularVelocity::default();
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
) {
    if !matches!(state.get(), SceneState::Limbo) {
        return;
    }
    if !spaces.contains(trigger.entity) {
        return;
    }

    unavi_util::async_task::spawn_async_task(async {
        n0_future::time::sleep(SPACE_LOAD_DELAY).await;
        info!("Space loaded, exiting limbo");

        if let Err(err) = AsyncCommands::default().trigger(EnterSpace).send().await {
            error!(?err, "Failed to send command");
        }
    });
}

pub fn exit_limbo_on_load_timeout(
    state: Res<State<SceneState>>,
    time: Res<Time>,
    spaces: Query<&SpaceLoadDeadline, (With<Space>, Without<HsdLoaded>)>,
    mut commands: Commands,
) {
    if !matches!(state.get(), SceneState::Limbo) {
        return;
    }
    if spaces.iter().any(|deadline| time.elapsed() >= deadline.0) {
        warn!("Space load timed out, exiting limbo anyway");
        commands.trigger(EnterSpace);
    }
}

#[derive(Event, Default)]
pub struct EnterSpace;

pub fn enter_space(
    _: On<EnterSpace>,
    mut next: ResMut<NextState<SceneState>>,
    mut commands: Commands,
) {
    next.set(SceneState::Space);
    commands.trigger(Respawn);
}
