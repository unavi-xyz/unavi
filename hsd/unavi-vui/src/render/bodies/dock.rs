//! Parks and fits the icon inside whichever slot draws it.

use wired_guest::math::Quat;

use super::Bodies;
use crate::{
    mote::MoteSpec,
    render::draw,
    tree::Mote,
    view::SlotView,
    wired::scene::{
        document::Document,
        properties::Property,
    },
};

impl Bodies {
    /// Puts each drawn mote's icon inside its shell and parks the rest.
    ///
    /// Runs before [`Bodies::apply`], which reads back what a slot ended up
    /// holding to decide whether its shell is glass.
    pub fn icons(
        &self,
        doc: &Document,
        motes: &[Mote],
        specs: &[MoteSpec],
        views: &[SlotView],
        drawn: &[usize],
        delta: f32,
    ) -> anyhow::Result<()> {
        self.ensure(doc, views.len())?;
        // A gentle turn so the icons read as things rather than pictures;
        // zero keeps every icon still.
        self.spin
            .set(delta.mul_add(self.tuning.icon_spin, self.spin.get()));
        let spin = Quat::new(
            0.0,
            (self.spin.get() * 0.5).sin(),
            0.0,
            (self.spin.get() * 0.5).cos(),
        );
        let slots = self.slots.borrow();

        // The spec decides, not the mote: a slot standing for a level rather
        // than showing it wears none of that level's glyph. A glyph takes the
        // face instead of the icon, which waits parked until it is gone.
        let wanted = |slot: usize| {
            let index = *drawn.get(slot)?;
            let worn = specs.get(index).is_some_and(wears_icon);
            motes
                .get(index)
                .filter(|mote| worn && slot < views.len() && mote.has_icon())
        };

        // Handing every stale icon back first, so a mote that moved between
        // slots is not parked by the slot it left after arriving.
        for (index, slot) in slots.iter().enumerate() {
            let held = slot.icon.borrow();
            let keep = wanted(index).is_some_and(|mote| held.as_ref().is_some_and(|h| h.is(mote)));
            if keep || held.is_none() {
                continue;
            }
            if let Some(mote) = held.as_ref()
                && let Some(icon) = mote.icon()
            {
                doc.local()
                    .set(icon, Property::Parent(Some(self.park)))
                    .flush()?;
            }
            drop(held);
            *slot.icon.borrow_mut() = None;
        }

        for (index, slot) in slots.iter().enumerate() {
            let Some(mote) = wanted(index) else {
                continue;
            };
            let Some(icon) = mote.icon() else {
                continue;
            };
            let radius = views.get(index).map_or(0.0, |view| view.radius);
            let fit = mote.icon_fit(doc, self.tuning.icon_fill)?;
            if slot.icon.borrow().is_none() {
                doc.local()
                    .set(icon, Property::Parent(Some(slot.root)))
                    .flush()?;
                *slot.icon.borrow_mut() = Some(mote.clone());
            }
            doc.local()
                .set(
                    icon,
                    Property::Transform(draw::fitted(fit.center, radius * fit.scale, spin)),
                )
                .flush()?;
        }
        Ok(())
    }
}

/// Whether a mote's icon is the face its slot shows. A glyph takes the face
/// instead, and the icon waits parked until it is gone.
const fn wears_icon(spec: &MoteSpec) -> bool {
    spec.icon && spec.glyph.is_none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        glyph::Glyph,
        tree::Kind,
    };

    fn mote() -> Mote {
        let mote = Mote::new(Kind::Item, "Tool");
        mote.set_icon(Some((1, 1)));
        mote
    }

    #[test]
    fn an_icon_is_worn_until_a_glyph_takes_its_place() {
        let mote = mote();
        assert!(wears_icon(&mote.spec()));

        mote.set_glyph(Some(
            Glyph::new(3, &[0b0001_0000, 0]).expect("a three grid"),
        ));
        assert!(!wears_icon(&mote.spec()), "the icon is parked");

        mote.set_glyph(None);
        assert!(wears_icon(&mote.spec()), "and returns with the glyph gone");
    }
}
