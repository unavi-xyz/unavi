//! The mounts a station uses: hanging a pane in place, pulling one out of
//! its place into a hand, and asking whether a hand still has it.

use wired_guest::math::{
    Transform,
    Vec3,
};

use super::{
    Mount,
    PaneId,
    Panes,
};
use crate::{
    component::pane::{
        COLUMN,
        LINE,
    },
    wired::input::types::PointerKind,
};

impl Panes {
    /// Hangs a pane at `pose`, in the frame of prim `to` or of the script's
    /// document, and leaves it alone if it already hangs there.
    pub fn hang(&mut self, id: PaneId, to: Option<(u64, u64)>, pose: Transform) {
        let mount = Mount::Hung { to, pose };
        if let Some(mounted) = self.get_mut(id)
            && mounted.mount != Some(mount)
        {
            mounted.mount = Some(mount);
        }
    }

    /// Puts a pane in `hand`, at reading distance.
    pub fn hold(&mut self, id: PaneId, hand: PointerKind) {
        if let Some(mounted) = self.get_mut(id) {
            mounted.mount = Some(Mount::Held { hand });
        }
    }

    /// Takes a pane off whatever it is mounted on, which draws nothing.
    pub fn unmount(&mut self, id: PaneId) {
        if let Some(mounted) = self.get_mut(id) {
            mounted.mount = None;
        }
    }

    /// The pointer whose grip pressed the pane while it hung, once.
    pub fn take_pulled(&mut self, id: PaneId) -> Option<PointerKind> {
        self.get_mut(id)?.pulled.take()
    }

    /// Floats a pane centred on `at` already in the hand of `by`, so a grip
    /// that pressed it carries it on.
    pub fn carry(&mut self, id: PaneId, at: Vec3, by: PointerKind) {
        if let Some(mounted) = self.get_mut(id) {
            mounted.mount = Some(Mount::Floating { at });
            mounted.put = None;
            mounted.float.grab(by);
        }
    }

    /// Whether a hand holds the pane by its grip.
    #[must_use]
    pub fn carried(&self, id: PaneId) -> bool {
        self.slots
            .get(id.0)
            .and_then(Option::as_ref)
            .is_some_and(|mounted| mounted.float.held_by.is_some())
    }

    /// The pane's size in whole columns and lines.
    #[must_use]
    pub fn cells(&self, id: PaneId) -> (u32, u32) {
        self.slots
            .get(id.0)
            .and_then(Option::as_ref)
            .map_or((0, 0), |mounted| {
                let size = mounted.pane.size();
                ((size.x / COLUMN).round() as u32, (size.y / LINE).round() as u32)
            })
    }
}
