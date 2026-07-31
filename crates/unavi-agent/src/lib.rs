//! VRM agent controller for desktop and VR.

use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;
use unavi_avatar::Grounded;
use unavi_input::cursor_lock::CursorGrabState;
use xdid::core::did::Did;

use crate::{
    config::AgentConfig,
    tracking::TrackingSource,
};

mod bones;
pub mod config;
mod eye_offset;
mod local_agent;
mod movement;
pub mod tracking;

/// Systems that update the local agent's head and body transforms each frame.
/// Consumers that read the resulting camera pose should run after this set.
#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AgentMovementSet;

pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            #[cfg(not(target_family = "wasm"))]
            bevy::post_process::auto_exposure::AutoExposurePlugin,
            TnuaControllerPlugin::<ControlScheme>::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
        ))
        .init_resource::<config::XrMode>()
        .init_resource::<movement::MovementYaw>()
        .init_resource::<movement::TargetBodyInput>()
        .init_resource::<movement::TargetHeadInput>()
        .add_observer(movement::teleport::handle_agent_teleport)
        .add_observer(local_agent::spawn_local_agent);

        #[cfg(not(target_family = "wasm"))]
        {
            use crate::config::XrMode;

            app.init_resource::<movement::xr::HmdWorldPose>()
                .init_resource::<movement::xr::TurnMode>()
                .init_resource::<movement::xr::SnapTurnReady>();

            let xr_active = |xr: Res<XrMode>| xr.0;

            app.add_systems(Startup, movement::xr::spawn_hmd_tracker.run_if(xr_active));

            app.add_systems(
                Update,
                (
                    eye_offset::setup_vrm_eye_offset,
                    (
                        movement::xr::update_hmd_world_pose,
                        movement::xr::sync_stage_to_body,
                        movement::xr::apply_xr_turn,
                        movement::xr::update_movement_yaw,
                    )
                        .chain()
                        .run_if(xr_active),
                    movement::apply_head_input.run_if(in_state(CursorGrabState::Locked)),
                    movement::apply_body_input,
                    movement::xr::update_xr_head_tracking.run_if(xr_active),
                    tracking::sync_tracked_pose_to_transform,
                    bones::apply_head_tracking,
                )
                    .chain()
                    .in_set(AgentMovementSet),
            );
        }

        #[cfg(target_family = "wasm")]
        app.add_systems(
            Update,
            (
                eye_offset::setup_vrm_eye_offset,
                movement::apply_head_input.run_if(in_state(CursorGrabState::Locked)),
                movement::apply_body_input,
                tracking::sync_tracked_pose_to_transform,
                bones::apply_head_tracking,
            )
                .chain()
                .in_set(AgentMovementSet),
        );

        app.add_systems(
            FixedUpdate,
            (
                config::apply_config_to_controller,
                movement::grounded::sync_grounded_state,
            ),
        );
    }
}

#[derive(Component)]
pub struct Agent;

#[derive(Component)]
pub struct AgentAvatar(pub Entity);

#[derive(Component)]
pub struct AgentCamera(pub Entity);

#[derive(Component)]
pub struct AgentDid(pub Did);

#[derive(Component, Default)]
#[require(Transform, Visibility)]
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
#[require(AgentConfig, TrackingSource, Transform, Visibility)]
pub struct LocalAgent;
