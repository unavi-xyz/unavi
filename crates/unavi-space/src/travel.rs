use avian3d::prelude::Position;
use bevy::prelude::*;
use blake3::Hash;
use unavi_agent::{
    LocalAgent,
    LocalAgentEntities,
};

use crate::{
    Space,
    anchor::{
        ActiveSpace,
        SPACE_CELL_SIZE,
        SpaceAnchor,
    },
};

/// A queued request to teleport the local agent into `target`, spawning the
/// space if it is not yet loaded. Applied once the space is anchored.
#[derive(Resource, Default)]
pub struct PendingTravel(pub Option<Hash>);

pub fn request_travel(world: &mut World, target: Hash) {
    world.resource_mut::<PendingTravel>().0 = Some(target);
}

/// Teleports the agent body onto the target space's grid cell, letting
/// [`crate::anchor::recenter_active_space`] promote it to the active space.
pub fn apply_pending_travel(
    mut pending: ResMut<PendingTravel>,
    active: Res<ActiveSpace>,
    spaces: Query<(&Space, &SpaceAnchor)>,
    agents: Query<&LocalAgentEntities, With<LocalAgent>>,
    mut bodies: Query<(&mut Transform, &mut Position), Without<Space>>,
    mut commands: Commands,
) {
    let Some(target) = pending.0 else {
        return;
    };

    let Some((_, target_anchor)) = spaces.iter().find(|(s, _)| s.0 == target) else {
        commands.spawn(Space(target));
        return;
    };

    let Some(active_ent) = active.0 else {
        return;
    };
    let Ok((_, active_anchor)) = spaces.get(active_ent) else {
        return;
    };

    let rel = target_anchor.grid_cell - active_anchor.grid_cell;
    let dest = Vec3::new(
        rel.x as f32 * SPACE_CELL_SIZE,
        0.0,
        rel.y as f32 * SPACE_CELL_SIZE,
    );

    let Ok(entities) = agents.single() else {
        return;
    };
    if let Ok((mut transform, mut position)) = bodies.get_mut(entities.body) {
        transform.translation = dest;
        position.0 = dest;
        pending.0 = None;
    }
}
