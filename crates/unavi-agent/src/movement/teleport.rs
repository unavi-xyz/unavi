use bevy::prelude::*;
use unavi_manifold::transition::CrossedSeam;

use crate::{
    AgentRig,
    movement::{
        TargetBodyInput,
        TargetHeadInput,
    },
};

/// Reorients the local agent's look and input intent across a chart
/// transition; physical momentum is carried by `unavi_portal`'s
/// `carry_momentum`.
pub fn handle_agent_teleport(
    event: On<CrossedSeam>,
    mut target_body: ResMut<TargetBodyInput>,
    mut target_head: ResMut<TargetHeadInput>,
    agents: Query<(), With<AgentRig>>,
) {
    if !agents.contains(event.entity) {
        return;
    }

    let delta_yaw = event.transition_rotation.to_euler(EulerRot::YXZ).0;

    // Body world yaw is `-target_head.x`, so subtracting rotates the heading
    // by `+delta_yaw`.
    target_head.0.x -= delta_yaw;

    target_body.0 = target_body.rotate_y(delta_yaw);
}
