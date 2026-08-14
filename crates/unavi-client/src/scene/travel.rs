use bevy::prelude::*;
use unavi_space::{
    Space,
    anchor::ActiveSpace,
    travel::PendingTravel,
};

use crate::scene::{
    SceneState,
    limbo::LimboArrival,
};

/// Emergency travel: unloads the current space and re-enters limbo targeting
/// `target`, so arrival runs through the same load-gated respawn (and thus
/// spawn-point) path as initial space entry. Other spaces are left untouched.
///
/// Travelling to the space already stood in reloads it: the old entity is
/// dropped and read back fresh, which is the only way to ask for a space again
/// after a script or a peer has left it in a state worth abandoning.
pub fn drive_travel(
    mut pending: ResMut<PendingTravel>,
    state: Res<State<SceneState>>,
    active: Res<ActiveSpace>,
    spaces: Query<(Entity, &Space)>,
    mut arrival: ResMut<LimboArrival>,
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
    let reloading = active_space.map(|(_, s)| s.0) == Some(target);

    if let Some((active_ent, _)) = active_space {
        commands.entity(active_ent).despawn();
    }
    if reloading || !spaces.iter().any(|(_, s)| s.0 == target) {
        commands.spawn(Space(target));
    }
    arrival.target = Some(target);
    next.set(SceneState::Limbo);
}
