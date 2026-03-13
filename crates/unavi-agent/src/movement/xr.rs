use std::f32::consts::FRAC_PI_6;

use bevy::{ecs::message::MessageWriter, prelude::*};
use bevy_mod_xr::session::XrTrackingRoot;
use bevy_xr_utils::{
    tracking_utils::XrTrackedView,
    transform_utils::{SnapToPosition, SnapToRotation},
};

use crate::{
    AgentRig, LocalAgentEntities,
    movement::MovementYaw,
    tracking::{TrackedHead, TrackedPose},
};

#[derive(Resource, Default)]
pub struct HmdWorldPose {
    pub translation: Vec3,
    pub rotation: Quat,
    pub yaw: f32,
}

#[derive(Resource)]
pub enum TurnMode {
    Snap {
        angle: f32,
        threshold: f32,
    },
    #[expect(unused, reason = "need config option")]
    Smooth {
        speed: f32,
        threshold: f32,
    },
}

impl Default for TurnMode {
    fn default() -> Self {
        Self::Snap {
            angle: FRAC_PI_6,
            threshold: 0.7,
        }
    }
}

/// Latch: stick must return to center before next snap.
#[derive(Resource, Default)]
pub struct SnapTurnReady(pub bool);

#[derive(Component)]
pub struct HmdTracker;

pub fn spawn_hmd_tracker(mut commands: Commands) {
    commands.spawn((HmdTracker, XrTrackedView, Transform::default()));
}

/// Reads HMD root-local transform + [`XrTrackingRoot`] global
/// transform to compute world-space HMD pose.
pub fn update_hmd_world_pose(
    hmd: Query<&Transform, With<HmdTracker>>,
    root: Query<&GlobalTransform, With<XrTrackingRoot>>,
    mut pose: ResMut<HmdWorldPose>,
) {
    let (Ok(hmd_local), Ok(root_gt)) = (hmd.single(), root.single()) else {
        return;
    };

    let world = root_gt.mul_transform(*hmd_local);
    pose.translation = world.translation();
    pose.rotation = world.rotation();

    let (yaw, _, _) = world.rotation().to_euler(EulerRot::YXZ);
    pose.yaw = yaw;
}

/// Snaps XR tracking root position to match the Tnua body,
/// keeping the stage aligned with physics.
pub fn sync_stage_to_body(
    agents: Query<&LocalAgentEntities>,
    rigs: Query<&Transform, With<AgentRig>>,
    mut pos_writer: MessageWriter<SnapToPosition>,
) {
    for entities in agents.iter() {
        let Ok(body) = rigs.get(entities.body) else {
            continue;
        };

        pos_writer.write(SnapToPosition(Vec3::new(
            body.translation.x,
            0.0,
            body.translation.z,
        )));
    }
}

/// Applies snap or smooth turning from right stick input.
pub fn apply_xr_turn(
    look_action: Query<
        &unavi_input::schminput::Vec2ActionValue,
        With<unavi_input::actions::LookAction>,
    >,
    turn_mode: Res<TurnMode>,
    mut snap_ready: ResMut<SnapTurnReady>,
    pose: Res<HmdWorldPose>,
    time: Res<Time>,
    mut rot_writer: MessageWriter<SnapToRotation>,
) {
    let Ok(action) = look_action.single() else {
        return;
    };

    let x = action.any.x;

    match *turn_mode {
        TurnMode::Snap { angle, threshold } => {
            if x.abs() < 0.2 {
                snap_ready.0 = true;
            }

            if snap_ready.0 && x.abs() > threshold {
                let sign = -x.signum();
                let target_yaw = sign.mul_add(angle, pose.yaw);
                rot_writer.write(SnapToRotation(Quat::from_rotation_y(target_yaw)));
                snap_ready.0 = false;
            }
        }
        TurnMode::Smooth { speed, threshold } => {
            if x.abs() > threshold {
                let target_yaw = (x * speed).mul_add(time.delta_secs(), pose.yaw);
                rot_writer.write(SnapToRotation(Quat::from_rotation_y(target_yaw)));
            }
        }
    }
}

/// Sets [`MovementYaw`] from HMD yaw for thumbstick-relative movement.
pub fn update_movement_yaw(pose: Res<HmdWorldPose>, mut yaw: ResMut<MovementYaw>) {
    yaw.0 = pose.yaw;
}

/// Syncs tracked head rotation to HMD world rotation so the
/// avatar's head bone follows the headset.
pub fn update_xr_head_tracking(
    agents: Query<&LocalAgentEntities>,
    mut tracked_heads: Query<&mut TrackedPose, With<TrackedHead>>,
    pose: Res<HmdWorldPose>,
) {
    for entities in agents.iter() {
        if let Ok(mut head_pose) = tracked_heads.get_mut(entities.tracked_head) {
            head_pose.rotation = pose.rotation;
        }
    }
}
