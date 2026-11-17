//! The placard, page or notice a surface shows beside its motes.

use wired_guest::math::Vec3;

use super::Surface;
use crate::{
    component::placard::{
        self,
        Content,
        PlacardView,
    },
    machinery::layout::Layout,
    part::spec::{
        MoteSpec,
        Role,
    },
};

impl Surface {
    /// What to show beside the motes: a notice if any drawn mote has one,
    /// else the attended mote's placard. A bed shows a page above it
    /// instead.
    pub(super) fn build_placard(
        &mut self,
        specs: &[MoteSpec],
        layout: &Layout,
    ) -> Option<PlacardView> {
        if layout.is_bed() {
            return self.build_page(specs, layout);
        }
        let (content, at, radius) = self.notice(specs).or_else(|| self.description(specs))?;
        let sheet = self.sheets.sheet(content)?;
        Some(placard::view(sheet, at, radius, &self.tuning))
    }

    /// The page above a bed: a notice, else the attended key at once, else
    /// the last key pressed, else at the root what the consumer set. Nothing
    /// waits for a dwell, since the page is the bed's own, not a mote's
    /// afterthought.
    fn build_page(&mut self, specs: &[MoteSpec], layout: &Layout) -> Option<PlacardView> {
        if !self.reveal.is_open() {
            return None;
        }
        let content = self
            .notice(specs)
            .map(|(content, ..)| content)
            .or_else(|| self.attended_content(specs))
            .or_else(|| self.kept.clone())
            .or_else(|| self.resting.clone().filter(|_| layout.is_cross()))?;
        let sheet = self.sheets.sheet(content)?;
        let top = layout.extents(&self.tuning).y;
        Some(placard::page(sheet, top, &self.tuning))
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
        let (slot, spec) = self.attended_spec(specs)?;
        let view = self.views.get(slot)?;
        placard::earned(self.tracker.dwell(), &self.tuning)
            .then(|| (Content::describing(spec), view.position, view.radius))
    }

    /// What the attended mote says about itself.
    fn attended_content(&self, specs: &[MoteSpec]) -> Option<Content> {
        let (_, spec) = self.attended_spec(specs)?;
        Some(Content::describing(spec))
    }

    /// The attended slot and its mote, unless it is the way back or a page
    /// tab, which are read by their chevrons alone, or something is in hand.
    fn attended_spec<'a>(&self, specs: &'a [MoteSpec]) -> Option<(usize, &'a MoteSpec)> {
        if self.grasp.is_seized() {
            return None;
        }
        let slot = self.tracker.current()?;
        let spec = specs.get(self.spec_index(slot)?)?;
        if matches!(spec.role, Role::Parent { .. } | Role::Page { .. }) {
            return None;
        }
        Some((slot, spec))
    }
}
