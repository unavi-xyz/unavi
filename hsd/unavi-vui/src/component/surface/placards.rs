//! The placard or notice a surface shows beside its motes.

use super::Surface;
use crate::{
    component::placard::{
        Placard,
        PlacardView,
        {
            self,
        },
    },
    part::spec::{
        MoteSpec,
        Role,
    },
};

impl Surface {
    /// What to show beside the motes: a notice if any drawn mote has one,
    /// else the attended mote's placard.
    pub(super) fn build_placard(&self, specs: &[MoteSpec]) -> Option<PlacardView> {
        self.build_notice(specs)
            .or_else(|| self.build_description(specs))
    }

    /// A drawn mote's notice, on its body whether or not it is attended. It
    /// is shown at full strength at once: it is news, not something earned
    /// by dwelling. Only one shows at a time, the first drawn slot's, and
    /// like the hover placard it gives way to a mote in hand.
    pub(super) fn build_notice(&self, specs: &[MoteSpec]) -> Option<PlacardView> {
        if !self.reveal.is_open() || self.grasp.is_seized() {
            return None;
        }
        self.drawn
            .iter()
            .zip(&self.views)
            .find_map(|(index, view)| {
                let message = specs.get(*index)?.notice.clone()?;
                Some(placard::view(
                    &Placard::notice(message),
                    view.position,
                    view.radius,
                    &self.tuning,
                ))
            })
    }

    /// The attended mote's placard, mounted on wherever its body currently is.
    pub(super) fn build_description(&self, specs: &[MoteSpec]) -> Option<PlacardView> {
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
        placard::earned(self.tracker.dwell(), &self.tuning).then(|| {
            let placard = Placard::describing(spec);
            placard::view(&placard, view.position, view.radius, &self.tuning)
        })
    }
}
