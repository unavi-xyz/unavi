use avian3d::prelude::*;
use bevy::{
    camera::{Exposure, visibility::RenderLayers},
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    pbr::{Atmosphere, AtmosphereSettings},
    post_process::bloom::Bloom,
    prelude::*,
    render::view::Hdr,
};
use bevy_tnua::{
    builtins::{TnuaBuiltinJumpConfig, TnuaBuiltinWalkConfig},
    prelude::*,
};
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use bevy_vrm::first_person::{DEFAULT_RENDER_LAYERS, FirstPersonFlag};
use unavi_avatar::{Avatar, AverageVelocity, VrmPath, default_character_animations};
use unavi_input::raycast::PrimaryRaycastInput;
use unavi_portal::{PortalTraveler, create::PORTAL_RENDER_LAYER};

use crate::{
    AgentCamera, AgentEntities, AgentRig, ControlScheme, ControlSchemeConfig, Grounded,
    config::{AgentConfig, XrMode},
    tracking::{TrackedHead, TrackedPose},
};

const RAYCAST_GRAB_DISTANCE: f32 = 3.0;

pub fn on_local_agent_added(mut world: DeferredWorld, ctx: HookContext) {
    let root = ctx.entity;

    let config = world.get::<AgentConfig>(root).cloned().unwrap_or_default();
    let is_xr = world.get_resource::<XrMode>().is_some_and(|xr| **xr);
    let vrm_path = world.get::<VrmPath>(root).map(|p| p.0.clone());
    let asset_server = world.resource::<AssetServer>().clone();

    let animations = default_character_animations(&asset_server);

    // Command phase.
    let mut commands = world.commands();

    let camera = spawn_camera(&mut commands, &asset_server, is_xr);

    let body = commands
        .spawn((
            AgentRig,
            Grounded(true),
            RigidBody::Dynamic,
            Pickable::IGNORE,
            Collider::capsule(config.effective_vrm_radius(), config.effective_vrm_height()),
            TnuaController::<ControlScheme>::default(),
            TnuaConfig::<ControlScheme>(asset_server.add(ControlSchemeConfig {
                basis: TnuaBuiltinWalkConfig {
                    float_height: config.float_height(),
                    max_slope: 55_f32.to_radians(),
                    ..Default::default()
                },
                jump: TnuaBuiltinJumpConfig {
                    height: config.jump_height,
                    ..Default::default()
                },
            })),
            TnuaAvian3dSensorShape(Collider::cylinder(
                config.effective_vrm_radius() - 0.01,
                0.0,
            )),
            LockedAxes::ROTATION_LOCKED,
            Transform::from_xyz(0.0, config.effective_vrm_height() / 2.0, 0.0),
            PortalTraveler,
        ))
        .id();

    let initial_eye_y = config.effective_vrm_height() / 2.0 - 0.1;
    let tracked_head = commands
        .spawn((
            TrackedHead,
            TrackedPose::new(Vec3::new(0.0, initial_eye_y, 0.0), Quat::IDENTITY),
            Transform::from_xyz(0.0, initial_eye_y, 0.0),
        ))
        .add_child(camera)
        .id();

    if !is_xr {
        // Desktop mode: raycast input from the head.
        commands.entity(tracked_head).insert((
            PrimaryRaycastInput,
            RayCaster::new(Vec3::ZERO, Dir3::NEG_Z)
                .with_max_hits(1)
                .with_solidness(false)
                .with_max_distance(RAYCAST_GRAB_DISTANCE)
                .with_query_filter(SpatialQueryFilter::default().with_excluded_entities([body])),
        ));
    }

    // Avatar (triggers Avatar's on_add hook).
    let mut avatar_cmd = commands.spawn(Avatar);
    if let Some(path) = vrm_path {
        avatar_cmd.insert(VrmPath(path));
    }
    let avatar = avatar_cmd.id();

    commands.entity(avatar).insert((
        AverageVelocity {
            target: Some(body),
            ..Default::default()
        },
        animations,
        Transform::from_xyz(0.0, -config.effective_vrm_height() / 2.0, 0.0),
    ));

    commands.entity(body).add_children(&[avatar, tracked_head]);
    commands.entity(root).add_child(body);
    commands.entity(root).insert(AgentEntities {
        avatar,
        camera,
        body,
        tracked_head,
    });
}

fn spawn_camera(
    commands: &mut Commands,
    #[allow(unused)] asset_server: &AssetServer,
    is_xr: bool,
) -> Entity {
    let fog_color = Color::Srgba(Srgba::from_u8_array([0, 192, 240, 255]));
    let fog_end = 1000.0;

    // TODO in xr mode query and add components to XrCamera
    // TODO move spawn_camera to its own hook

    let camera = commands
        .spawn((
            AgentCamera,
            Projection::Perspective(PerspectiveProjection {
                near: 0.001,
                ..default()
            }),
            Hdr,
            Exposure::SUNLIGHT,
            Bloom::OLD_SCHOOL,
            Msaa::Sample4,
            Atmosphere::EARTH,
            AtmosphereSettings::default(),
            Transform::default().looking_at(Vec3::NEG_Z, Vec3::Y),
            RenderLayers::from_layers(&[0, PORTAL_RENDER_LAYER])
                .union(&DEFAULT_RENDER_LAYERS[&FirstPersonFlag::FirstPersonOnly]),
            DistanceFog {
                color: fog_color,
                falloff: FogFalloff::Linear {
                    start: fog_end * 0.8,
                    end: fog_end,
                },
                ..default()
            },
        ))
        .id();

    if !is_xr {
        commands.entity(camera).insert(Camera3d::default());
    }

    #[cfg(not(target_family = "wasm"))]
    commands
        .entity(camera)
        .insert((bevy::post_process::auto_exposure::AutoExposure {
            range: -4.0..=4.0,
            ..default()
        },));

    #[cfg(all(target_family = "wasm", not(feature = "webgpu")))]
    commands.entity(camera).insert((
        Mesh3d(asset_server.add(Cuboid::from_size(Vec3::splat(fog_end)).mesh().build())),
        MeshMaterial3d(asset_server.add(StandardMaterial {
            base_color: fog_color,
            unlit: true,
            cull_mode: None,
            ..default()
        })),
    ));

    camera
}
