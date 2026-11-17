//! What every VUI surface shares, independent of layout: attention, grasp,
//! paging and the styles those produce.

use wired_guest::math::{
    Transform,
    Vec2,
    Vec3,
};

use crate::{
    component::placard::{
        Cache,
        PlacardView,
    },
    machinery::{
        assist,
        attention::Tracker,
        grasp::{
            Grasp,
            Outcome,
        },
        layout::Layout,
        reveal::Reveal,
    },
    part::{
        spec::{
            MoteSpec,
            {
                self,
            },
        },
        view::{
            Frame,
            PageView,
            SlotView,
        },
    },
    tokens::{
        palette::Palette,
        state::Attention,
        tuning::Tuning,
    },
};

mod paging;
mod placards;
mod style;

/// Slots a surface keeps for its page tabs, on top of the ones it keeps for
/// the level itself.
pub const TAB_SLOTS: usize = 2;

/// The machinery every VUI surface shares: what holds attention, what is in
/// hand, which window of an oversized collection is drawn, and where every
/// body currently belongs.
///
/// Every public index is a **draw slot**, matching [`Surface::views`]: the
/// level's own slots first, then any page tabs.
/// [`Surface::spec_index`] is the one translation back to the collection, and
/// answers `None` for a tab, which stands for no mote.
pub struct Surface {
    tuning:   Tuning,
    /// Slots the level may fill, which is not counting the tabs.
    capacity: usize,
    palette:  Palette,
    reveal:   Reveal,
    tracker:  Tracker,
    grasp:    Grasp,
    lean:     Vec<Vec3>,
    /// How far each slot has come toward the attention it is under, eased
    /// rather than switched. Parallel to [`Surface::lean`], and indexed by
    /// draw slot like everything else here.
    heat:     Vec<f32>,
    views:    Vec<SlotView>,
    drawn:    Vec<usize>,
    placard:  Option<PlacardView>,
    sheets:   Cache,
    page:     PageView,
    /// Leading slots held on every page, and how many slots remain for the
    /// window; both settled by the last [`Surface::update`].
    pinned:   usize,
    per_page: usize,
}

impl Surface {
    #[must_use]
    pub fn new(capacity: usize, tuning: Tuning, palette: Palette) -> Self {
        let slots = capacity + TAB_SLOTS;
        Self {
            tuning,
            capacity,
            palette,
            reveal: Reveal::ARRIVING,
            tracker: Tracker::new(),
            grasp: Grasp::new(),
            lean: vec![Vec3::ZERO; slots],
            heat: vec![0.0; slots],
            views: Vec::with_capacity(slots),
            drawn: Vec::with_capacity(capacity),
            placard: None,
            sheets: Cache::default(),
            page: PageView::default(),
            pinned: 0,
            per_page: capacity,
        }
    }

    #[must_use]
    pub const fn tuning(&self) -> &Tuning {
        &self.tuning
    }

    #[must_use]
    pub const fn palette(&self) -> &Palette {
        &self.palette
    }

    /// The most motes of a level it can draw at once.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Every slot it may ever draw: the level's and the page tabs'.
    #[must_use]
    pub const fn slots(&self) -> usize {
        self.capacity + TAB_SLOTS
    }

    /// How far the centre has arrived, which a readout there comes in by.
    #[must_use]
    pub const fn centre_bloom(&self) -> f32 {
        self.reveal.centre()
    }

    pub fn views(&self) -> &[SlotView] {
        &self.views
    }

    /// The collection index each of the level's slots stands for. Parallel to
    /// the leading [`Surface::views`]; the tabs after them have none.
    #[must_use]
    pub fn drawn(&self) -> &[usize] {
        &self.drawn
    }

    #[must_use]
    pub fn spec_index(&self, slot: usize) -> Option<usize> {
        self.drawn.get(slot).copied()
    }

    #[must_use]
    pub const fn page(&self) -> PageView {
        self.page
    }

    #[must_use]
    pub const fn attended(&self) -> Option<usize> {
        self.tracker.current()
    }

