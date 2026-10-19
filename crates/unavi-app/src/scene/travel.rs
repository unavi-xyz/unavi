use bevy::prelude::*;
use bevy_hsd::document::{
    HsdDocId,
    Minted,
};
use unavi_space::{
    grid::ActiveSpace,
    membership::{
        Space,
        SpaceId,
    },
    portal::ByPortal,
    travel::PendingTravel,
};

use crate::{
    camera::veil::Veil,
    scene::{
        SceneState,
        home::{
            HomeSpace,
            remint_home,
        },
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
/// is hidden. From limbo, nothing is standing, so the request retargets the
/// arrival, and every space limbo is not waiting on is dropped: a target
/// turned away from, a space fallen out of, a home that resolved late. Portal
/// spaces are the portal's to keep.
///
/// Travelling to the space already stood in drops and re-reads its entity,
/// which is the only way to ask for a space again after a script or peer has
/// left it in a state worth abandoning.
pub fn drive_travel(
    mut pending: ResMut<PendingTravel>,
    mut departing: ResMut<Departing>,
    time: Res<Time>,
    state: Res<State<SceneState>>,
    active: Res<ActiveSpace>,
    spaces: Query<(Entity, &Space, Has<ByPortal>)>,
    unminted: Query<(Entity, &Space), (With<HsdDocId>, Without<Minted>)>,
    home: Option<Res<HomeSpace>>,
    mut arrival: ResMut<LimboArrival>,
    mut veil: ResMut<Veil>,
    mut next: ResMut<NextState<SceneState>>,
    mut commands: Commands,
) {
    remint_home(home.as_deref(), &unminted, &mut commands);
    let requested = pending.0.take();

    match state.get() {
        SceneState::Limbo => {
            // A departure cut short by a fall back to limbo still means to
            // go where it was going.
            let unfinished = departing.0.take();
            if let Some(target) = requested.or(unfinished) {
                arrival.retarget(target, time.elapsed());
                if !spaces.iter().any(|(_, s, _)| s.0 == target) {
                    commands.spawn(Space(target));
                }
            }
            let Some(target) = arrival.target else {
                return;
            };
            for (entity, space, by_portal) in &spaces {
                if space.0 != target && !by_portal {
                    commands.entity(entity).despawn();
                }
            }
        }
        SceneState::Space => {
            if requested.is_some() {
                departing.0 = requested;
            }
            let Some(target) = departing.0 else {
                return;
            };
            // Held every tick, since anything revealing the veil meanwhile
            // would otherwise stall the departure.
            veil.cover();
            if !veil.is_covered() {
                return;
            }
            departing.0 = None;

            let active_space = active.0.and_then(|e| spaces.get(e).ok());
            let reloading = active_space.map(|(_, s, _)| s.0) == Some(target);

            if let Some((active_ent, ..)) = active_space {
                commands.entity(active_ent).despawn();
            }
            if reloading || !spaces.iter().any(|(_, s, _)| s.0 == target) {
                commands.spawn(Space(target));
            }
            arrival.retarget(target, time.elapsed());
            next.set(SceneState::Limbo);
        }
    }
}
