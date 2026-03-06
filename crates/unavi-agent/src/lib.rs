//! VRM agent controller for desktop and VR.
//!
//! # Scaling Model
//!
//! The agent system maintains a separation between real-world height and VRM
//! avatar height:
//!
//! - **`real_height`**: Capsule height from head to floor (default 1.7m, or VR
//!   headset height). Physics collider size. Camera positioned based on this
//!   height. Never changes based on avatar.
//!
//! - **`vrm_height`**: VRM model's height measured from eye bones. Calculated
//!   when VRM loads by measuring bone positions. May be larger or smaller than
//!   `real_height`.
//!
//! - **`WorldScale`**: Scales the entire world to match perception. Formula:
//!   `real_height / vrm_height`. If VRM is taller than `real_height` → world
//!   shrinks → agent feels taller. If VRM is shorter than `real_height` → world
//!   grows → agent feels shorter. Applied to world objects, NOT the avatar
//!   itself.
//!
//! The VRM avatar is positioned (not scaled) so that:
//! - VRM feet align with capsule bottom
//! - VRM eyes align with the camera/head position

use bevy::{post_process::auto_exposure::AutoExposurePlugin, prelude::*};
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;
use unavi_avatar::{AvatarPlugin, Grounded};

mod bones;
pub mod config;
mod eye_offset;
mod grounded;
mod movement;
mod spawner;
pub mod tracking;
#[cfg(not(target_family = "wasm"))]
mod xr_movement;

pub use config::{AgentConfig, WorldScale, XrMode};
pub use tracking::{TrackedHand, TrackedHead, TrackedPose, TrackingSource};
use unavi_input::cursor_lock::CursorGrabState;
#[cfg(not(target_family = "wasm"))]
pub use xr_movement::{HmdWorldPose, TurnMode};

pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            #[cfg(not(target_family = "wasm"))]
            AutoExposurePlugin,
            TnuaControllerPlugin::<ControlScheme>::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
            AvatarPlugin,
        ))
        .init_resource::<config::XrMode>()
        .init_resource::<movement::MenuAnimationState>()
        .init_resource::<movement::MovementYaw>()
        .init_resource::<movement::TargetBodyInput>()
        .init_resource::<movement::TargetHeadInput>()
        .add_observer(movement::handle_agent_teleport)
        .add_observer(spawner::on_local_agent_added);

        #[cfg(not(target_family = "wasm"))]
        {
            app.init_resource::<xr_movement::HmdWorldPose>()
                .init_resource::<xr_movement::TurnMode>()
                .init_resource::<xr_movement::SnapTurnReady>();

            let xr_active = |xr: Res<XrMode>| xr.0;

            app.add_systems(Startup, xr_movement::spawn_hmd_tracker.run_if(xr_active));

            app.add_systems(
                Update,
                (
                    eye_offset::setup_vrm_eye_offset,
                    (
                        xr_movement::update_hmd_world_pose,
                        xr_movement::sync_stage_to_body,
                        xr_movement::apply_xr_turn,
                        xr_movement::update_movement_yaw,
                    )
                        .chain()
                        .run_if(xr_active),
                    movement::apply_head_input.run_if(in_state(CursorGrabState::Locked)),
                    movement::apply_body_input,
                    movement::apply_menu_animation,
                    xr_movement::update_xr_head_tracking.run_if(xr_active),
                    tracking::sync_tracked_pose_to_transform,
                    bones::apply_head_tracking,
                )
                    .chain(),
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
                .chain(),
        );

        app.add_systems(
            FixedUpdate,
            (
                config::apply_config_to_controller,
                grounded::sync_grounded_state,
            ),
        );
    }
}

#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
pub enum ControlScheme {
    Jump(TnuaBuiltinJump),
}

#[derive(Component)]
pub struct AgentEntities {
    pub avatar: Entity,
    pub camera: Entity,
    pub body: Entity,
    pub tracked_head: Entity,
}

#[derive(Component, Default)]
#[require(AgentConfig, TrackingSource, Transform, GlobalTransform, Visibility)]
pub struct LocalAgent;

#[derive(Component, Default)]
#[require(Transform, GlobalTransform, Visibility)]
pub struct AgentRig;

#[derive(Component, Default)]
#[require(Transform, GlobalTransform, Visibility)]
pub struct AgentCamera;
