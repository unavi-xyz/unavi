//! VRM player controller for desktop and VR.
//!
//! # Player Scaling Model
//!
//! The player system maintains a separation between the player's real-world height
//! and their VRM avatar's visual height:
//!
//! - **`real_height`**: Player's capsule height from head to floor (default 1.7m, or VR headset height)
//!   - This is the player's physics collider size
//!   - Camera is positioned based on this height
//!   - Never changes based on avatar
//!
//! - **`vrm_height`**: VRM model's height measured from eye bones
//!   - Calculated when VRM loads by measuring bone positions
//!   - May be larger or smaller than `real_height`
//!
//! - **`WorldScale`**: Scales the entire world to match player perception
//!   - Formula: `real_height / vrm_height`
//!   - If VRM is taller than `real_height` → world shrinks → player feels taller
//!   - If VRM is shorter than `real_height` → world grows → player feels shorter
//!   - Applied to world objects, NOT the avatar itself
//!
//! The VRM avatar is positioned (not scaled) so that:
//! - VRM feet align with capsule bottom
//! - VRM eyes align with the player's camera/head position

use bevy::{post_process::auto_exposure::AutoExposurePlugin, prelude::*};
use bevy_tnua::prelude::TnuaControllerPlugin;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;
use bevy_vrm::VrmPlugins;
use unavi_input::CursorGrabState;

pub mod animation;
pub mod avatar_spawner;
mod bones;
pub mod config;
mod eye_offset;
mod input;
mod spawner;
pub mod tracking;

pub use avatar_spawner::AvatarSpawner;
pub use bones::AvatarBones;
pub use config::{PlayerConfig, WorldScale};
pub use spawner::LocalPlayerSpawner;
pub use tracking::{TrackedHand, TrackedHead, TrackedPose, TrackingSource};

/// Main player plugin.
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AutoExposurePlugin,
            TnuaControllerPlugin::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
            VrmPlugins,
        ))
        .init_state::<CursorGrabState>()
        .add_systems(
            Update,
            (
                eye_offset::setup_vrm_eye_offset,
                animation::init_animation_players,
                bones::populate_avatar_bones,
                input::apply_head_input.run_if(in_state(CursorGrabState::Locked)),
                input::apply_body_input,
                tracking::sync_tracked_pose_to_transform,
                bones::apply_head_tracking,
            )
                .chain(),
        )
        .add_systems(
            FixedUpdate,
            (
                animation::load::load_animation_nodes,
                animation::velocity::calc_average_velocity,
                animation::weights::play_avatar_animations.run_if(animation::is_desktop_mode),
            ),
        );
    }
}

/// References to player entity children for efficient lookups.
#[derive(Component)]
pub struct PlayerEntities {
    pub avatar: Entity,
    pub camera: Entity,
    pub body: Entity,
    pub tracked_head: Entity,
}

/// Marker for the local player entity.
#[derive(Component, Default)]
#[require(Transform, GlobalTransform, Visibility)]
pub struct LocalPlayer;

/// Marker for the physics rig entity.
#[derive(Component, Default)]
#[require(Transform, GlobalTransform, Visibility)]
pub struct PlayerRig;

/// Marker for the player's avatar entity.
#[derive(Component, Default)]
#[require(Transform, GlobalTransform, Visibility)]
pub struct PlayerAvatar;

/// Marker for the player's camera entity.
#[derive(Component, Default)]
#[require(Camera3d, Transform, GlobalTransform, Visibility)]
pub struct PlayerCamera;
