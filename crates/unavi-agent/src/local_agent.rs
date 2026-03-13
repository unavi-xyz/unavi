use avian3d::prelude::*;
use bevy::{camera::visibility::RenderLayers, prelude::*};
use bevy_tnua::{
    builtins::{TnuaBuiltinJumpConfig, TnuaBuiltinWalkConfig},
    prelude::*,
};
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use bevy_vrm::first_person::{DEFAULT_RENDER_LAYERS, FirstPersonFlag};
use unavi_avatar::{
    Avatar, VrmPath,
    animation::{defaults::default_character_animations, velocity::AverageVelocity},
};
use unavi_input::raycast::PrimaryRaycastInput;
use unavi_portal::{PortalTraveler, create::PORTAL_RENDER_LAYER};

use crate::{
    Agent, AgentCamera, AgentRig, ControlScheme, ControlSchemeConfig, Grounded, LocalAgent,
    LocalAgentEntities,
    config::{AgentConfig, XrMode},
    tracking::{TrackedHead, TrackedPose},
};

const RAYCAST_GRAB_DISTANCE: f32 = 3.0;

pub fn on_local_agent_added(
    event: On<Add, LocalAgent>,
    asset_server: Res<AssetServer>,
    xr_mode: Res<XrMode>,
    agent: Query<(&AgentConfig, Option<&VrmPath>)>,
    mut commands: Commands,
) {
    let Ok((config, vrm_path)) = agent.get(event.entity) else {
        return;
    };

    let animations = default_character_animations(&asset_server);
    let camera = spawn_camera(&mut commands, xr_mode.0);

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

    if !xr_mode.0 {
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

    let mut avatar_cmd = commands.spawn(Avatar);
    if let Some(path) = vrm_path {
        avatar_cmd.insert(path.clone());
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
    commands.entity(event.entity).add_child(body).insert((
        Agent,
        LocalAgentEntities {
            avatar,
            camera,
            body,
            tracked_head,
        },
    ));
}

fn spawn_camera(commands: &mut Commands, is_xr: bool) -> Entity {
    let camera = if is_xr {
        commands.spawn_empty().id()
    } else {
        commands.spawn(Camera3d::default()).id()
    };

    commands.entity(camera).insert((
        AgentCamera,
        Projection::Perspective(PerspectiveProjection {
            near: 0.001,
            ..default()
        }),
        Transform::default().looking_at(Vec3::NEG_Z, Vec3::Y),
        RenderLayers::from_layers(&[0, PORTAL_RENDER_LAYER])
            .union(&DEFAULT_RENDER_LAYERS[&FirstPersonFlag::FirstPersonOnly]),
    ));

    camera
}
