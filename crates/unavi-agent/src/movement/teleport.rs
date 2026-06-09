use bevy::prelude::*;
use unavi_manifold::transition::CrossedSeam;

use crate::{
    AgentRig,
    movement::{
        TargetBodyInput,
        TargetHeadInput,
    },
};

/// Reorient the local agent's look and input intent across a chart transition.
///
/// Physical momentum is carried by `unavi_portal`'s `carry_momentum`; this only
/// rotates the camera/look target and the world-space input direction so they
/// stay consistent with the agent's new heading after the seam.
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

    // Body world yaw is `-target_head.x`, so subtract to rotate the heading by
    // `+delta_yaw`, matching the new transform from the portal teleport math.
    target_head.0.x -= delta_yaw;

    target_body.0 = target_body.rotate_y(delta_yaw);
}
