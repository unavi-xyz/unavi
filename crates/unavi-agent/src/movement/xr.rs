use bevy::{
    ecs::message::MessageWriter,
    prelude::*,
};
use bevy_mod_xr::session::XrTrackingRoot;
use bevy_xr_utils::{
    tracking_utils::XrTrackedView,
    transform_utils::{
        SnapToPosition,
        SnapToRotation,
    },
};
use unavi_input::{
    action::{
        Action,
        ActionState,
    },
    config::InputConfig,
};

use crate::{
    AgentRig,
    LocalAgentEntities,
    movement::MovementYaw,
    tracking::{
        TrackedHead,
        TrackedPose,
    },
};

#[derive(Resource, Default)]
pub struct HmdWorldPose {
    pub translation: Vec3,
    pub rotation:    Quat,
    pub yaw:         f32,
}

/// Latch: stick must return to center before next snap.
#[derive(Resource, Default)]
pub struct SnapTurnReady(pub bool);

#[derive(Component)]
pub struct HmdTracker;

pub fn spawn_hmd_tracker(mut commands: Commands) {
    commands.spawn((HmdTracker, XrTrackedView, Transform::default()));
}

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

    let (yaw, ..) = world.rotation().to_euler(EulerRot::YXZ);
    pose.yaw = yaw;
}

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

/// Stick travel below this re-arms the snap latch.
const SNAP_RECENTRE: f32 = 0.2;

pub fn apply_xr_turn(
    input: Res<ActionState>,
    config: Res<InputConfig>,
    mut snap_ready: ResMut<SnapTurnReady>,
    pose: Res<HmdWorldPose>,
    time: Res<Time>,
    mut rot_writer: MessageWriter<SnapToRotation>,
) {
    let tuning = &config.tuning;
    let x = input.axis(Action::Look).x;

    if tuning.smooth_turn {
        if x.abs() > tuning.turn_threshold {
            let speed = tuning.smooth_turn_degrees_per_second.to_radians();
            let target_yaw = (x * speed).mul_add(time.delta_secs(), pose.yaw);
            rot_writer.write(SnapToRotation(Quat::from_rotation_y(target_yaw)));
        }
        return;
    }

    if x.abs() < SNAP_RECENTRE {
        snap_ready.0 = true;
    }

    if snap_ready.0 && x.abs() > tuning.turn_threshold {
        let sign = -x.signum();
        let target_yaw = sign.mul_add(tuning.snap_turn_degrees.to_radians(), pose.yaw);
        rot_writer.write(SnapToRotation(Quat::from_rotation_y(target_yaw)));
        snap_ready.0 = false;
    }
}

/// HMD yaw, the reference direction for thumbstick-relative movement.
pub fn update_movement_yaw(pose: Res<HmdWorldPose>, mut yaw: ResMut<MovementYaw>) {
    yaw.0 = pose.yaw;
}

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
