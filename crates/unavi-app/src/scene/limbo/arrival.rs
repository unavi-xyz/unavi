//! The gate out of limbo: a space is entered once it is loaded, its scripts
//! have settled, and there is ground to stand on.

use std::time::Duration;

use bevy::prelude::*;
use bevy_hsd::{
    hierarchy::descends_from,
    loaded::HsdLoaded,
};
use unavi_script::ScriptStatus;
use unavi_space::{
    grid::ActiveSpace,
    membership::{
        Space,
        SpaceId,
    },
};

use crate::scene::{
    SceneState,
    respawn::Respawn,
};

/// How long to wait before warning that a space is still not ready. Limbo
/// keeps waiting past it, since entering an unloaded space strands the agent
/// with nothing to stand on.
pub(super) const SPACE_LOAD_TIMEOUT: Duration = Duration::from_secs(30);
/// How long a loaded, settled space may go without ground before the agent is
/// let in anyway, since a space can legitimately have no floor.
pub(super) const GROUND_GRACE: Duration = Duration::from_secs(3);

/// The space limbo is holding the local agent for.
#[derive(Resource, Default)]
pub struct LimboArrival {
    /// Namespace being travelled to. `None` takes whichever space loads first,
    /// which is how the startup entry into home arrives.
    pub target:          Option<SpaceId>,
    timeout:             Duration,
    timed_out:           bool,
    ready:               Option<Arrival>,
    /// The space just entered, pending promotion by `recenter_active_space`.
    /// `active.0` is not set until that runs in `PostUpdate`, so this covers
    /// the gap where the agent is in `SceneState::Space` with no active space.
    pub(super) entering: Option<Entity>,
}

/// A space that is loaded with every script settled, waiting on ground.
struct Arrival {
    space:    Entity,
    /// When the space first held that state.
    since:  Duration,
    /// The spawn point found standing on ground, relative to the space's
    /// origin, which may move before entry.
    ground: Option<Vec3>,
}

impl LimboArrival {
    /// The space the probe should look for ground in, until it finds some.
    pub(super) fn probing_space(&self) -> Option<Entity> {
        self.ready
            .as_ref()
            .filter(|ready| ready.ground.is_none())
            .map(|ready| ready.space)
    }

    pub(super) const fn mark_grounded(&mut self, offset: Vec3) {
        if let Some(ready) = &mut self.ready {
            ready.ground = Some(offset);
        }
    }
}

pub fn arm_limbo_arrival(time: Res<Time>, mut arrival: ResMut<LimboArrival>) {
    arrival.timeout = time.elapsed() + SPACE_LOAD_TIMEOUT;
    arrival.timed_out = false;
    arrival.ready = None;
    arrival.entering = None;
}

/// Drops back to limbo whenever the space underfoot is gone or still loading,
/// so the agent never stands in a space that has no scene to stand on.
pub fn fall_back_to_limbo(
    state: Res<State<SceneState>>,
    active: Res<ActiveSpace>,
    loaded: Query<(), (With<Space>, With<HsdLoaded>)>,
    mut arrival: ResMut<LimboArrival>,
    mut next: ResMut<NextState<SceneState>>,
) {
    if !matches!(state.get(), SceneState::Space) {
        return;
    }
    if active.0.is_some_and(|space| loaded.contains(space)) {
        // The active space settled; the entry that chose it is done.
        arrival.entering = None;
        return;
    }
    // `recenter_active_space` promotes the arrival in `PostUpdate`, after this
    // runs; the agent would otherwise bounce back before the promotion lands.
    if arrival.entering.is_some_and(|space| loaded.contains(space)) {
        return;
    }
    info!("No loaded space, returning to limbo");
    next.set(SceneState::Limbo);
}

/// Leaves limbo once the arrival space is loaded, its scripts have settled,
/// and the probe has found ground, or ground has been missing for
/// [`GROUND_GRACE`].
///
/// Reading [`HsdLoaded`] every tick rather than observing its insertion: a
/// space opened earlier through a portal is already loaded when travel to it
/// begins, so no insertion is coming.
pub fn drive_limbo_exit(
    state: Res<State<SceneState>>,
    time: Res<Time>,
    spaces: Query<(Entity, &Space, Has<HsdLoaded>)>,
    origins: Query<&GlobalTransform, With<Space>>,
    scripts: Query<(Entity, &ScriptStatus)>,
    parents: Query<&ChildOf>,
    mut arrival: ResMut<LimboArrival>,
    mut commands: Commands,
) {
    if !matches!(state.get(), SceneState::Limbo) {
        return;
    }
    let now = time.elapsed();

    if now >= arrival.timeout && !arrival.timed_out {
        arrival.timed_out = true;
        warn!("Space still not ready after the load timeout, continuing to wait");
    }

    let target = arrival.target;
    let candidate = spaces
        .iter()
        .filter(|(_, space, _)| target.is_none_or(|ns| ns == space.0))
        .find_map(|(entity, _, loaded)| loaded.then_some(entity));
    let Some(space) = candidate else {
        arrival.ready = None;
        return;
    };

    // Past the timeout a script still loading no longer holds the agent back,
    // since one that never instantiates would otherwise hold it forever.
    let unsettled = !arrival.timed_out
        && scripts.iter().any(|(script, status)| {
            !status.is_settled() && descends_from(script, space, &parents)
        });
    if unsettled {
        arrival.ready = None;
        return;
    }

    let ready = arrival
        .ready
        .take()
        .filter(|ready| ready.space == space)
        .unwrap_or(Arrival {
            space,
            since: now,
            ground: None,
        });
    let (ground, since) = (ready.ground, ready.since);
    arrival.ready = Some(ready);

    if let Some(offset) = ground {
        info!("Space loaded, exiting limbo");
        commands.trigger(EnterSpace {
            space: Some(space),
            at:    origins
                .get(space)
                .ok()
                .map(|origin| origin.translation() + offset),
        });
    } else if now >= since + GROUND_GRACE {
        warn!("No ground found under any spawn point, exiting limbo anyway");
        commands.trigger(EnterSpace {
            space: Some(space),
            at:    None,
        });
    }
}

#[derive(Event)]
pub struct EnterSpace {
    /// Space to spawn into, overriding the active one so an arrival lands in
    /// what it travelled to rather than whichever space is nearest limbo.
    space: Option<Entity>,
    /// The spawn point the ground probe found ground under.
    at:    Option<Vec3>,
}

pub fn enter_space(
    trigger: On<EnterSpace>,
    mut arrival: ResMut<LimboArrival>,
    mut next: ResMut<NextState<SceneState>>,
    mut commands: Commands,
) {
    arrival.target = None;
    arrival.ready = None;
    arrival.entering = trigger.event().space;
    next.set(SceneState::Space);
    commands.trigger(Respawn {
        space: trigger.event().space,
        at:    trigger.event().at,
    });
}
