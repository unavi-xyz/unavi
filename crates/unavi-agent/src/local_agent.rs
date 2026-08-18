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
use unavi_manifold::{
    ManifoldBody,
    ManifoldViewer,
    visuals::SEAM_RENDER_LAYER,
};

use crate::{
    Agent,
    AgentAvatar,
    AgentCamera,
    AgentRig,
    ControlScheme,
    ControlSchemeConfig,
    Grounded,
    LocalAgent,
    LocalAgentEntities,
    config::{
        AgentConfig,
        XrMode,
    },
    tracking::{
        TrackedHead,
        TrackedPose,
    },
};

const CAMERA_NEAR_PLANE: f32 = 0.01;

pub fn spawn_local_agent(
    trigger: On<Add, LocalAgent>,
    asset_server: Res<AssetServer>,
    xr_mode: Res<XrMode>,
    agent: Query<(&AgentConfig, Option<&VrmPath>)>,
    mut commands: Commands,
) {
    let Ok((config, vrm_path)) = agent.get(trigger.entity) else {
        warn_once!("No agent config");
        return;
    };

    let animations = default_character_animations(&asset_server);
    let camera = spawn_camera(&mut commands, xr_mode.0);

    let body = commands
        .spawn((
            AgentRig,
            Grounded(true),
            Pickable::IGNORE,
            RigidBody::Dynamic,
            Collider::capsule(config.effective_vrm_radius(), config.effective_vrm_height()),
            TnuaController::<ControlScheme>::default(),
            TnuaConfig::<ControlScheme>(asset_server.add(ControlSchemeConfig {
                basis: TnuaBuiltinWalkConfig {
                    float_height: config.float_height(),
                    max_slope: 55.0f32.to_radians(),
                    ..Default::default()
                },
                jump:  TnuaBuiltinJumpConfig {
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
            ManifoldBody,
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

    // The agent's own body sits around the ray's origin, so every pointer
    // would hit it first.
    commands.insert_resource(PointerFilter(
        SpatialQueryFilter::default().with_excluded_entities([body]),
    ));

    if xr_mode.0 {
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
        AverageVelocity {
            target: Some(body),
            ..Default::default()
        },
        animations,
        Transform::from_xyz(0.0, -config.effective_vrm_height() / 2.0, 0.0),
    ));

    commands.entity(body).add_children(&[avatar, tracked_head]);
    commands
        .entity(trigger.entity)
        .insert((
            Agent,
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
        RenderLayers::from_layers(&[0, SEAM_RENDER_LAYER])
            .union(&DEFAULT_RENDER_LAYERS[&FirstPersonFlag::FirstPersonOnly]),
        ManifoldViewer,
    ));

    camera
}
