//! Where each panel stands: carried with the viewer, anchored, settled and
//! filed into.

use wired_guest::math::{
    Transform,
    Vec3,
};

use super::Vui;
use crate::{
    component::event::{
        Event,
        Released,
    },
    machinery::{
        mount,
        pointer::{
            self,
            Gaze,
        },
    },
};

impl Vui {
    /// Moves every placed panel by the move the viewer just made, so a panel
    /// stands where it did relative to them. Landed levels share the slot's
    /// anchor, so they are carried by the same move.
    ///
    /// Every slot is carried even if one fails, since the jump is seen only
    /// once and a skipped panel would be left behind for good. The first
    /// failure is returned.
    pub(super) fn carry(&mut self, by: &Transform) -> anyhow::Result<()> {
        let mut first_err = None;
        for slot in self.slots.iter_mut().flatten() {
            let Some(anchor) = slot.anchor else {
                continue;
            };
            match slot.panel.carry(&self.doc, by, &anchor) {
                Ok(carried) => slot.anchor = Some(carried),
                Err(err) => {
                    slot.anchor = Some(*by * anchor);
                    first_err.get_or_insert(err);
                }
            }
        }
        first_err.map_or(Ok(()), Err)
    }

    /// Places a panel the first time it is drawn, and reports where it
    /// stands from then on.
    pub(super) fn anchor(&mut self, index: usize, gaze: &Gaze) -> anyhow::Result<Transform> {
        let Some(Some(slot)) = self.slots.get(index) else {
            return Ok(Transform::IDENTITY);
        };
        if let Some(anchor) = slot.anchor {
            return Ok(anchor);
        }
        let anchor = slot.panel.mount().anchor(&gaze.eye);
        let Some(Some(slot)) = self.slots.get_mut(index) else {
            return Ok(anchor);
        };
        slot.panel.place(&self.doc, &anchor)?;
        slot.anchor = Some(anchor);
        Ok(anchor)
    }

    /// Stands a panel where the level it just opened was let go.
    ///
    /// A panel measures its place once and keeps it, so this is the same
    /// move a summon makes: the level opens around the drop rather than back
    /// where the panel happened to be standing.
    pub(super) fn settle(&mut self, index: usize, at: Vec3, gaze: &Gaze) -> anyhow::Result<()> {
        let Some(Some(slot)) = self.slots.get_mut(index) else {
            return Ok(());
        };
        let anchor = mount::landed(at, &gaze.eye, &self.tuning);
        slot.panel.place(&self.doc, &anchor)?;
        slot.anchor = Some(anchor);
        Ok(())
    }

    /// Answers a landing, in the order a release means things: a grid under
    /// it takes it, else a level opens there, else the room has it.
    ///
    /// Nothing of the consumer's is drawn in that last case: the mote goes
    /// back where it came from, and a landing says where it was let go so the
    /// consumer can put its own thing there.
    pub(super) fn place(
        &mut self,
        index: usize,
        released: Released,
        opens_at: Option<Vec3>,
        gaze: &Gaze,
    ) -> anyhow::Result<()> {
        if let Some(target) = self.filed_into(gaze)
            && let Some(Some(slot)) = self.slots.get_mut(target)
            && slot.panel.stow(&released.mote)
        {
            self.report(index, [Event::Filed(released.mote)]);
            return Ok(());
        }

        if let Some(at) = opens_at
            && let Some(Some(slot)) = self.slots.get_mut(index)
            && let Some(event) = slot.panel.open(&released.mote)
        {
            self.settle(index, at, gaze)?;
            self.report(index, [event]);
            return Ok(());
        }

        self.report(index, [Event::Planted(released.mote, released.landing)]);
        Ok(())
    }

    /// The panel the pointer is over, which files rather than plants.
    pub(super) fn filed_into(&self, gaze: &Gaze) -> Option<usize> {
        self.slots.iter().enumerate().find_map(|(index, slot)| {
            let slot = slot.as_ref()?;
            let anchor = slot.shown.then_some(slot.anchor)??;
            pointer::aim(&gaze.ray, &anchor, slot.panel.field_lift())
                .filter(|aim| slot.panel.accepts(aim.local))
                .map(|_| index)
        })
    }
}
