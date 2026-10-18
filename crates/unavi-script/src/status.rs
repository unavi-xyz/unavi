//! Where a script is in its life, shared with the tasks that run it.

use std::sync::{
    Arc,
    atomic::{
        AtomicBool,
        Ordering,
    },
};

use bevy::prelude::*;

/// One script's lifecycle flags. Shared rather than plain fields because the
/// tick that changes them runs off the world.
#[derive(Component, Clone, Default)]
pub struct ScriptStatus(Arc<Flags>);

#[derive(Default)]
struct Flags {
    initialized: AtomicBool,
    /// A trap leaves a component instance permanently un-enterable, so a
    /// trapped script is driven no further.
    trapped:     AtomicBool,
    /// Whether an `update` call is in flight.
    updating:    AtomicBool,
    /// Whether an `init` or `fixed-update` call is in flight.
    fixed:       AtomicBool,
    /// Whether any lifecycle call is in flight. Web uses this to hold the
    /// whole instance, since a suspended call keeps its `jco` task stack
    /// current on the page's thread.
    entered:     AtomicBool,
}

/// Which in-flight flag a call holds. `init` shares the fixed one, since it
/// is driven from the fixed schedule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lane {
    Update,
    Fixed,
}

impl ScriptStatus {
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.0.initialized.load(Ordering::Acquire)
    }

    pub fn set_initialized(&self) {
        self.0.initialized.store(true, Ordering::Release);
    }

    #[must_use]
    pub fn is_trapped(&self) -> bool {
        self.0.trapped.load(Ordering::Acquire)
    }

    /// Retires the script, answering whether this call was the one to do so,
    /// so a failure is reported once rather than every frame.
    #[must_use]
    pub fn trap(&self) -> bool {
        !self.0.trapped.swap(true, Ordering::AcqRel)
    }

    /// Claims `lane`, answering `false` when a call already holds it.
    #[must_use]
    pub fn begin(&self, lane: Lane) -> bool {
        !self.lane(lane).swap(true, Ordering::AcqRel)
    }

    pub fn end(&self, lane: Lane) {
        self.lane(lane).store(false, Ordering::Release);
    }

    /// Claims the whole instance for one lifecycle call, refusing while any
    /// call is in flight.
    ///
    /// Web drives a `jco` guest on the page's own thread. A call suspended on
    /// a host import leaves the instance's task stack current there, so a
    /// second call entered before the first resumes corrupts it: the second
    /// call's task takes over the global current-task slot, and the first's
    /// next host import can find no subtask. Native serializes calls on the
    /// store lock instead, and uses [`Lane`] to skip only a call's own
    /// schedule.
    #[must_use]
    pub fn enter(&self) -> bool {
        !self.0.entered.swap(true, Ordering::AcqRel)
    }

    pub fn leave(&self) {
        self.0.entered.store(false, Ordering::Release);
    }

    fn lane(&self, lane: Lane) -> &AtomicBool {
        match lane {
            Lane::Update => &self.0.updating,
            Lane::Fixed => &self.0.fixed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_trap_is_reported_once() {
        let status = ScriptStatus::default();
        assert!(status.trap());
        assert!(!status.trap());
        assert!(status.is_trapped());
    }

    #[test]
    fn a_lane_holds_one_call_at_a_time() {
        let status = ScriptStatus::default();
        assert!(status.begin(Lane::Update));
        assert!(!status.begin(Lane::Update));
        assert!(status.begin(Lane::Fixed), "the lanes are independent");
        status.end(Lane::Update);
        assert!(status.begin(Lane::Update));
    }

    #[test]
    fn an_entered_instance_refuses_a_second_call() {
        let status = ScriptStatus::default();
        assert!(status.enter());
        assert!(
            !status.enter(),
            "no second call may enter while one is in flight"
        );
        status.leave();
        assert!(status.enter());
    }
}
