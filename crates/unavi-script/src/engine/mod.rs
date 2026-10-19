//! Runs script instances: instantiation, and the lifecycle calls each tick.

use std::time::Duration;

use bevy::prelude::*;

use crate::status::ScriptStatus;

pub mod log;

#[cfg(not(target_family = "wasm"))] mod native;
#[cfg(target_family = "wasm")] mod web;

/// How often `fixed-update` runs.
pub const FIXED_INTERVAL: Duration = Duration::from_millis(50);

pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        cfg_select! {
            target_family = "wasm" => {
                app.add_plugins(web::WebEnginePlugin);
            }
            _ => {
                app.add_plugins(native::NativeEnginePlugin);
            }
        }
    }
}

/// Which lifecycle export a call runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TickKind {
    Init,
    Update,
    FixedUpdate,
}

impl TickKind {
    /// The WIT export name, for the log line a trap or a failed init writes.
    #[must_use]
    pub const fn export(self) -> &'static str {
        match self {
            Self::Init => "init",
            Self::Update => "update",
            Self::FixedUpdate => "fixed-update",
        }
    }
}

/// The time a lifecycle call carries.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tick {
    pub dt:    f32,
    pub time:  f64,
    pub index: u64,
}

/// One export's call history.
#[derive(Default)]
struct Clock {
    last:  Option<Duration>,
    index: u64,
}

impl Clock {
    fn next(&mut self, now: Duration) -> Tick {
        let dt = self
            .last
            .map_or(Duration::ZERO, |last| now.saturating_sub(last));
        self.last = Some(now);
        let index = self.index;
        self.index += 1;
        Tick {
            dt: dt.as_secs_f32(),
            time: now.as_secs_f64(),
            index,
        }
    }
}

/// When a script's `update` and `fixed-update` last ran.
#[derive(Component, Default)]
pub struct TickClocks {
    update:       Clock,
    fixed:        Clock,
    /// When the next `fixed-update` is due. A call that runs late pulls the
    /// next one earlier, by up to one interval, so the rate holds on average.
    fixed_due:    Option<Duration>,
    /// Whether a `fixed-update` came due and gave its frame to an `update`.
    /// Only the web engine, which serves one call per frame, has a use for it.
    #[cfg(any(target_family = "wasm", test))]
    fixed_waited: bool,
}

impl TickClocks {
    /// Whether `fixed-update` is due at `now`.
    #[must_use]
    pub fn fixed_due(&self, now: Duration) -> bool {
        self.fixed_due.is_none_or(|due| now >= due)
    }

    /// Advances `kind`'s clock to `now`. `init` carries the time alone.
    pub fn next(&mut self, kind: TickKind, now: Duration) -> Tick {
        match kind {
            TickKind::Init => Tick {
                dt:    0.0,
                time:  now.as_secs_f64(),
                index: 0,
            },
            TickKind::Update => self.update.next(now),
            TickKind::FixedUpdate => {
                let due = self.fixed_due.unwrap_or(now) + FIXED_INTERVAL;
                self.fixed_due = Some(due.max(now));
                self.fixed.next(now)
            }
        }
    }
}

/// Which schedule is deciding a script's next call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Schedule {
    /// Runs `update` once a frame.
    Update,
    /// Runs `init` until the script initializes, then `fixed-update` at
    /// [`FIXED_INTERVAL`].
    Fixed,
}

/// Which lifecycle call, if any, `schedule` should make of a script right
/// now: `None` when the script is trapped, not yet due, or a call for this
/// schedule is already in flight.
///
/// This is native's rule, asked once per schedule: its calls run concurrently
/// on their own lanes. Web serves one call per frame and uses
/// [`next_tick_one_call_per_frame`] instead.
#[must_use]
pub fn next_tick(
    schedule: Schedule,
    status: &ScriptStatus,
    clocks: &TickClocks,
    now: Duration,
) -> Option<TickKind> {
    if status.is_trapped() {
        return None;
    }
    match (schedule, status.is_initialized()) {
        (Schedule::Update, true) => Some(TickKind::Update),
        (Schedule::Fixed, false) => Some(TickKind::Init),
        (Schedule::Fixed, true) if clocks.fixed_due(now) => Some(TickKind::FixedUpdate),
        (Schedule::Update, false) | (Schedule::Fixed, true) => None,
    }
}

/// Which lifecycle call a web frame should make of a script, if any.
///
/// Web serves one call per frame. A call suspended on a host import resumes on
/// the microtask queue after the frame ends, and a second call entered before
/// the first resumes corrupts the instance, so `update` and `fixed-update` can
/// never both have a frame. `update` is the one the eye sees, so a
/// `fixed-update` that comes due waits a frame and takes the next one: above
/// the fixed rate that leaves the update the frames in between, and below it
/// the two alternate rather than the fixed tick taking every frame and
/// stopping the update outright.
#[cfg(any(target_family = "wasm", test))]
#[must_use]
pub fn next_tick_one_call_per_frame(
    status: &ScriptStatus,
    clocks: &mut TickClocks,
    now: Duration,
) -> Option<TickKind> {
    if status.is_trapped() {
        return None;
    }
    if !status.is_initialized() {
        clocks.fixed_waited = false;
        return Some(TickKind::Init);
    }

    let due = clocks.fixed_due(now);
    let kind = if due && clocks.fixed_waited {
        TickKind::FixedUpdate
    } else {
        TickKind::Update
    };
    clocks.fixed_waited = due && matches!(kind, TickKind::Update);
    Some(kind)
}

