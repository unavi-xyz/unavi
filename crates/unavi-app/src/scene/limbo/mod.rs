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
use unavi_portal::body::PrevTranslation;
use unavi_space::grid::{
    ActiveSpace,
    SPACE_CELL_SIZE,
    Spaceless,
};

pub mod arrival;
pub mod ground;

#[cfg(test)] mod tests;

const LIMBO_OFFSET: Vec3 = Vec3::new(SPACE_CELL_SIZE, 0.0, SPACE_CELL_SIZE);
/// Where on the floor the agent is put down, kept off the floor entity's own
/// origin: a body spawned exactly there picks up a phantom step and drops
/// when it moves away. A few metres to the side is plain floor.
const LIMBO_SPAWN_OFFSET: Vec3 = Vec3::new(3.0, 0.0, 3.0);
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

/// Marks a local agent already put down on the limbo floor this stay.
#[derive(Component)]
pub(super) struct PlacedInLimbo;

/// Puts the agent down on the limbo floor once per stay, then lets the normal
/// character controller carry it.
///
/// Pinning the body every tick fought gravity, Tnua's walk spring and the
/// solver's contact response, and that feedback grew into a bounce that could
/// drive the camera into the floor. Limbo is a place to stand and walk in, not
/// a loading screen that holds the player still.
pub fn place_agent_in_limbo(
    agents: Query<
        (Entity, &LocalAgentEntities, &AgentConfig),
        (With<LocalAgent>, Without<PlacedInLimbo>),
    >,
    floor: Query<&Transform, With<Limbo>>,
    mut bodies: Query<
        (
            &mut Transform,
            Option<&mut Position>,
            &mut LinearVelocity,
            &mut AngularVelocity,
            Option<&mut PrevTranslation>,
        ),
        Without<Limbo>,
    >,
    mut commands: Commands,
) {
    let Ok(floor) = floor.single() else {
        return;
    };
    for (agent, ents, config) in &agents {
        let Ok((mut tr, pos, mut vel, mut ang_vel, prev)) = bodies.get_mut(ents.body) else {
            continue;
        };

        // The body's origin is its capsule's centre, so it sits a float height
        // above the floor. Writing `Position` too keeps the jump out of the
        // transform sync and out of portal-crossing detection.
        let spawn = floor.translation + LIMBO_SPAWN_OFFSET + Vec3::Y * config.float_height();
        tr.translation = spawn;
        if let Some(mut pos) = pos {
            pos.0 = spawn;
        }
        if let Some(mut prev) = prev {
            prev.0 = Some(spawn);
        }
        *vel = LinearVelocity::default();
        *ang_vel = AngularVelocity::default();

        commands.entity(agent).insert(PlacedInLimbo);
        commands.entity(ents.body).remove::<Sleeping>();
    }
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
        commands
            .entity(entity)
            .remove::<(Spaceless, PlacedInLimbo)>();
    }
}