    #[must_use]
    pub const fn is_seized(&self) -> bool {
        self.grasp.is_seized()
    }

    /// Carries a hold in progress by the move the viewer made.
    pub fn carry(&mut self, by: &Transform) {
        self.grasp.carry(by);
    }

    /// The held slot once the hold is a take rather than a tap.
    #[must_use]
    pub fn displaced(&self) -> Option<usize> {
        self.grasp
            .seized()
            .filter(|held| held.displaced)
            .map(|held| held.slot)
    }

    /// The slot the grasp is down on, whether or not it has travelled.
    #[must_use]
    pub fn seized_slot(&self) -> Option<usize> {
        self.grasp.seized().map(|held| held.slot)
    }

    /// Starts the surface arriving, or sends it away.
    pub const fn set_open(&mut self, open: bool) {
        self.reveal.set_open(open);
    }

    /// Whether it is up or arriving, rather than sent away.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.reveal.is_open()
    }

    /// Whether anything is still drawn. A surface that has finished going away
    /// needs no more frames.
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.reveal.is_visible()
    }

    /// The attended mote's placard, or `None` until attention has been held
    /// long enough.
    #[must_use]
    pub const fn placard(&self) -> Option<&PlacardView> {
        self.placard.as_ref()
    }

    /// Presses the slot currently holding attention.
    ///
    /// Only a grip carries: a mote acted on with the trigger fires where it
    /// sits however far the hand then travels, which is what makes the two
    /// buttons mean different things on the same mote.
    pub fn press(&mut self, at: Vec3, carrying: bool) {
        let Some(slot) = self.tracker.current() else {
            return;
        };
        let takeable = carrying
            && self
                .views
                .get(slot)
                .is_some_and(|view| view.role.is_takeable());
        self.grasp.press(slot, at, takeable);
    }

    /// Drops a grasp in progress without it tapping or placing anything.
    pub const fn cancel(&mut self) {
        self.grasp.cancel();
    }

    /// Releasing after the pointer has wandered off a fixed mote cancels
    /// rather than activating.
    pub fn release(&mut self) -> Option<Outcome> {
        let wandered = self
            .grasp
            .seized()
            .is_some_and(|held| !held.takeable && self.tracker.current() != Some(held.slot));
        let outcome = self.grasp.release();
        if wandered { None } else { outcome }
    }

    /// Lays out `specs` under `layout`, drawing the first `pinned` of them in
    /// the leading slots on every page.
    ///
    /// Anything past one page's worth paginates rather than being dropped.
    pub fn update(&mut self, specs: &[MoteSpec], layout: Layout, pinned: usize, frame: &Frame) {
        self.reveal.update(frame.delta, &self.tuning);
        self.repage(specs.len(), layout, pinned);
        self.collect(specs);
        let level = self.drawn.len();
        let extras = self.tabs(&layout);

        // A mote in hand holds attention; nothing else is a drop target.
        let dragging = self.grasp.seized().is_some_and(|held| held.takeable);
        if !self.reveal.is_open() {
            // Nothing on its way out is attending anything. Its slots collapse
            // to the anchor but the placard hangs off the surface rather than
            // off a slot, so it would otherwise be left full-size over the
            // point the surface vanished into.
            self.tracker.update(None, frame.delta);
        } else if dragging {
            self.tracker
                .update(self.grasp.seized().map(|held| held.slot), frame.delta);
        } else {
            let candidate = frame
                .aim
                .and_then(|aim| self.aimed_slot(aim.local, &layout, &extras, level));
            self.tracker.update(candidate, frame.delta);
        }

        if let (Some(hand), true) = (frame.hand, self.grasp.is_seized()) {
            self.grasp.track(hand, &self.tuning);
        }

        let seized = self.grasp.seized().map(|held| held.slot);
        let attended = self.tracker.current();
        // Nothing warms to an approach while something is in hand: a mote
        // lighting up under a carried item claims to be a drop target, and
        // it is not.
        let approaching = frame.aim.filter(|_| !dragging);

        self.views.clear();
        let count = level + extras.len();
        for slot in 0..count {
            let (spec, plane) = if slot < level {
                let (Some(spec), Some(plane)) = (specs.get(self.drawn[slot]), layout.slot(slot))
                else {
                    continue;
                };
                (spec, plane)
            } else {
                let (spec, plane) = &extras[slot - level];
                (spec, *plane)
            };
            let local = Vec3::new(plane.x, plane.y, 0.0);
            let world = frame.anchor.translation + frame.anchor.rotation * local;
            let is_seized = seized == Some(slot);

            let attention = self.tracker.state(slot, is_seized);

            // A mote that has not been chosen warms by how near the pointer
            // has come, topping out at `Near`. A flat lift for every sibling
            // says only that the surface is in use, and says it as loudly on
            // the mote furthest from the hand as on the one beside it.
            let want = match attention {
                Attention::Attended | Attention::Engaged => attention.heat(),
                Attention::Idle | Attention::Near => {
                    let near = approaching
                        .map_or(0.0, |aim| assist::proximity(world, aim.world, &self.tuning));
                    Attention::Near.heat() * near
                }
            };
            self.heat[slot] = assist::approach_scalar(
                self.heat[slot],
                want,
                self.tuning.heat_speed,
                frame.delta,
                self.tuning.heat_settle,
            );
            let heat = self.heat[slot];

            // The field recedes around whatever is chosen, so the attended
            // mote stands out by everything else giving way rather than by
            // burning brighter still.
            let crowd = if attended.is_some_and(|held| held != slot) {
                self.tuning.sibling_dim
            } else {
                1.0
            };

            let target = frame.aim.map_or(Vec3::ZERO, |aim| {
                frame.anchor.rotation.inverse()
                    * assist::lean(world, aim.world, attention, &self.tuning)
            });
            self.lean[slot] =
                assist::approach(self.lean[slot], target, self.tuning.lean_speed, frame.delta);

            // A carried mote is where the hand is, and nothing about the form
            // arriving or leaving should drag it off the pointer.
            let position = match (is_seized && dragging, frame.hand) {
                (true, Some(hand)) => {
                    frame.anchor.rotation.inverse() * (hand - frame.anchor.translation)
                }
                _ => (local + self.lean[slot]) * self.reveal.form(),
            };

            let presentation = spec::present(spec, heat, &self.tuning);

            self.views.push(SlotView {
                position,
                radius: presentation.radius,
                style: self.style_at(spec.role, heat, spec.condition, spec.tint, crowd),
                role: spec.role,
                attention,
                heat,
                pips: presentation.pips,
                seized: is_seized && dragging,
                label_offset: Vec3::new(
                    0.0,
                    -(presentation.radius + self.tuning.label_gap),
                    self.tuning.label_lift,
                ),
                label_size: self.tuning.label_size,
                bloom: if is_seized && dragging {
                    1.0
                } else {
                    self.reveal.slot(slot, count, &self.tuning)
                },
            });
        }

        self.placard = self.build_placard(specs);
    }

    /// The slot under `local`: a tab if the pointer is over one, else
    /// whichever of the level's `level` slots the layout resolves.
    fn aimed_slot(
        &self,
        local: Vec2,
        layout: &Layout,
        extras: &[(MoteSpec, Vec2)],
        level: usize,
    ) -> Option<usize> {
        let current = self.tracker.current().filter(|slot| *slot < level);
        self.extra_at(local, extras, level).or_else(|| {
            layout
                .resolve(local, current, &self.tuning)
                .filter(|slot| *slot < level)
        })
    }

    /// The collection index behind each drawn slot: the pinned lead, then this
    /// page's window.
    fn collect(&mut self, specs: &[MoteSpec]) {
        self.drawn.clear();
        self.drawn.extend(0..self.pinned);
        self.drawn
            .extend((self.page.skipped..specs.len()).take(self.per_page));
    }
}

#[cfg(test)] mod tests;
