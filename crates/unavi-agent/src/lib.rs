//! The local player: a VRM rig driven by desktop or VR input.
//!
//! Split three ways: the rig (body collider, Tnua controller, camera —
//! [`AgentRig`], [`spawn_local_agent`](local_agent::spawn_local_agent)), the
//! avatar it wears (VRM mesh and animation, owned by `unavi-avatar`), and the
//! input that drives both ([`movement`]). `unavi-agent` reads input and
//! physics state; it never reads a script or the network directly.

use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;
use unavi_avatar::Grounded;
use unavi_input::cursor_lock::CursorGrabState;

use crate::config::{
    AgentConfig,
    InputMode,
};

mod bones;
pub mod config;
mod fit;
mod local_agent;
mod movement;

/// Systems that update the local agent's head and body transforms each
/// frame. Consumers that read the resulting camera pose should run after
/// this set.
#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AgentMovementSet;

/// Adds the local agent: its rig, movement, and (on native) VR tracking.
/// Requires [`unavi_avatar::AvatarPlugin`] to be added too, since a local
/// agent spawns an [`unavi_avatar::Avatar`].
pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        let xr_active = |mode: Res<InputMode>| mode.is_xr();

        app.add_plugins((
            #[cfg(not(target_family = "wasm"))]
            bevy::post_process::auto_exposure::AutoExposurePlugin,
            TnuaControllerPlugin::<ControlScheme>::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
        ))
        .init_resource::<InputMode>()
        .add_observer(movement::teleport::handle_agent_teleport)
        .add_observer(local_agent::spawn_local_agent)
        .add_systems(
            Update,
            (
                fit::fit_rig_to_avatar,
                movement::apply_head_input
                    .run_if(in_state(CursorGrabState::Locked).and_then(not(xr_active))),
                bones::apply_head_tracking,
            )
                .chain()
                .in_set(AgentMovementSet),
        )
        .add_systems(
            FixedUpdate,
            (
                movement::apply_body_input.in_set(TnuaUserControlsSystems),
                config::apply_config_to_controller,
                movement::grounded::sync_grounded_state,
            ),
        );

        #[cfg(not(target_family = "wasm"))]
        {
            app.init_resource::<movement::xr::HmdWorldPose>()
                .init_resource::<movement::xr::SnapTurnReady>()
                .add_systems(Startup, movement::xr::spawn_hmd_tracker.run_if(xr_active))
                .add_systems(
                    Update,
                    (
                        movement::xr::update_hmd_world_pose,
                        movement::xr::sync_stage_to_body,
                        movement::xr::apply_xr_turn,
                        movement::xr::update_movement_yaw,
                        movement::xr::update_xr_head_tracking,
                    )
                        .chain()
                        .before(AgentMovementSet)
                        .run_if(xr_active),
                );
        }
    }
}

#[derive(Component)]
pub struct AgentAvatar(pub Entity);

#[derive(Component)]
pub struct AgentCamera(pub Entity);

#[derive(Component, Default)]
#[require(
    Transform,
    Visibility,
    movement::MovementYaw,
    movement::TargetBodyInput,
    movement::TargetHeadInput
)]
pub struct AgentRig;

#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
pub enum ControlScheme {
    Jump(TnuaBuiltinJump),
}

#[derive(Component)]
pub struct LocalAgentEntities {
    pub body:         Entity,
    pub tracked_head: Entity,
}

#[derive(Component, Default)]
#[require(AgentConfig, Transform, Visibility)]
pub struct LocalAgent;

/// The rig's head anchor: the camera's parent, and the bone the user's
/// look/HMD rotation is applied to.
#[derive(Component, Default)]
#[require(Transform)]
pub struct TrackedHead;
