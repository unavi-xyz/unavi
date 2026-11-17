//! The `slate_*` calls: a pane of rows standing where a script puts it.

use wired_guest::math::{
    Color,
    Vec2,
    Vec3,
};

use super::Vui;
use crate::component::slate::slates::{
    SlateEvent,
    SlateId,
};

impl Vui {
    pub fn slate(&mut self) -> anyhow::Result<SlateId> {
        self.slates.add(&self.doc)
    }

    pub fn slate_rows(&mut self, id: SlateId, rows: Vec<crate::component::slate::Row>) {
        self.slates.set_rows(id, rows);
    }

    #[must_use]
    pub fn slate_size(&self, id: SlateId) -> Vec2 {
        self.slates.size(id)
    }

    pub fn slate_at(&mut self, id: SlateId, at: Vec3) {
        self.slates.set_position(id, at);
    }

    pub fn slate_tether(&mut self, id: SlateId, to: Option<Vec3>) {
        self.slates.set_tether(id, to);
    }

    #[must_use]
    pub fn slate_position(&self, id: SlateId) -> Option<Vec3> {
        self.slates.position(id)
    }

    pub fn slate_accent(&mut self, id: SlateId, accent: Option<Color>) {
        self.slates.set_accent(id, accent);
    }

    pub fn slate_shown(&mut self, id: SlateId, shown: bool) {
        self.slates.set_shown(id, shown);
    }

    pub fn slate_drain(&mut self, id: SlateId) -> Vec<SlateEvent> {
        self.slates.drain(id)
    }

    pub fn slate_remove(&mut self, id: SlateId) -> anyhow::Result<()> {
        self.slates.remove(&self.doc, id)
    }
}
