use avian3d::prelude::*;
use bevy::{
    camera::visibility::RenderLayers,
    prelude::*,
};
use bevy_tnua::{
    builtins::{
        TnuaBuiltinJumpConfig,
        TnuaBuiltinWalkConfig,
    },
    prelude::*,
};
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use bevy_vrm::first_person::{
    DEFAULT_RENDER_LAYERS,
    FirstPersonFlag,
};
use unavi_avatar::{
    Avatar,
    VrmPath,
    animation::{
        defaults::default_character_animations,
        velocity::AverageVelocity,
    },
};
use unavi_input::pointer::{
    PointerAnchor,
    PointerKind,
    backend::PointerFilter,
};
use unavi_portal::{
    body::{
        PortalBody,
        PortalViewer,
    },
    render_layers::PORTAL_RENDER_LAYER,
};

use crate::{
    AgentAvatar,
    AgentCamera,
    AgentRig,
    ControlScheme,
    ControlSchemeConfig,
    Grounded,
    LocalAgent,
    LocalAgentEntities,
    TrackedHead,
    config::{
        AgentConfig,
        InputMode,
    },
};

const CAMERA_NEAR_PLANE: f32 = 0.01;
/// Max ground slope Tnua's walk basis will still treat as "grounded".
const MAX_WALK_SLOPE_DEGREES: f32 = 55.0;
/// The initial camera/head height is set a little below the rig's full
/// height, since the avatar's own eye position has not been measured yet
/// (see `fit::fit_rig_to_avatar`); it is corrected once the VRM loads.
const INITIAL_EYE_DROP: f32 = 0.1;

pub fn spawn_local_agent(
    trigger: On<Add, LocalAgent>,
    asset_server: Res<AssetServer>,
    input_mode: Res<InputMode>,
    agent: Query<(&AgentConfig, Option<&VrmPath>)>,
    mut commands: Commands,
) {
    let Ok((config, vrm_path)) = agent.get(trigger.entity) else {
        warn_once!("No agent config");
        return;
    };

    let animations = default_character_animations(&asset_server);
    let camera = spawn_camera(&mut commands, input_mode.is_xr());

    let (body_collider, sensor_shape) = config.rig_shapes();

    let body = commands
        .spawn((
            AgentRig,
            Grounded(true),
            Pickable::IGNORE,
            RigidBody::Dynamic,
            body_collider,
            TnuaController::<ControlScheme>::default(),
            TnuaConfig::<ControlScheme>(asset_server.add(ControlSchemeConfig {
                basis: TnuaBuiltinWalkConfig {
                    float_height: config.float_height(),
                    max_slope: MAX_WALK_SLOPE_DEGREES.to_radians(),
                    ..Default::default()
                },
                jump:  TnuaBuiltinJumpConfig {
                    height: config.jump_height,
                    ..Default::default()
                },
            })),
            TnuaAvian3dSensorShape(sensor_shape),
            LockedAxes::ROTATION_LOCKED,
            Transform::from_xyz(0.0, config.rig_height() / 2.0, 0.0),
            PortalBody,
        ))
        .id();

    let initial_eye_y = config.rig_height() / 2.0 - INITIAL_EYE_DROP;
    let tracked_head = commands
        .spawn((TrackedHead, Transform::from_xyz(0.0, initial_eye_y, 0.0)))
        .add_child(camera)
        .id();

    // The agent's own body sits around the ray's origin, so every pointer
    // would hit it first.
    commands.insert_resource(PointerFilter(
        SpatialQueryFilter::default().with_excluded_entities([body]),
    ));

    if input_mode.is_xr() {
        spawn_hand_pointers(&mut commands);
    } else {
        commands
            .entity(tracked_head)
            .insert(PointerAnchor(PointerKind::Screen));
    }

    let mut avatar_cmd = commands.spawn(Avatar);
    if let Some(path) = vrm_path {
        avatar_cmd.insert(path.clone());
    }
    let avatar = avatar_cmd.id();

    commands.entity(avatar).insert((
        AverageVelocity::new(body),
        animations,
        Transform::from_xyz(0.0, -config.rig_height() / 2.0, 0.0),
    ));

    commands.entity(body).add_children(&[avatar, tracked_head]);
    commands
        .entity(trigger.entity)
        .insert((
            AgentAvatar(avatar),
            AgentCamera(camera),
            LocalAgentEntities { body, tracked_head },
        ))
        .add_child(body);
}

/// `XrTracker` parents each hand under the tracking root, so its transform is
/// the world pose the pointer's ray is cast from.
#[cfg(not(target_family = "wasm"))]
fn spawn_hand_pointers(commands: &mut Commands) {
    use bevy_mod_xr::session::XrTracker;
    use bevy_xr_utils::tracking_utils::{
        XrTrackedLeftGrip,
        XrTrackedRightGrip,
    };

    commands.spawn((
        PointerAnchor(PointerKind::LeftHand),
        XrTrackedLeftGrip,
        XrTracker,
    ));
    commands.spawn((
        PointerAnchor(PointerKind::RightHand),
        XrTrackedRightGrip,
        XrTracker,
    ));
}

#[cfg(target_family = "wasm")]
const fn spawn_hand_pointers(_commands: &mut Commands) {}

fn spawn_camera(commands: &mut Commands, is_xr: bool) -> Entity {
    let camera = if is_xr {
        commands.spawn_empty().id()
    } else {
        commands.spawn(Camera3d::default()).id()
    };

    commands.entity(camera).insert((
        Projection::Perspective(PerspectiveProjection {
            near: CAMERA_NEAR_PLANE,
            ..default()
        }),
        Transform::default().looking_at(Vec3::NEG_Z, Vec3::Y),
        RenderLayers::from_layers(&[0, PORTAL_RENDER_LAYER])
            .union(&DEFAULT_RENDER_LAYERS[&FirstPersonFlag::FirstPersonOnly]),
        PortalViewer,
    ));

    camera
}
