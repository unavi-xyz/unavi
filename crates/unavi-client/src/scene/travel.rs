use bevy::prelude::*;
use unavi_space::{
    Space,
    anchor::ActiveSpace,
    travel::PendingTravel,
};

use crate::scene::SceneState;

/// Emergency travel: unloads the current space and re-enters limbo targeting
/// `target`, so arrival runs through the same load-gated respawn (and thus
/// spawn-point) path as initial space entry. Other spaces are left untouched.
pub fn drive_travel(
    mut pending: ResMut<PendingTravel>,
    state: Res<State<SceneState>>,
    active: Res<ActiveSpace>,
    spaces: Query<(Entity, &Space)>,
    mut next: ResMut<NextState<SceneState>>,
    mut commands: Commands,
) {
    if !matches!(state.get(), SceneState::Space) {
        return;
    }
    let Some(target) = pending.0 else {
        return;
    };
    pending.0 = None;

    let active_space = active.0.and_then(|e| spaces.get(e).ok());
    if active_space.map(|(_, s)| s.0) == Some(target) {
        return;
    }

    if let Some((active_ent, _)) = active_space {
        commands.entity(active_ent).despawn();
    }
    if !spaces.iter().any(|(_, s)| s.0 == target) {
        commands.spawn(Space(target));
    }
    next.set(SceneState::Limbo);
}
