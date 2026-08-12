use wired_math::types::Vec3;

use crate::{
    assist,
    attention::{
        Attention,
        Tracker,
    },
    grasp::{
        Grasp,
        Outcome,
    },
    layout::Layout,
    mote::{
        self,
        MoteSpec,
        Role,
    },
    palette::Palette,
    placard::{
        self,
        Placard,
        PlacardView,
    },
    tuning::Tuning,
    view::{
        Frame,
        PageView,
        SlotView,
        Style,
    },
};

/// The machinery every VUI surface shares: what holds attention, what is in
/// hand, which window of an oversized collection is drawn, and where every
/// body currently belongs.
///
/// Every public index is a **draw slot**, matching [`Surface::views`].
/// [`Surface::spec_index`] is the one translation back to the collection.
pub struct Surface {
    pub tuning:  Tuning,
    pub palette: Palette,
    tracker:     Tracker,
    grasp:       Grasp,
    lean:        Vec<Vec3>,
    views:       Vec<SlotView>,
    drawn:       Vec<usize>,
    placard:     Option<PlacardView>,
    page:        PageView,
    /// Leading slots held on every page, and how many slots remain for the
    /// window; both settled by the last [`Surface::update`].
    pinned:      usize,
    per_page:    usize,
}

impl Surface {
    #[must_use]
    pub fn new(capacity: usize, tuning: Tuning, palette: Palette) -> Self {
        Self {
            tuning,
            palette,
            tracker: Tracker::new(),
            grasp: Grasp::new(),
            lean: vec![Vec3::ZERO; capacity],
            views: Vec::with_capacity(capacity),
            drawn: Vec::with_capacity(capacity),
            placard: None,
            page: PageView::default(),
            pinned: 0,
            per_page: capacity,
        }
    }

    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.lean.len()
    }

    #[must_use]
    pub fn views(&self) -> &[SlotView] {
        &self.views
    }

    /// The collection index each drawn slot stands for, parallel to
    /// [`Surface::views`].
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

    /// The held slot once the hold is a take rather than a tap.
    #[must_use]
    pub fn displaced(&self) -> Option<usize> {
        self.grasp
            .seized()
            .filter(|held| held.displaced)
            .map(|held| held.slot)
    }

    /// The attended mote's placard, or `None` until attention has been held
    /// long enough.
    #[must_use]
    pub const fn placard(&self) -> Option<&PlacardView> {
        self.placard.as_ref()
    }

    /// A mote's style with no attention on it.
    #[must_use]
    pub const fn resting_style(&self, spec: &MoteSpec) -> Style {
        self.style(spec.role, Attention::Idle)
    }

    /// Turns to `page`, settled against what the collection actually has by
    /// the next [`Surface::update`] — so turning before the first draw is a
    /// real turn rather than a silent no-op.
    ///
    /// Attention does not survive: the slot under the pointer now stands for
    /// something else.
    pub const fn turn_to(&mut self, page: usize) {
        if page != self.page.index {
            self.page.index = page;
            self.tracker = Tracker::new();
        }
    }

    pub const fn turn_by(&mut self, delta: isize) {
        let page = self.page.index.saturating_add_signed(delta);
        self.turn_to(page);
    }

    /// Presses the slot currently holding attention.
    pub fn press(&mut self, at: Vec3) {
        let Some(slot) = self.tracker.current() else {
            return;
        };
        let takeable = self
            .views
            .get(slot)
            .is_some_and(|view| view.role.is_takeable());
        self.grasp.press(slot, at, takeable);
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
        self.repage(specs.len(), layout, pinned);
        self.collect(specs);

        // A mote in hand holds attention; nothing else is a drop target.
        let dragging = self.grasp.seized().is_some_and(|held| held.takeable);
        if dragging {
            self.tracker
                .update(self.grasp.seized().map(|held| held.slot), frame.delta);
        } else {
            let candidate = frame
                .aim
                .and_then(|aim| layout.resolve(aim.local, self.tracker.current(), &self.tuning))
                .filter(|slot| *slot < self.drawn.len());
            self.tracker.update(candidate, frame.delta);
        }

        if let (Some(hand), true) = (frame.hand, self.grasp.is_seized()) {
            self.grasp.track(hand, &self.tuning);
        }

        let seized = self.grasp.seized().map(|held| held.slot);
        let attended = self.tracker.current();

        self.views.clear();
        for slot in 0..self.drawn.len() {
            let (Some(spec), Some(plane)) = (specs.get(self.drawn[slot]), layout.slot(slot)) else {
                continue;
            };
            let local = Vec3::new(plane.x, plane.y, 0.0);
            let world = frame.anchor.translation + frame.anchor.rotation * local;
            let is_seized = seized == Some(slot);

            let attention =
                self.tracker
                    .state(slot, is_seized, attended.is_some_and(|held| held != slot));

            let target = frame.aim.map_or(Vec3::ZERO, |aim| {
                frame.anchor.rotation.inverse()
                    * assist::lean(world, aim.world, attention, &self.tuning)
            });
            self.lean[slot] =
                assist::approach(self.lean[slot], target, self.tuning.lean_speed, frame.delta);

            let position = match (is_seized && dragging, frame.hand) {
                (true, Some(hand)) => {
                    frame.anchor.rotation.inverse() * (hand - frame.anchor.translation)
                }
                _ => local + self.lean[slot],
            };

            let presentation = mote::present(spec, attention, &self.tuning);

            self.views.push(SlotView {
                position,
                radius: presentation.radius,
                style: self.style(spec.role, attention),
                role: spec.role,
                attention,
                pips: presentation.pips,
                seized: is_seized && dragging,
                label_offset: Vec3::new(
                    0.0,
                    -(presentation.radius + self.tuning.label_gap),
                    self.tuning.label_lift,
                ),
                label_size: self.tuning.label_size,
            });
        }

        self.placard = self.build_placard(specs);
    }

    /// Recomputes the window over `total`, clamping the current page into it.
    fn repage(&mut self, total: usize, layout: Layout, pinned: usize) {
        let slots = layout.len().min(self.capacity());
        self.pinned = pinned.min(slots).min(total);
        self.per_page = slots - self.pinned;
        let paged = total - self.pinned;

        let count = if self.per_page == 0 {
            usize::from(paged > 0).max(1)
        } else {
            paged.div_ceil(self.per_page).max(1)
        };
        let index = self.page.index.min(count - 1);
        if index != self.page.index {
            self.tracker = Tracker::new();
        }

        self.page = PageView {
            index,
            count,
            skipped: self.pinned + index * self.per_page,
            total,
        };
    }

    /// The collection index behind each drawn slot: the pinned lead, then this
    /// page's window.
    fn collect(&mut self, specs: &[MoteSpec]) {
        self.drawn.clear();
        self.drawn.extend(0..self.pinned);
        self.drawn
            .extend((self.page.skipped..specs.len()).take(self.per_page));
    }

    /// The attended mote's placard, mounted on wherever its body currently is.
    fn build_placard(&self, specs: &[MoteSpec]) -> Option<PlacardView> {
        if self.grasp.is_seized() {
            return None;
        }
        let slot = self.tracker.current()?;
        let spec = specs.get(self.spec_index(slot)?)?;
        let view = self.views.get(slot)?;
        let opacity = placard::opacity(self.tracker.dwell(), &self.tuning);
        (opacity > 0.0).then(|| {
            let placard = Placard::describing(spec);
            placard::view(&placard, view.position, view.radius, opacity, &self.tuning)
        })
    }

    const fn style(&self, role: Role, attention: Attention) -> Style {
        let color = if matches!(role, Role::Parent { .. }) && !attention.is_active() {
            self.palette.dim
        } else {
            self.palette.tint(attention)
        };
        Style {
            color,
            alpha: match role {
                Role::Group { .. } => self.palette.glass(attention),
                Role::Action | Role::Item | Role::Cast | Role::Parent { .. } => {
                    self.palette.solid_alpha
                }
            },
            emissive: self.palette.emissive(attention),
        }
    }
}
