//! The placard or notice a surface shows beside its motes.

use wired_guest::math::Vec3;

use super::Surface;
use crate::{
    component::placard::{
        self,
        Content,
        PlacardView,
    },
    part::spec::{
        MoteSpec,
        Role,
    },
};

impl Surface {
    /// What to show beside the motes: a notice if any drawn mote has one,
    /// else the attended mote's placard.
    pub(super) fn build_placard(&mut self, specs: &[MoteSpec]) -> Option<PlacardView> {
        let (content, at, radius) = self.notice(specs).or_else(|| self.description(specs))?;
        let sheet = self.sheets.sheet(content)?;
        Some(placard::view(sheet, at, radius, &self.tuning))
    }

    /// A drawn mote's notice, on its body whether or not it is attended. It
    /// is shown at full strength at once: it is news, not something earned
    /// by dwelling. Only one shows at a time, the first drawn slot's, and
    /// like the hover placard it gives way to a mote in hand.
    fn notice(&self, specs: &[MoteSpec]) -> Option<(Content, Vec3, f32)> {
        if !self.reveal.is_open() || self.grasp.is_seized() {
            return None;
        }
        self.drawn
            .iter()
            .zip(&self.views)
            .find_map(|(index, view)| {
                let message = specs.get(*index)?.notice.clone()?;
                Some((Content::Notice(message), view.position, view.radius))
            })
    }

    /// The attended mote's placard, mounted on wherever its body currently is.
    fn description(&self, specs: &[MoteSpec]) -> Option<(Content, Vec3, f32)> {
        if self.grasp.is_seized() {
            return None;
        }
        let slot = self.tracker.current()?;
        let spec = specs.get(self.spec_index(slot)?)?;
        let view = self.views.get(slot)?;
        // The way back and the page tabs are read by their chevrons alone.
        if matches!(spec.role, Role::Parent { .. } | Role::Page { .. }) {
            return None;
        }
        placard::earned(self.tracker.dwell(), &self.tuning)
            .then(|| (Content::describing(spec), view.position, view.radius))
    }
}
