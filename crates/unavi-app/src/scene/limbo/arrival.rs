//! The gate out of limbo: a space is entered once it is loaded, its scripts
//! have settled, and there is ground to stand on.

use std::time::Duration;

use avian3d::prelude::Collider;
use bevy::prelude::*;
use bevy_hsd::{
    hierarchy::descends_from,
    loaded::HsdLoaded,
    prim::Prim,
};
use unavi_script::ScriptStatus;
use unavi_space::{
    grid::ActiveSpace,
    membership::{
        Space,
        SpaceId,
    },
};

use crate::{
    camera::veil::Veil,
    scene::{
        SceneState,
        respawn::Respawn,
    },
};

/// How long to wait before warning that a space is still not ready. Limbo
/// keeps waiting past it, since entering an unloaded space strands the agent
/// with nothing to stand on.
pub(super) const SPACE_LOAD_TIMEOUT: Duration = Duration::from_secs(30);
/// How long a loaded, settled space may go without ground before the agent is
/// let in anyway, since a space can legitimately have no floor.
pub(super) const GROUND_GRACE: Duration = Duration::from_secs(3);
/// The longest a loaded, settled space is waited on for ground, however much
/// content keeps arriving and restarting [`GROUND_GRACE`].
pub(super) const GROUND_MAX_WAIT: Duration = Duration::from_secs(15);
/// The shortest time limbo lasts once entered, so a space that is ready at
/// once does not flash limbo by in a few frames.
pub(super) const MIN_LIMBO_STAY: Duration = Duration::from_secs(3);

/// The space limbo is holding the local agent for.
#[derive(Resource, Default)]
pub struct LimboArrival {
    /// Namespace being travelled to. `None` takes whichever space loads first,
    /// which is how the startup entry into home arrives.
    pub target:          Option<SpaceId>,
    timeout:             Duration,
    timed_out:           bool,
    /// When limbo was entered, for [`MIN_LIMBO_STAY`].
    entered_at:          Duration,
    ready:               Option<Arrival>,
    /// The space and time scripts first settled in, kept while they flip back
    /// to loading, so [`GROUND_MAX_WAIT`] still runs out.
    first_settled:       Option<(Entity, Duration)>,
    /// Whether this gate has the veil closing on the way out, so only it
    /// lifts the veil again if the space stops holding.
    covering:            bool,
    /// The space just entered, pending promotion by `recenter_active_space`.
    /// `active.0` is not set until that runs in `PostUpdate`, so this covers
    /// the gap where the agent is in `SceneState::Space` with no active space.
    pub(super) entering: Option<Entity>,
}

/// A space that is loaded with every script settled, waiting on ground.
struct Arrival {
    space:  Entity,
    /// When the space first held that state.
    first:  Duration,
    /// The last time it held that state or gained content, which the grace
    /// runs from.
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

    /// Waits for `target` instead, dropping whatever was found for the last
    /// and restarting the load timeout.
    pub fn retarget(&mut self, target: SpaceId, now: Duration) {
        self.target = Some(target);
        self.ready = None;
        self.first_settled = None;
        self.timeout = now + SPACE_LOAD_TIMEOUT;
        self.timed_out = false;
    }

    const fn release(&mut self, veil: &mut Veil) {
        if self.covering {
            self.covering = false;
            veil.reveal();
        }
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
    arrival.entered_at = time.elapsed();
    arrival.ready = None;
    arrival.first_settled = None;
    arrival.entering = None;
    arrival.covering = false;
}

/// Drops back to limbo whenever the space underfoot is gone or still loading,
/// so the agent never stands in a space that has no scene to stand on.
pub fn fall_back_to_limbo(
    state: Res<State<SceneState>>,
    active: Res<ActiveSpace>,
    loaded: Query<(), (With<Space>, With<HsdLoaded>)>,
    mut arrival: ResMut<LimboArrival>,
    mut veil: ResMut<Veil>,
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
    veil.snap_covered();
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
    content: Query<Entity, Or<(Added<Prim>, Added<Collider>)>>,
    parents: Query<&ChildOf>,
    mut arrival: ResMut<LimboArrival>,
    mut veil: ResMut<Veil>,
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
        arrival.release(&mut veil);
        return;
    };

    // Past the timeout a script still loading no longer holds the agent back,
    // since one that never instantiates would otherwise hold it forever.
    let unsettled = !arrival.timed_out
        && scripts
            .iter()
            .any(|(script, status)| !status.is_settled() && descends_from(script, space, &parents));
    if unsettled {
        arrival.ready = None;
        arrival.release(&mut veil);
        return;
    }

    let first = arrival
        .first_settled
        .filter(|(settled, _)| *settled == space)
        .map_or(now, |(_, at)| at);
    arrival.first_settled = Some((space, first));
    let mut ready = arrival
        .ready
        .take()
        .filter(|ready| ready.space == space)
        .unwrap_or(Arrival {
            space,
            first,
            since: now,
            ground: None,
        });
    // Content still landing under the space may be the ground itself, so the
    // grace runs from the last of it.
    let arriving = content
        .iter()
        .any(|entity| descends_from(entity, space, &parents));
    if arriving && ready.ground.is_none() {
        ready.since = now;
    }
    let (ground, since, first) = (ready.ground, ready.since, ready.first);
    arrival.ready = Some(ready);

    let (passed, at) = ground.map_or_else(
        || {
            (
                now >= since + GROUND_GRACE || now >= first + GROUND_MAX_WAIT,
                None,
            )
        },
        |offset| {
            let at = origins
                .get(space)
                .ok()
                .map(|origin| origin.translation() + offset);
            (true, at)
        },
    );
    if !passed {
        arrival.release(&mut veil);
        return;
    }
    if now < arrival.entered_at + MIN_LIMBO_STAY {
        return;
    }

    // The agent is let in only once the world is hidden, so the move into the
    // space happens behind the fog.
    if !arrival.covering {
        arrival.covering = true;
        if ground.is_some() {
            info!("Space loaded, exiting limbo");
        } else {
            warn!("No ground found under any spawn point, exiting limbo anyway");
        }
    }
    veil.cover();
    if veil.is_covered() {
        commands.trigger(EnterSpace {
            space: Some(space),
            at,
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
    arrival.covering = false;
    arrival.entering = trigger.event().space;
    next.set(SceneState::Space);
    commands.trigger(Respawn {
        space: trigger.event().space,
        at:    trigger.event().at,
    });
}