/// Which in-flight flag a call of `kind` holds. Native serializes calls on
/// the store lock and uses this to skip only a call's own schedule; web holds
/// the whole instance instead, so it has no use for the mapping.
#[cfg(not(target_family = "wasm"))]
#[must_use]
pub const fn lane_for(kind: TickKind) -> crate::status::Lane {
    if matches!(kind, TickKind::Update) {
        crate::status::Lane::Update
    } else {
        crate::status::Lane::Fixed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tick_carries_the_time_since_the_last_of_its_kind() {
        let mut clocks = TickClocks::default();
        let ms = Duration::from_millis;
        assert_eq!(clocks.next(TickKind::Update, ms(100)).dt, 0.0);
        let tick = clocks.next(TickKind::Update, ms(116));
        assert!((tick.dt - 0.016).abs() < 1.0e-6);
        assert_eq!(tick.index, 1);
        assert_eq!(
            clocks.next(TickKind::FixedUpdate, ms(116)).index,
            0,
            "each export counts its own calls"
        );
    }

    #[test]
    fn fixed_update_waits_out_its_interval() {
        let mut clocks = TickClocks::default();
        assert!(clocks.fixed_due(Duration::ZERO));
        let _ = clocks.next(TickKind::FixedUpdate, Duration::from_millis(10));
        assert!(!clocks.fixed_due(Duration::from_millis(40)));
        assert!(clocks.fixed_due(Duration::from_millis(60)));
        let _ = clocks.next(TickKind::FixedUpdate, Duration::from_millis(70));
        assert!(
            clocks.fixed_due(Duration::from_millis(110)),
            "a call 10 ms late brings the next one 10 ms closer"
        );
    }

    #[test]
    fn a_trapped_script_is_never_due() {
        let status = ScriptStatus::default();
        assert!(status.trap());
        let clocks = TickClocks::default();
        assert_eq!(
            next_tick(Schedule::Fixed, &status, &clocks, Duration::ZERO),
            None
        );
    }

    #[test]
    fn fixed_runs_init_until_initialized_then_fixed_update_on_schedule() {
        let status = ScriptStatus::default();
        let clocks = TickClocks::default();
        assert_eq!(
            next_tick(Schedule::Fixed, &status, &clocks, Duration::ZERO),
            Some(TickKind::Init)
        );
        status.set_initialized();
        assert_eq!(
            next_tick(Schedule::Fixed, &status, &clocks, Duration::ZERO),
            Some(TickKind::FixedUpdate)
        );
    }

    #[test]
    fn update_is_only_due_once_initialized() {
        let status = ScriptStatus::default();
        let clocks = TickClocks::default();
        assert_eq!(
            next_tick(Schedule::Update, &status, &clocks, Duration::ZERO),
            None
        );
        status.set_initialized();
        assert_eq!(
            next_tick(Schedule::Update, &status, &clocks, Duration::ZERO),
            Some(TickKind::Update)
        );
    }

    #[test]
    fn a_shared_frame_initializes_before_anything_else() {
        let status = ScriptStatus::default();
        let mut clocks = TickClocks::default();
        assert_eq!(
            next_tick_one_call_per_frame(&status, &mut clocks, Duration::ZERO),
            Some(TickKind::Init)
        );
    }

    /// One call per frame, a second of 60 fps: the fixed tick still runs its 20
    /// times, and the update keeps the frames between them.
    #[test]
    fn a_shared_frame_keeps_both_rates_at_60_fps() {
        let status = ScriptStatus::default();
        status.set_initialized();
        let mut clocks = TickClocks::default();
        let ms = Duration::from_millis;

        let (mut updates, mut fixed) = (0, 0);
        let mut previous = None;
        for frame in 1..=60u64 {
            let now = ms(frame * 1000 / 60);
            let kind = next_tick_one_call_per_frame(&status, &mut clocks, now).expect("owed");
            assert!(
                !(kind == TickKind::FixedUpdate && previous == Some(TickKind::FixedUpdate)),
                "a fixed tick took two frames running"
            );
            previous = Some(kind);
            match kind {
                TickKind::Update => updates += 1,
                TickKind::FixedUpdate => fixed += 1,
                TickKind::Init => unreachable!("the script is initialized"),
            }
            let _ = clocks.next(kind, now);
        }

        assert!(
            (19..=21).contains(&fixed),
            "fixed ran {fixed} times in a second"
        );
        assert!(updates >= 35, "the update kept only {updates} of 60 frames");
    }

    /// Slow enough that the fixed interval falls inside one frame: the two
    /// alternate, so the update keeps half the frames instead of stopping.
    #[test]
    fn a_shared_frame_alternates_below_the_fixed_rate() {
        let status = ScriptStatus::default();
        status.set_initialized();
        let mut clocks = TickClocks::default();
        let ms = Duration::from_millis;

        let mut kinds = Vec::new();
        for frame in 1..=10u64 {
            let now = ms(frame * 60);
            let kind = next_tick_one_call_per_frame(&status, &mut clocks, now).expect("owed");
            kinds.push(kind);
            let _ = clocks.next(kind, now);
        }

        let updates = kinds
            .iter()
            .filter(|kind| **kind == TickKind::Update)
            .count();
        assert!(updates >= 4, "the update stopped: {kinds:?}");
    }
}
