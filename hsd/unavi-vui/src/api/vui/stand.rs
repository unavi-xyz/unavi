//! The `stand_*` calls: a placard standing where a script puts it.

use wired_guest::math::{
    Color,
    Vec2,
    Vec3,
};

use super::Vui;
use crate::component::{
    placard::Placard,
    stand::StandId,
};

impl Vui {
    /// Makes a placard that stands where a script puts it. Nothing shows
    /// until it has text and is shown.
    pub fn stand(&mut self) -> anyhow::Result<StandId> {
        self.stands.add(&self.doc)
    }

    pub fn stand_text(&mut self, id: StandId, text: &Placard) {
        self.stands.set_text(id, text, &self.tuning);
    }

    /// The laid-out pane, width by height in metres, as it will stand.
    #[must_use]
    pub fn stand_size(&self, id: StandId) -> Vec2 {
        self.stands.size(id)
    }

    pub fn stand_at(&mut self, id: StandId, at: Vec3) {
        self.stands.set_position(id, at);
    }

    pub fn stand_scale(&mut self, id: StandId, scale: f32) {
        self.stands.set_scale(id, scale, &self.tuning);
    }

    pub fn stand_accent(&mut self, id: StandId, accent: Option<Color>) {
        self.stands.set_accent(id, accent);
    }

    pub fn stand_tether(&mut self, id: StandId, to: Option<Vec3>) {
        self.stands.set_tether(id, to);
    }

    pub fn stand_shown(&mut self, id: StandId, shown: bool) {
        self.stands.set_shown(id, shown);
    }

    /// Frees a standing placard's prims.
    pub fn stand_remove(&mut self, id: StandId) -> anyhow::Result<()> {
        self.stands.remove(&self.doc, id)
    }
}
