use std::f32::consts::FRAC_PI_2;

use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;
use unavi_portal::teleport::PortalTeleport;

use crate::{
    AgentRig,
    movement::{TargetBodyInput, TargetHeadInput},
};

pub fn handle_agent_teleport(
    event: On<PortalTeleport>,
    mut target_body: ResMut<TargetBodyInput>,
    mut target_head: ResMut<TargetHeadInput>,
    mut agents: Query<&mut LinearVelocity, With<AgentRig>>,
) {
    let Ok(mut velocity) = agents.get_mut(event.entity) else {
        return;
    };

    // Update target head input.
    let (mut yaw, pitch, _) = event.delta_rotation.to_euler(EulerRot::YXZ);

    if yaw.is_sign_negative() {
        yaw -= FRAC_PI_2;
    } else {
        yaw += FRAC_PI_2;
    }

    target_head.0.x += yaw;
    target_head.0.y += pitch;

    // Update target body input and velocity.
    target_body.0 = target_body.rotate_y(-yaw);
    velocity.0 = velocity.rotate_y(-yaw);
}
