use bevy::prelude::*;
use unavi_portal::crossing::Crossed;

use crate::movement::{
    TargetBodyInput,
    TargetHeadInput,
};

/// Reorients the local agent's look and input intent across a portal
/// crossing; physical momentum is carried by `unavi_portal`'s
/// `carry_momentum`.
pub fn handle_agent_teleport(
    event: On<Crossed>,
    mut rigs: Query<(&mut TargetBodyInput, &mut TargetHeadInput)>,
) {
    let Ok((mut target_body, mut target_head)) = rigs.get_mut(event.entity) else {
        return;
    };

    let delta_yaw = event.transition_rotation.to_euler(EulerRot::YXZ).0;

    // Body world yaw is `-target_head.x`, so subtracting rotates the heading
    // by `+delta_yaw`.
    target_head.0.x -= delta_yaw;

    target_body.0 = target_body.rotate_y(delta_yaw);
}
