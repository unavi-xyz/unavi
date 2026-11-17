//! The `pane_*` calls: a framed page of blocks standing or floating where a
//! script puts it.

use wired_guest::math::{
    Vec2,
    Vec3,
};

use super::Vui;
use crate::component::pane::{
    Event,
    Pane,
    Refused,
    block::Block,
    panes::{
        Mount,
        PaneId,
    },
};

impl Vui {
    pub fn pane(&mut self, pane: Pane) -> anyhow::Result<PaneId> {
        self.panes.add(&self.doc, pane)
    }

    pub fn pane_title(&mut self, id: PaneId, title: &str) -> Result<(), Refused> {
        self.panes.set_title(id, title)
    }

    pub fn pane_status(&mut self, id: PaneId, status: Option<&str>) -> Result<(), Refused> {
        self.panes.set_status(id, status)
    }

    pub fn pane_blocks(&mut self, id: PaneId, blocks: Vec<Block>) -> Result<(), Refused> {
        self.panes.set_blocks(id, blocks)
    }

    /// The pane, width by height in metres.
    #[must_use]
    pub fn pane_size(&self, id: PaneId) -> Vec2 {
        self.panes.size(id, self.tuning.pane_em)
    }

    pub fn pane_standing(&mut self, id: PaneId, at: Vec3, tether: Option<Vec3>) {
        self.panes.set_mount(id, Mount::Standing { at, tether });
    }

    pub fn pane_floating(&mut self, id: PaneId, at: Vec3) {
        self.panes.set_mount(id, Mount::Floating { at });
    }

    #[must_use]
    pub fn pane_position(&self, id: PaneId) -> Option<Vec3> {
        self.panes.position(id)
    }

    pub fn pane_shown(&mut self, id: PaneId, shown: bool) {
        self.panes.set_shown(id, shown);
    }

    pub fn pane_drain(&mut self, id: PaneId) -> Vec<Event> {
        self.panes.drain(id)
    }

    pub fn pane_remove(&mut self, id: PaneId) -> anyhow::Result<()> {
        self.panes.remove(&self.doc, id)
    }
}
