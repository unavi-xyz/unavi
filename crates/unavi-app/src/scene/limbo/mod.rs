//! The empty scene the local agent waits in while no space is ready to stand
//! in: its floor, and the gate that decides when to leave.

use avian3d::prelude::*;
use bevy::{
    image::{
        ImageAddressMode,
        ImageLoaderSettings,
        ImageSampler,
        ImageSamplerDescriptor,
    },
    math::Affine2,
    prelude::*,
};
use unavi_agent::{
    LocalAgent,
    LocalAgentEntities,
    config::AgentConfig,
};
use unavi_space::grid::{
    ActiveSpace,
    SPACE_CELL_SIZE,
    Spaceless,
};

pub mod arrival;
pub mod diagnose;
pub mod ground;

#[cfg(test)] mod tests;

const LIMBO_OFFSET: Vec3 = Vec3::new(SPACE_CELL_SIZE, 0.0, SPACE_CELL_SIZE);
/// Half the floor's extent on each horizontal axis.
const PLANE_SIZE: f32 = 2048.0;
const FLOOR_THICKNESS: f32 = 1.0;
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
        base_color_texture: Some(dev_white),
        clearcoat: 0.4,
        clearcoat_perceptual_roughness: 0.4,
        perceptual_roughness: 0.7,
        uv_transform: Affine2::from_scale(Vec2::splat(PLANE_SIZE / TEXTURE_SIZE)),
        ..Default::default()
    };

    let mesh = Plane3d::new(Vec3::Y, Vec2::splat(PLANE_SIZE));

    // A slab under the visible plane rather than an infinite half-space, hung
    // off the entity's origin so its top face stays at the floor height.
    let floor = Collider::compound(vec![(
        Vec3::new(0.0, -FLOOR_THICKNESS * 0.5, 0.0),
        Quat::IDENTITY,
        Collider::cuboid(PLANE_SIZE * 2.0, FLOOR_THICKNESS, PLANE_SIZE * 2.0),
    )]);

    commands.spawn((
        Limbo,
        RigidBody::Static,
        floor,
        Transform::from_translation(LIMBO_OFFSET),
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
    ));
}

/// Pins the agent body to the limbo floor for as long as limbo lasts.
///
/// Held every tick rather than placed once on entry, since a space becoming
/// active shifts the floor out from under an agent parked at a fixed point.
/// The body's origin is its capsule's centre, so it is held at its float
/// height above the floor rather than buried half in it.
pub fn hold_agent_in_limbo(
    agents: Query<(&LocalAgentEntities, &AgentConfig), With<LocalAgent>>,
    floor: Query<&Transform, With<Limbo>>,
    mut bodies: Query<
        (
            &mut Transform,
            &mut Position,
            &mut LinearVelocity,
            &mut AngularVelocity,
        ),
        Without<Limbo>,
    >,
) {
    let (Ok((ents, config)), Ok(floor)) = (agents.single(), floor.single()) else {
        return;
    };
    let held = floor.translation + Vec3::Y * config.float_height();
    let Ok((mut tr, mut pos, mut vel, mut ang_vel)) = bodies.get_mut(ents.body) else {
        return;
    };

    tr.translation = held;
    pos.0 = held;
    *vel = LinearVelocity::default();
    *ang_vel = AngularVelocity::default();
}

pub fn despawn_limbo(limbo: Query<Entity, With<Limbo>>, mut commands: Commands) {
    for entity in limbo {
        commands.entity(entity).despawn();
    }
}

/// Marks the local agent spaceless and drops the active space, so no system
/// reads its limbo-floor position as a place in a space.
pub fn enter_limbo(
    agents: Query<Entity, (With<LocalAgent>, Without<Spaceless>)>,
    mut active: ResMut<ActiveSpace>,
    mut commands: Commands,
) {
    active.0 = None;
    for entity in &agents {
        commands.entity(entity).insert(Spaceless);
    }
}

/// Lets the agent stand in a space again; the next recenter promotes one.
pub fn leave_limbo(
    agents: Query<Entity, (With<LocalAgent>, With<Spaceless>)>,
    mut commands: Commands,
) {
    for entity in &agents {
        commands.entity(entity).remove::<Spaceless>();
    }
}
