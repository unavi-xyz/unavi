//! What a station asks of the panes it arranges, so the arrangement can be
//! run against a stand-in.

use wired_guest::math::{
    Transform,
    Vec3,
};

use crate::{
    component::pane::panes::{
        PaneId,
        Panes,
    },
    wired::input::types::PointerKind,
};

/// The panes a station of `T` arranges.
pub trait Stage<T> {
    /// A pane's size in columns and lines.
    fn cells(&self, id: T) -> (u32, u32);
    /// Hangs a pane at `pose` in the script's frame.
    fn hang(&mut self, id: T, pose: Transform);
    fn show(&mut self, id: T, shown: bool);
    /// The pointer whose grip pressed the pane while it hung, once.
    fn take_pulled(&mut self, id: T) -> Option<PointerKind>;
    /// Floats a pane at `at`, already in the hand of `by`.
    fn carry(&mut self, id: T, at: Vec3, by: PointerKind);
    /// Whether a hand holds a floating pane.
    fn carried(&self, id: T) -> bool;
    fn position(&self, id: T) -> Option<Vec3>;
    /// Takes a pane off whatever it hangs from.
    fn unmount(&mut self, id: T);
}

impl Stage<PaneId> for Panes {
    fn cells(&self, id: PaneId) -> (u32, u32) {
        Self::cells(self, id)
    }

    fn hang(&mut self, id: PaneId, pose: Transform) {
        Self::hang(self, id, None, pose);
    }

    fn show(&mut self, id: PaneId, shown: bool) {
        self.set_shown(id, shown);
    }

    fn take_pulled(&mut self, id: PaneId) -> Option<PointerKind> {
        Self::take_pulled(self, id)
    }

    fn carry(&mut self, id: PaneId, at: Vec3, by: PointerKind) {
        Self::carry(self, id, at, by);
    }

    fn carried(&self, id: PaneId) -> bool {
        Self::carried(self, id)
    }

    fn position(&self, id: PaneId) -> Option<Vec3> {
        Self::position(self, id)
    }

    fn unmount(&mut self, id: PaneId) {
        Self::unmount(self, id);
    }
}
