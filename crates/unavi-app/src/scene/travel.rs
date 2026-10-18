use bevy::prelude::*;
use unavi_space::{
    grid::ActiveSpace,
    membership::{
        Space,
        SpaceId,
    },
    travel::PendingTravel,
};

use crate::{
    camera::veil::Veil,
    scene::{
        SceneState,
        limbo::arrival::LimboArrival,
    },
};

/// The space being left, while the veil closes over the one still standing.
#[derive(Resource, Default)]
pub struct Departing(Option<SpaceId>);

/// Emergency travel: unloads the current space and re-enters limbo targeting
/// `target`, so arrival runs the same load-gated respawn path as initial
/// space entry.
///
/// From a space, the veil closes first and the space is dropped only once it
/// is hidden. From limbo, nothing is standing to drop, so the request just
/// retargets the arrival.
///
/// Travelling to the space already stood in drops and re-reads its entity,
/// which is the only way to ask for a space again after a script or peer has
/// left it in a state worth abandoning.
pub fn drive_travel(
    mut pending: ResMut<PendingTravel>,
    mut departing: ResMut<Departing>,
    state: Res<State<SceneState>>,
    active: Res<ActiveSpace>,
    spaces: Query<(Entity, &Space)>,
    mut arrival: ResMut<LimboArrival>,
    mut veil: ResMut<Veil>,
    mut next: ResMut<NextState<SceneState>>,
    mut commands: Commands,
) {
    let requested = pending.0.take();

    match state.get() {
        SceneState::Limbo => {
            // A departure cut short by a fall back to limbo still means to
            // go where it was going.
            let unfinished = departing.0.take();
            let Some(target) = requested.or(unfinished) else {
                return;
            };
            arrival.retarget(target);
            if !spaces.iter().any(|(_, s)| s.0 == target) {
                commands.spawn(Space(target));
            }
        }
        SceneState::Space => {
            if requested.is_some() {
                departing.0 = requested;
                veil.cover();
            }
            let Some(target) = departing.0 else {
                return;
            };
            if !veil.is_covered() {
                return;
            }
            departing.0 = None;

            let active_space = active.0.and_then(|e| spaces.get(e).ok());
            let reloading = active_space.map(|(_, s)| s.0) == Some(target);

            if let Some((active_ent, _)) = active_space {
                commands.entity(active_ent).despawn();
            }
            if reloading || !spaces.iter().any(|(_, s)| s.0 == target) {
                commands.spawn(Space(target));
            }
            arrival.retarget(target);
            next.set(SceneState::Limbo);
        }
    }
}
