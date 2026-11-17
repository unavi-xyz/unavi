//! Which window of an oversized level is drawn, and the tabs that turn it.

use smol_str::SmolStr;
use wired_guest::math::{
    Vec2,
    Vec3,
};

use super::Surface;
use crate::{
    machinery::{
        attention::Tracker,
        layout::Layout,
    },
    part::{
        spec::{
            MoteSpec,
            Role,
        },
        view::PageView,
    },
    tokens::{
        palette::{
            self,
        },
        state::Condition,
    },
};

impl Surface {
    /// Turns to `page`, settled against what the collection actually has by
    /// the next [`Surface::update`] — so turning before the first draw is a
    /// real turn rather than a silent no-op.
    ///
    /// Attention does not survive: the slot under the pointer now stands for
    /// something else.
    pub fn turn_to(&mut self, page: usize) {
        if page != self.page.index {
            self.page.index = page;
            self.kept = None;
            self.forget_slots();
        }
    }

    /// Drops everything a slot has built up toward the mote that stood in
    /// it: attention, warmth and lean. After a page turn or a level change
    /// the slot stands for something else, and carrying these over shows as
    /// the new occupant flashing in already lit.
    pub(super) fn forget_slots(&mut self) {
        self.tracker = Tracker::new();
        self.heat.fill(0.0);
        self.lean.fill(Vec3::ZERO);
    }

    pub fn turn_by(&mut self, delta: isize) {
        let page = self.page.index.saturating_add_signed(delta);
        self.turn_to(page);
    }

    /// Turns the page if `slot` is a page tab, reporting whether it was one.
    pub fn turn_tab(&mut self, slot: usize) -> bool {
        let Some(Role::Page { forward, .. }) = self.views.get(slot).map(|view| view.role) else {
            return false;
        };
        self.turn_by(if forward { 1 } else { -1 });
        true
    }

    /// Starts over for a different level: nothing carries over from the
    /// motes that stood in these slots. The motes arrive in turn again, but
    /// the form stays open.
    pub fn relevel(&mut self) {
        self.page.index = 0;
        self.kept = None;
        self.forget_slots();
        self.reveal.restage();
    }

    /// The tabs to draw this frame, each with where it sits: back a page when
    /// there are pages behind, forward when there are pages ahead.
    pub(super) fn tabs(&self, layout: &Layout) -> Vec<(MoteSpec, Vec2)> {
        let behind = self.page.index;
        let ahead = self.page.count - self.page.index - 1;
        [(false, behind), (true, ahead)]
            .into_iter()
            .filter(|(_, remaining)| *remaining > 0)
            .map(|(forward, remaining)| {
                let spec = MoteSpec {
                    role:        Role::Page { forward, remaining },
                    label:       SmolStr::default(),
                    description: None,
                    condition:   Condition::default(),
                    icon:        false,
                    glyph:       None,
                    tint:        None,
                    film:        palette::FILM,
                    frost:       0.0,
                    notice:      None,
                };
                (spec, layout.tab(forward, &self.tuning))
            })
            .collect()
    }

    /// The tab, as a draw slot after the `level` ones, the pointer is over.
    pub(super) fn extra_at(
        &self,
        local: Vec2,
        extras: &[(MoteSpec, Vec2)],
        level: usize,
    ) -> Option<usize> {
        extras
            .iter()
            .enumerate()
            .map(|(index, (_, at))| (level + index, (local - *at).length()))
            .filter(|(_, distance)| *distance <= self.tuning.tab_hit)
            .min_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(slot, _)| slot)
    }

    /// Recomputes the window over `total`, clamping the current page into it.
    pub(super) fn repage(&mut self, total: usize, layout: Layout, pinned: usize) {
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
            self.forget_slots();
        }

        self.page = PageView {
            index,
            count,
            skipped: self.pinned + index * self.per_page,
            total,
        };
    }
}
