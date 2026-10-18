//! What every VUI surface shares, independent of layout: attention, grasp,
//! paging and the styles those produce.

use smol_str::SmolStr;
use wired_guest::math::{
    Color,
    Transform,
    Vec2,
    Vec3,
};

use crate::{
    assist,
    attention::{
        self,
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
    palette::{
        self,
        Palette,
    },
    placard::{
        self,
        Placard,
        PlacardView,
    },
    reveal::Reveal,
    tuning::Tuning,
    view::{
        Frame,
        PageView,
        SlotView,
        Style,
    },
};

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

    /// Turns to `page`, settled against what the collection actually has by
    /// the next [`Surface::update`] — so turning before the first draw is a
    /// real turn rather than a silent no-op.
    ///
    /// Attention does not survive: the slot under the pointer now stands for
    /// something else.
    pub fn turn_to(&mut self, page: usize) {
        if page != self.page.index {
            self.page.index = page;
            self.forget_slots();
        }
    }

    /// Drops everything a slot has built up toward the mote that stood in
    /// it: attention, warmth and lean. After a page turn or a level change
    /// the slot stands for something else, and carrying these over shows as
    /// the new occupant flashing in already lit.
    fn forget_slots(&mut self) {
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
        self.forget_slots();
        self.reveal.restage();
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

            let presentation = mote::present(spec, heat, &self.tuning);

            self.views.push(SlotView {
                position,
                radius: presentation.radius,
                style: self.style_at(spec.role, heat, spec.active, spec.tint, crowd),
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

    /// The tabs to draw this frame, each with where it sits: back a page when
    /// there are pages behind, forward when there are pages ahead.
    fn tabs(&self, layout: &Layout) -> Vec<(MoteSpec, Vec2)> {
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
                    active:      false,
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

    /// The tab, as a draw slot after the `level` ones, the pointer is over.
    fn extra_at(&self, local: Vec2, extras: &[(MoteSpec, Vec2)], level: usize) -> Option<usize> {
        extras
            .iter()
            .enumerate()
            .map(|(index, (_, at))| (level + index, (local - *at).length()))
            .filter(|(_, distance)| *distance <= self.tuning.tab_hit)
            .min_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(slot, _)| slot)
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
            self.forget_slots();
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

    /// What to show beside the motes: a notice if any drawn mote has one,
    /// else the attended mote's placard.
    fn build_placard(&self, specs: &[MoteSpec]) -> Option<PlacardView> {
        self.build_notice(specs)
            .or_else(|| self.build_description(specs))
    }

    /// A drawn mote's notice, on its body whether or not it is attended. It
    /// is shown at full strength at once: it is news, not something earned
    /// by dwelling. Only one shows at a time, the first drawn slot's, and
    /// like the hover placard it gives way to a mote in hand.
    fn build_notice(&self, specs: &[MoteSpec]) -> Option<PlacardView> {
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
    fn build_description(&self, specs: &[MoteSpec]) -> Option<PlacardView> {
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

    /// A role's style sampled at `heat`, by blending the two attention states
    /// it falls between.
    ///
    /// Blending the finished styles rather than each palette value keeps every
    /// rule in [`Surface::style`] — a parent mote receding, a container's
    /// glass, an item's warmth — exactly what it was at each state, and none
    /// of them has to know heat exists.
    /// `crowd` is how much of its brightness the mote keeps while something
    /// else holds attention; exactly 1 leaves the sampled style untouched.
    fn style_at(
        &self,
        role: Role,
        heat: f32,
        active: bool,
        tint: Option<Color>,
        crowd: f32,
    ) -> Style {
        let (low, high, t) = attention::bracket(heat);
        // Landing on an endpoint exactly, rather than blending to it: a style
        // is only rewritten when it differs, and `a + (b - a) * 1.0` is not
        // `b` in floating point — so a settled mote would redraw forever.
        let sampled = if t <= 0.0 {
            self.style(role, low, active, tint)
        } else if t >= 1.0 {
            self.style(role, high, active, tint)
        } else {
            let (from, to) = (
                self.style(role, low, active, tint),
                self.style(role, high, active, tint),
            );
            Style {
                color:    palette::blend(from.color, to.color, t),
                alpha:    (to.alpha - from.alpha).mul_add(t, from.alpha),
                emissive: (to.emissive - from.emissive).mul_add(t, from.emissive),
            }
        };
        Style {
            emissive: sampled.emissive * crowd,
            ..sampled
        }
    }

    const fn style(
        &self,
        role: Role,
        attention: Attention,
        active: bool,
        tint: Option<Color>,
    ) -> Style {
        // A mote's own hue is what it *is*, so it beats every resting rule —
        // the source's warmth and the parent's dim both belong to the neutral
        // family. What is in hand keeps the hue too, lit up by the grip
        // rather than recoloured: the accent's grey reads as a mote going
        // dead when it is selected, where a glow keeps it alive.
        let color = match (role, attention, tint) {
            (_, Attention::Engaged, Some(tint)) => self.palette.tinted(tint, attention),
            (_, Attention::Engaged, _) => self.palette.tint(attention),
            // An off toggle keeps its hue but sits back from it, so the tools
            // of a ring stay told apart while the fill says which are on.
            (Role::Toggle, _, Some(tint)) if !active => {
                self.palette.tinted(self.palette.muted(tint), attention)
            }
            (Role::Toggle, ..) if !active => self.palette.dim,
            (_, _, Some(tint)) => self.palette.tinted(tint, attention),
            (Role::Parent { .. } | Role::Page { .. }, ..) if !attention.is_active() => {
                self.palette.dim
            }
            (Role::Item { .. }, ..) => self.palette.item(attention, role.is_source()),
            (_, _, None) => self.palette.tint(attention),
        };
        Style {
            color,
            alpha: match role {
                // A toggle's fill *is* the signal: on is the mote wearing its
                // hue, off is the same glass as everything else.
                Role::Toggle if active => self.palette.toggle_active_alpha,
                Role::Toggle
                | Role::Group { .. }
                | Role::Action
                | Role::Item { .. }
                | Role::Cast
                | Role::Parent { .. }
                | Role::Page { .. } => self.palette.glass(attention),
            },
            emissive: self.palette.emissive_lit(attention, active),
        }
    }
}

#[cfg(test)]
mod tests {
    use smol_str::SmolStr;
    use wired_guest::math::{
        Quat,
        Transform,
        Vec2,
    };

    use super::*;
    use crate::{
        layout::{
            Centre,
            Layout,
        },
        mote::{
            Arrange,
            PipPlacement,
            Role,
        },
        view::Aim,
    };

    const R: f32 = Tuning::DEFAULT.orbit_radius;
    const PITCH: Vec2 = Vec2::new(0.08, 0.08);

    fn spec(role: Role) -> MoteSpec {
        MoteSpec {
            role,
            label: SmolStr::new_static("Citrus"),
            description: None,
            active: false,
            icon: false,
            glyph: None,
            tint: None,
            film: crate::palette::FILM,
            frost: 0.0,
            notice: None,
        }
    }

    fn named(label: &str) -> MoteSpec {
        MoteSpec {
            role:        Role::Action,
            label:       SmolStr::new(label),
            description: None,
            active:      false,
            icon:        false,
            glyph:       None,
            tint:        None,
            film:        crate::palette::FILM,
            frost:       0.0,
            notice:      None,
        }
    }

    fn takeable() -> MoteSpec {
        spec(Role::Item { unique: false })
    }

    fn group(children: usize, groups: usize) -> MoteSpec {
        spec(Role::Group {
            children,
            groups,
            arrange: Arrange::Orbit,
        })
    }

    fn anchor() -> Transform {
        Transform {
            translation: Vec3::ZERO,
            rotation:    Quat::IDENTITY,
            scale:       Vec3::ONE,
        }
    }

    fn frame(aim: Option<Aim>) -> Frame {
        Frame {
            anchor: anchor(),
            aim,
            hand: aim.map(|aim| aim.world),
            delta: 0.016,
        }
    }

    fn frame_with_hand(aim: Option<Aim>, hand: Vec3) -> Frame {
        Frame {
            hand: Some(hand),
            ..frame(aim)
        }
    }

    fn aim_at(local: Vec2) -> Aim {
        Aim {
            local,
            world: Vec3::new(local.x, local.y, 0.0),
        }
    }

    fn surface() -> Surface {
        Surface::new(12, Tuning::DEFAULT, Palette::DEFAULT)
    }

    /// Steps the slots under `centre`, the way an orbit does.
    fn ring_update(surface: &mut Surface, specs: &[MoteSpec], centre: Centre, frame: &Frame) {
        let (layout, pinned) = Layout::orbit(specs.len(), centre, surface.capacity(), R);
        surface.update(specs, layout, pinned, frame);
    }

    /// Steps a grid, the way a grid shape does.
    fn grid_update(surface: &mut Surface, specs: &[MoteSpec], columns: usize, rows: usize) {
        let layout = Layout::grid(columns, rows, PITCH);
        surface.update(specs, layout, 0, &frame(None));
    }

    /// Holds attention past the placard delay.
    fn dwell(surface: &mut Surface, specs: &[MoteSpec], aim: Aim) {
        let frame = Frame {
            delta: Tuning::DEFAULT.placard_delay + 0.05,
            ..frame(Some(aim))
        };
        ring_update(surface, specs, Centre::Open, &frame);
        ring_update(surface, specs, Centre::Open, &frame);
    }

    #[test]
    fn the_centre_holds_the_first_mote_whatever_it_stands_for() {
        let mut surface = Surface::new(5, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = [
            named("Hand"),
            named("Home"),
            named("Places"),
            named("Pocket"),
            named("Self"),
        ];
        ring_update(&mut surface, &specs, Centre::Held, &frame(None));

        assert_eq!(surface.views().len(), 5);
        assert_eq!(
            surface.views()[0].position.truncate(),
            Vec2::ZERO,
            "a frozen root can put its own subject in the middle, not only \
             the way back"
        );
        assert_eq!(surface.spec_index(0), Some(0));
        assert!(!surface.page().is_paged());
    }

    #[test]
    fn pips_report_the_real_child_count() {
        let mut surface = surface();
        ring_update(&mut surface, &[group(3, 0)], Centre::Open, &frame(None));
        assert_eq!(surface.views()[0].pips.count, 3);
        assert!(!surface.views()[0].pips.overflow);
    }

    #[test]
    fn container_children_are_marked_for_see_through_drawing() {
        let mut surface = surface();
        ring_update(&mut surface, &[group(5, 2)], Centre::Open, &frame(None));
        assert_eq!(surface.views()[0].pips.groups(), 2);
    }

    #[test]
    fn an_oversized_branch_reports_overflow_rather_than_lying() {
        let mut surface = surface();
        ring_update(
            &mut surface,
            &[group(Tuning::DEFAULT.pip_cap + 5, 0)],
            Centre::Open,
            &frame(None),
        );
        assert_eq!(surface.views()[0].pips.count, Tuning::DEFAULT.pip_cap);
        assert!(surface.views()[0].pips.overflow);
    }

    #[test]
    fn leaves_carry_no_pips() {
        let mut surface = surface();
        ring_update(
            &mut surface,
            &[spec(Role::Action), spec(Role::Cast)],
            Centre::Open,
            &frame(None),
        );
        assert!(surface.views().iter().all(|view| view.pips.count == 0));
    }

    #[test]
    fn a_branch_reads_bigger_than_a_leaf() {
        let mut surface = surface();
        ring_update(
            &mut surface,
            &[group(2, 0), spec(Role::Action)],
            Centre::Open,
            &frame(None),
        );
        let (branch, leaf) = (surface.views()[0], surface.views()[1]);
        assert!(branch.radius > leaf.radius, "containers read as containers");
    }

    #[test]
    fn the_parent_mote_is_small_dim_and_marked_around_itself() {
        let mut surface = surface();
        ring_update(
            &mut surface,
            &[spec(Role::Parent { depth: 2 }), spec(Role::Action)],
            Centre::Held,
            &frame(None),
        );
        let parent = surface.views()[0];
        assert!(parent.radius < surface.views()[1].radius);
        assert_eq!(parent.style.color, Palette::DEFAULT.dim);
        assert_eq!(parent.pips.count, 2, "depth is legible without text");
        assert_eq!(parent.pips.placement, PipPlacement::Around);
    }

    #[test]
    fn aiming_at_a_slot_attends_it_and_only_it() {
        let mut surface = surface();
        let specs = [spec(Role::Action), spec(Role::Action), spec(Role::Action)];
        ring_update(
            &mut surface,
            &specs,
            Centre::Open,
            &frame(Some(aim_at(Vec2::new(0.0, R)))),
        );
        assert_eq!(surface.attended(), Some(0));
        let active = surface
            .views()
            .iter()
            .filter(|view| view.attention.is_active())
            .count();
        assert_eq!(active, 1, "only one mote can be the one you will get");
    }

    #[test]
    fn reach_bounds_exactly_what_resolves() {
        let surface = surface();
        let layout = Layout::orbit(4, Centre::Open, surface.capacity(), R).0;
        assert!(
            layout
                .resolve(
                    Vec2::new(0.0, R * Tuning::DEFAULT.reach_frac * 0.99),
                    None,
                    surface.tuning()
                )
                .is_some()
        );
        assert!(
            layout
                .resolve(
                    Vec2::new(0.0, R * Tuning::DEFAULT.reach_frac * 1.01),
                    None,
                    surface.tuning()
                )
                .is_none(),
            "a binding sizes its hit surface from the reach; anything attended \
             past it is a mote that lights up and cannot be pressed"
        );
    }

    #[test]
    fn a_tap_reports_the_slot_it_started_on() {
        let mut surface = surface();
        let specs = [spec(Role::Action), spec(Role::Action)];
        let aim = aim_at(Vec2::new(0.0, R));
        ring_update(&mut surface, &specs, Centre::Open, &frame(Some(aim)));
        surface.press(aim.world, false);
        ring_update(&mut surface, &specs, Centre::Open, &frame(Some(aim)));
        assert_eq!(surface.release(), Some(Outcome::Tap(0)));
    }

    #[test]
    fn a_held_mote_leaves_its_slot_and_follows_the_hand_in_three_dimensions() {
        let mut surface = surface();
        let specs = [takeable(), spec(Role::Action)];
        let start = aim_at(Vec2::new(0.0, R));
        ring_update(&mut surface, &specs, Centre::Open, &frame(Some(start)));
        let resting = surface.views()[0].position;

        surface.press(start.world, true);
        // Off the orbit plane entirely: a pickup is not a slider.
        let hand = Vec3::new(R * 2.0, -R, 0.6);
        ring_update(
            &mut surface,
            &specs,
            Centre::Open,
            &frame_with_hand(Some(start), hand),
        );

        let held = surface.views()[0];
        assert!(held.seized);
        assert!(
            (held.position - hand).length() < 1.0e-4,
            "the body follows the hand freely, off-plane included"
        );
        assert!((held.position - resting).length() > 0.01);
    }

    #[test]
    fn a_fixed_mote_never_leaves_its_slot() {
        let mut surface = surface();
        let specs = [spec(Role::Action), spec(Role::Action)];
        let start = aim_at(Vec2::new(0.0, R));
        ring_update(&mut surface, &specs, Centre::Open, &frame(Some(start)));
        let resting = surface.views()[0].position;

        surface.press(start.world, true);
        ring_update(
            &mut surface,
            &specs,
            Centre::Open,
            &frame_with_hand(Some(start), Vec3::new(1.0, 1.0, 1.0)),
        );
        assert!(!surface.views()[0].seized);
        assert!((surface.views()[0].position - resting).length() < 0.02);
    }

    #[test]
    fn dragging_does_not_light_up_whatever_it_passes_over() {
        let mut surface = surface();
        let specs = [takeable(), spec(Role::Action), spec(Role::Action)];
        let start = aim_at(Vec2::new(0.0, R));
        ring_update(&mut surface, &specs, Centre::Open, &frame(Some(start)));
        surface.press(start.world, true);

        // Sweep the aim onto a different slot while holding the first.
        let over_another = aim_at(Vec2::new(0.0, -R));
        ring_update(
            &mut surface,
            &specs,
            Centre::Open,
            &frame(Some(over_another)),
        );

        assert_eq!(
            surface.attended(),
            Some(0),
            "attention stays with what is in hand; nothing else is a target"
        );
        assert!(!surface.views()[1].attention.is_active());
        assert!(!surface.views()[2].attention.is_active());
    }

    #[test]
    fn a_fixed_mote_still_tgrids_attention_and_cancels_if_you_slide_off() {
        let mut surface = surface();
        let specs = [spec(Role::Action), spec(Role::Action), spec(Role::Action)];
        let start = aim_at(Vec2::new(0.0, R));
        ring_update(&mut surface, &specs, Centre::Open, &frame(Some(start)));
        surface.press(start.world, true);

        ring_update(
            &mut surface,
            &specs,
            Centre::Open,
            &frame(Some(aim_at(Vec2::new(0.0, -R)))),
        );
        assert_ne!(
            surface.attended(),
            Some(0),
            "a button is not holding anything"
        );
        assert_eq!(
            surface.release(),
            None,
            "releasing off a button cancels rather than activating it"
        );
    }

    #[test]
    fn unheld_motes_stay_in_their_slots_while_another_is_dragged() {
        let mut surface = surface();
        let specs = [takeable(), spec(Role::Action)];
        let start = aim_at(Vec2::new(0.0, R));
        ring_update(&mut surface, &specs, Centre::Open, &frame(Some(start)));
        let other = surface.views()[1].position;

        surface.press(start.world, true);
        ring_update(
            &mut surface,
            &specs,
            Centre::Open,
            &frame_with_hand(Some(start), Vec3::new(R, -R, 0.3)),
        );
        assert!(!surface.views()[1].seized);
        assert!((surface.views()[1].position - other).length() < 0.02);
    }

    #[test]
    fn every_mote_says_where_its_name_goes() {
        let mut surface = surface();
        ring_update(
            &mut surface,
            &[spec(Role::Action)],
            Centre::Open,
            &frame(None),
        );
        let view = surface.views()[0];
        assert!(
            view.label_offset.y < -view.radius,
            "the label clears the body rather than sitting inside it"
        );
        assert!(view.label_size > 0.0);
    }

    #[test]
    fn only_a_mote_that_has_held_attention_earns_a_placard() {
        let mut surface = surface();
        let specs = [spec(Role::Action), spec(Role::Action)];
        let aim = aim_at(Vec2::new(0.0, R));

        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        assert!(surface.placard().is_none(), "nothing is attended");

        ring_update(&mut surface, &specs, Centre::Open, &frame(Some(aim)));
        assert!(surface.placard().is_none(), "the delay has not passed");

        dwell(&mut surface, &specs, aim);
        let placard = surface.placard().expect("attended long enough");
        assert_eq!(placard.lines[0].text, "Citrus", "titled with its own name");
    }

    #[test]
    fn a_mote_told_nothing_is_just_its_name() {
        let mut surface = surface();
        let specs = [spec(Role::Action)];
        dwell(&mut surface, &specs, aim_at(Vec2::new(0.0, R)));

        let placard = surface.placard().expect("placard");
        let texts = placard
            .lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>();
        assert_eq!(texts, vec!["Citrus"]);
    }

    #[test]
    fn grabbing_a_mote_puts_its_placard_away() {
        let mut surface = surface();
        let specs = [takeable(), spec(Role::Action)];
        let aim = aim_at(Vec2::new(0.0, R));
        dwell(&mut surface, &specs, aim);
        assert!(surface.placard().is_some());

        surface.press(aim.world, true);
        ring_update(&mut surface, &specs, Centre::Open, &frame(Some(aim)));
        assert!(
            surface.placard().is_none(),
            "a card riding a mote through the air is unreadable, and by then \
             you are past wanting to be told what it is"
        );
    }

    #[test]
    fn looking_away_takes_the_placard_with_it() {
        let mut surface = surface();
        let specs = [spec(Role::Action), spec(Role::Action)];
        dwell(&mut surface, &specs, aim_at(Vec2::new(0.0, R)));
        assert!(surface.placard().is_some());

        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        assert!(surface.placard().is_none());
    }

    #[test]
    fn the_way_back_and_the_page_tabs_earn_no_placard() {
        for role in [
            Role::Parent { depth: 1 },
            Role::Page {
                forward:   true,
                remaining: 2,
            },
        ] {
            let mut surface = surface();
            let described = MoteSpec {
                description: Some(SmolStr::new_static("Back to Home.")),
                ..spec(role)
            };
            let specs = [described, spec(Role::Action)];
            let aim = aim_at(Vec2::new(0.0, R));
            dwell(&mut surface, &specs, aim);

            assert_eq!(surface.attended(), Some(0), "{role:?} is attended");
            assert!(
                surface.placard().is_none(),
                "{role:?} is read by its chevron"
            );
        }
    }

    #[test]
    fn the_placard_rides_the_body_it_describes() {
        let mut surface = surface();
        let specs = [spec(Role::Action), spec(Role::Action)];
        let aim = aim_at(Vec2::new(0.0, R));
        dwell(&mut surface, &specs, aim);

        let view = surface.views()[0];
        let placard = surface.placard().expect("placard");
        assert!((placard.position.x - view.position.x).abs() < 1.0e-5);
        assert!(
            placard.position.y > view.position.y,
            "mounted above, so it never covers the mote it is about"
        );
    }

    #[test]
    fn an_oversized_level_paginates_rather_than_being_truncated() {
        let mut surface = Surface::new(3, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = (0..10).map(|i| named(&format!("m{i}"))).collect::<Vec<_>>();

        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        assert_eq!(surface.drawn().len(), 3);
        assert_eq!(surface.page().count, 4, "10 motes over 3 slots");
        assert_eq!(surface.page().total, 10);
        assert_eq!(surface.drawn(), vec![0, 1, 2]);

        surface.turn_by(1);
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        assert_eq!(surface.drawn(), vec![3, 4, 5]);

        surface.turn_by(2);
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        assert_eq!(
            surface.drawn(),
            vec![9],
            "a short last page draws what is left rather than wrapping"
        );
        let page = surface.page();
        assert_eq!(page.index + 1, page.count, "on the last page");
    }

    #[test]
    fn a_held_centre_stays_put_across_every_page() {
        let mut surface = Surface::new(3, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = (0..7).map(|i| named(&format!("m{i}"))).collect::<Vec<_>>();

        ring_update(&mut surface, &specs, Centre::Held, &frame(None));
        assert_eq!(surface.drawn(), vec![0, 1, 2]);
        assert_eq!(surface.page().count, 3, "6 children over 2 free slots");

        surface.turn_by(1);
        ring_update(&mut surface, &specs, Centre::Held, &frame(None));
        assert_eq!(
            surface.drawn(),
            vec![0, 3, 4],
            "the way back is not something you can page away from"
        );
    }

    #[test]
    fn turning_the_page_does_not_carry_attention_onto_a_stranger() {
        let mut surface = Surface::new(3, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = (0..9).map(|i| named(&format!("m{i}"))).collect::<Vec<_>>();
        let aim = aim_at(Vec2::new(0.0, R));

        ring_update(&mut surface, &specs, Centre::Open, &frame(Some(aim)));
        assert_eq!(surface.attended(), Some(0));

        surface.turn_by(1);
        assert_eq!(
            surface.attended(),
            None,
            "the slot under the pointer stands for something else now"
        );
    }

    #[test]
    fn paging_past_the_end_clamps_rather_than_emptying_the_ring() {
        let mut surface = Surface::new(3, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = (0..5).map(|i| named(&format!("m{i}"))).collect::<Vec<_>>();
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        surface.turn_by(99);
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        assert_eq!(surface.page().index, 1);
        assert_eq!(surface.drawn(), vec![3, 4]);
    }

    #[test]
    fn a_level_that_fits_is_not_paged() {
        let mut surface = surface();
        let specs = [named("a"), named("b")];
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        assert!(!surface.page().is_paged());
        assert_eq!(surface.page().index, 0);
    }

    #[test]
    fn a_tap_on_a_later_page_reports_the_mote_it_landed_on() {
        let mut surface = Surface::new(3, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = (0..9).map(|i| named(&format!("m{i}"))).collect::<Vec<_>>();
        let aim = aim_at(Vec2::new(0.0, R));

        surface.turn_by(2);
        ring_update(&mut surface, &specs, Centre::Open, &frame(Some(aim)));
        surface.press(aim.world, false);
        ring_update(&mut surface, &specs, Centre::Open, &frame(Some(aim)));

        let Some(Outcome::Tap(slot)) = surface.release() else {
            panic!("a tap");
        };
        assert_eq!(
            surface.spec_index(slot),
            Some(6),
            "a drawn slot is not a collection index once anything paginates"
        );
    }

    #[test]
    fn a_level_draws_no_more_than_its_capacity_and_a_tab_says_there_is_more() {
        let mut surface = Surface::new(3, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = vec![spec(Role::Action); 10];
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        assert_eq!(surface.drawn().len(), 3);
        assert_eq!(
            surface.views().len(),
            3 + 1,
            "the level's three slots, and the tab saying there is more"
        );
    }

    #[test]
    fn a_custom_palette_reaches_every_mote() {
        let custom = crate::palette::rgb(0.0, 0.4, 0.2);
        let mut surface = Surface::new(
            4,
            Tuning::DEFAULT,
            Palette {
                base: custom,
                ..Palette::DEFAULT
            },
        );
        let specs = [spec(Role::Action)];
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        assert_eq!(surface.views()[0].style.color, custom);
    }

    #[test]
    fn a_consumer_s_hue_takes_over_identity_from_the_palette() {
        let tint = crate::palette::rgb(0.8, 0.2, 0.3);
        let mut surface = surface();
        let specs = [MoteSpec {
            tint: Some(tint),
            ..spec(Role::Item { unique: false })
        }];
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        assert_eq!(
            surface.views()[0].style.color,
            tint,
            "the hue replaces the source's warmth: one colour per form"
        );
    }

    #[test]
    fn attention_lifts_a_consumer_s_hue_rather_than_repainting_it() {
        let tint = crate::palette::rgb(0.8, 0.2, 0.3);
        let mut surface = surface();
        let specs = [MoteSpec {
            tint: Some(tint),
            ..spec(Role::Action)
        }];
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        let idle = surface.views()[0].style.color;

        let attended = hold(&mut surface, &specs, 60).style.color;

        assert_ne!(attended, Palette::DEFAULT.accent, "hover is not in hand");
        assert!(
            attended.r > idle.r,
            "a lift is a brightening, not a repaint"
        );
        assert!(attended.g > idle.g);
        assert!(attended.b > idle.b);
    }

    #[test]
    fn hovering_brightens_a_mote_without_repainting_it() {
        let mut surface = surface();
        let specs = [spec(Role::Action), spec(Role::Action)];
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        let resting = surface.views()[0].style;

        ring_update(
            &mut surface,
            &specs,
            Centre::Open,
            &frame(Some(aim_at(Vec2::new(0.0, R)))),
        );
        let hovered = surface.views()[0].style;

        assert_ne!(hovered.color, Palette::DEFAULT.accent, "hover stays quiet");
        assert!(hovered.color.r > resting.color.r);
        assert!(hovered.emissive > resting.emissive);
    }

    /// Holds `aim` until every slot has settled, and reports the sibling in
    /// slot 1.
    fn settle_beside(specs: &[MoteSpec], aim: Aim) -> SlotView {
        let mut surface = surface();
        for _ in 0..60 {
            ring_update(&mut surface, specs, Centre::Open, &frame(Some(aim)));
        }
        surface.views()[1]
    }

    /// The whole of "approach reveals": a mote nobody has chosen still knows
    /// the pointer is coming, and knows it by degree.
    #[test]
    fn a_mote_warms_as_the_pointer_comes_toward_it() {
        let specs = [spec(Role::Action), spec(Role::Action)];
        let across = settle_beside(&specs, aim_at(Vec2::new(0.0, R)));
        let closer = settle_beside(&specs, aim_at(Vec2::new(0.0, R * 0.3)));

        assert!(
            closer.heat > across.heat,
            "the same sibling, under the same attended mote, is warmer for \
             the pointer having come nearer: {} against {}",
            closer.heat,
            across.heat
        );
        assert!(
            closer.heat < Attention::Attended.heat(),
            "warming is not being chosen"
        );
    }

    /// And the other half: the field gives way around what is chosen, rather
    /// than every sibling lighting up because the surface is in use.
    #[test]
    fn attending_one_mote_dims_the_rest_rather_than_lifting_them() {
        let mut surface = surface();
        let specs = [spec(Role::Action), spec(Role::Action)];
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        let resting = surface.views()[1].style.emissive;

        let sibling = settle_beside(&specs, aim_at(Vec2::new(0.0, R)));
        assert!(
            sibling.style.emissive < resting,
            "a sibling receded to {} from a resting {resting}",
            sibling.style.emissive
        );
    }

    #[test]
    fn receding_is_a_dimming_and_never_a_repaint() {
        let surface = surface();
        let alone = surface.style_at(Role::Action, 0.3, false, None, 1.0);
        let beside = surface.style_at(Role::Action, 0.3, false, None, Tuning::DEFAULT.sibling_dim);

        assert!(beside.emissive < alone.emissive);
        assert_eq!(beside.color, alone.color);
        assert!((beside.alpha - alone.alpha).abs() < 1.0e-6);
    }

    /// Holds `aim` on slot 0 for `frames` and reports it.
    fn hold(surface: &mut Surface, specs: &[MoteSpec], frames: usize) -> SlotView {
        let aim = aim_at(Vec2::new(0.0, R));
        for _ in 0..frames {
            ring_update(surface, specs, Centre::Open, &frame(Some(aim)));
        }
        surface.views()[0]
    }

    /// Only the size lags at first: nothing grows between `Idle` and `Near`,
    /// which are the same size by design, so a mote brightens for a frame or
    /// two before it starts to swell. That reads as noticing, and the point
    /// here is only that it never arrives in one step.
    #[test]
    fn a_mote_grows_into_being_attended_rather_than_stepping_to_it() {
        let mut surface = surface();
        let specs = [spec(Role::Action), spec(Role::Action)];
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        let resting = surface.views()[0].radius;

        let early = hold(&mut surface, &specs, 3).radius;
        let later = hold(&mut surface, &specs, 3).radius;
        let settled = hold(&mut surface, &specs, 60).radius;

        assert!(
            resting < early && early < later && later < settled,
            "a body that reaches its attended size in one step reads as a \
             widget lighting up: {resting} -> {early} -> {later} -> {settled}"
        );
    }

    #[test]
    fn a_mote_brightens_from_the_very_first_frame() {
        let mut surface = surface();
        let specs = [spec(Role::Action), spec(Role::Action)];
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        let resting = surface.views()[0].style;

        let first = hold(&mut surface, &specs, 1).style;
        let settled = hold(&mut surface, &specs, 60).style;

        assert!(first.emissive > resting.emissive, "light leads the size");
        assert!(
            first.emissive < settled.emissive,
            "and still has somewhere to go"
        );
    }

    /// Easing changes how a mote gets to a state, never what the state is.
    #[test]
    fn a_settled_mote_looks_exactly_as_its_state_says() {
        let mut surface = surface();
        let specs = [spec(Role::Action), spec(Role::Action)];
        let settled = hold(&mut surface, &specs, 60);

        assert_eq!(settled.attention, Attention::Attended);
        assert_eq!(
            settled.radius,
            mote::present(&specs[0], Attention::Attended.heat(), surface.tuning()).radius
        );
        assert_eq!(
            settled.style,
            surface.style(
                specs[0].role,
                Attention::Attended,
                specs[0].active,
                specs[0].tint
            )
        );
    }

    /// A style is written to the scene only when it differs, so one that never
    /// quite arrives redraws every mote every frame forever.
    #[test]
    fn a_settled_mote_stops_changing() {
        let mut surface = surface();
        let specs = [spec(Role::Action), spec(Role::Action)];
        let settled = hold(&mut surface, &specs, 60);
        let still = hold(&mut surface, &specs, 1);

        assert_eq!(settled.style, still.style);
        assert_eq!(settled.radius, still.radius);
    }

    /// A toggle's fill is its signal: off is a clear outline, on is the mote
    /// wearing its hue. Attention never trades the one for the other.
    #[test]
    fn a_toggle_s_fill_is_its_signal() {
        let surface = surface();
        let palette = Palette::DEFAULT;
        let hue = crate::palette::rgb(0.8, 0.2, 0.3);
        let off = surface.style(Role::Toggle, Attention::Attended, false, Some(hue));
        let on = surface.style(Role::Toggle, Attention::Attended, true, Some(hue));

        assert!(
            off.alpha < on.alpha,
            "off is the outline and on is the fill: {} < {}",
            off.alpha,
            on.alpha
        );
        assert_eq!(
            on.color,
            palette.tinted(hue, Attention::Attended),
            "an on toggle wears its hue"
        );
    }

    #[test]
    fn an_off_toggle_keeps_its_hue_but_sits_back_from_it() {
        let surface = surface();
        let palette = Palette::DEFAULT;
        let hue = crate::palette::rgb(0.9, 0.2, 0.3);
        for attention in [Attention::Idle, Attention::Attended] {
            let off = surface
                .style(Role::Toggle, attention, false, Some(hue))
                .color;
            let on = surface
                .style(Role::Toggle, attention, true, Some(hue))
                .color;

            assert_ne!(off, palette.dim, "not flat grey");
            assert!(off.r > off.g && off.r > off.b, "still that hue: {off:?}");
            if attention == Attention::Idle {
                assert!(lightness(off) < lightness(on), "quieter than lit");
            }
            assert!(
                chroma(off) < chroma(on),
                "{attention:?}: and less saturated than lit"
            );
        }
    }

    #[test]
    fn an_off_toggle_with_no_hue_stays_neutral() {
        let off = surface().style(Role::Toggle, Attention::Idle, false, None);
        assert_eq!(off.color, Palette::DEFAULT.dim);
    }

    #[test]
    fn every_role_is_the_same_glass() {
        let surface = surface();
        let group = Role::Group {
            children: 2,
            groups:   0,
            arrange:  mote::Arrange::Orbit,
        };
        for attention in [Attention::Idle, Attention::Attended] {
            let alpha = surface.style(group, attention, false, None).alpha;
            for role in [
                Role::Action,
                Role::Cast,
                Role::Item { unique: true },
                Role::Parent { depth: 1 },
                Role::Page {
                    forward:   true,
                    remaining: 1,
                },
            ] {
                assert!(
                    (surface.style(role, attention, false, None).alpha - alpha).abs() < 1.0e-6,
                    "{role:?} at {attention:?} is not the glass a group is"
                );
            }
        }
    }

    #[test]
    fn attention_leaving_eases_back_down_and_settles() {
        let mut surface = surface();
        let specs = [spec(Role::Action), spec(Role::Action)];
        let hot = hold(&mut surface, &specs, 60).style.emissive;

        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        let cooling = surface.views()[0].style.emissive;
        let resting = Palette::DEFAULT.emissive(Attention::Idle);
        assert!(
            cooling < hot && cooling > resting,
            "attention leaving is a settle too, not a snap; {hot} -> {cooling}"
        );

        for _ in 0..60 {
            ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        }
        assert_eq!(surface.views()[0].style.emissive, resting);
    }

    #[test]
    fn a_full_grid_paginates_rather_than_dropping_what_it_cannot_show() {
        let mut surface = Surface::new(4, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = (0..9).map(|i| named(&format!("m{i}"))).collect::<Vec<_>>();

        grid_update(&mut surface, &specs, 2, 2);
        assert_eq!(surface.drawn().len(), 4);
        assert_eq!(surface.page().count, 3, "9 items over 4 cells");
        assert_eq!(surface.drawn(), vec![0, 1, 2, 3]);

        surface.turn_by(1);
        grid_update(&mut surface, &specs, 2, 2);
        assert_eq!(surface.drawn(), vec![4, 5, 6, 7]);

        surface.turn_by(1);
        grid_update(&mut surface, &specs, 2, 2);
        assert_eq!(
            surface.drawn(),
            vec![8],
            "the last page is as short as it is"
        );
        let page = surface.page();
        assert_eq!(page.index + 1, page.count, "on the last page");
        assert!(page.index > 0);
    }

    #[test]
    fn an_empty_grid_is_one_empty_page() {
        let mut surface = Surface::new(4, Tuning::DEFAULT, Palette::DEFAULT);
        grid_update(&mut surface, &[], 2, 2);
        assert_eq!(surface.views(), []);
        assert_eq!(surface.page().count, 1);
        assert_eq!(surface.page().total, 0);
        assert!(!surface.page().is_paged());
    }

    fn nested_specs(children: usize) -> Vec<MoteSpec> {
        std::iter::once(spec(Role::Parent { depth: 1 }))
            .chain((0..children).map(|i| named(&format!("m{i}"))))
            .collect()
    }

    /// Steps a nested grid level until its form and motes have all arrived.
    fn settle_nested(surface: &mut Surface, specs: &[MoteSpec], frame: &Frame) {
        let layout = Layout::held_grid(3, 3, PITCH);
        for _ in 0..60 {
            surface.update(specs, layout, 1, frame);
        }
    }

    fn tab_roles(surface: &Surface) -> Vec<Role> {
        surface
            .views()
            .iter()
            .skip(surface.drawn().len())
            .map(|view| view.role)
            .collect()
    }

    #[test]
    fn the_way_back_sits_on_the_anchor_of_a_nested_grid() {
        let mut surface = Surface::new(9, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = nested_specs(5);
        settle_nested(&mut surface, &specs, &frame(None));

        assert_eq!(surface.views()[0].role, Role::Parent { depth: 1 });
        assert_eq!(surface.views()[0].position.truncate(), Vec2::ZERO);
        let first = surface.views()[1].position.truncate();
        assert!(
            (first - Vec2::new(-PITCH.x, PITCH.y)).length() < 1.0e-5,
            "content starts in the top left corner, not beside the way back"
        );
    }

    #[test]
    fn a_nested_grid_pages_eight_to_a_page() {
        let mut surface = Surface::new(9, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = nested_specs(20);
        let layout = Layout::held_grid(3, 3, PITCH);

        surface.update(&specs, layout, 1, &frame(None));
        assert_eq!(surface.page().count, 3, "20 children over 8 cells");
        assert_eq!(surface.drawn(), vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);

        surface.turn_by(1);
        surface.update(&specs, layout, 1, &frame(None));
        assert_eq!(
            surface.drawn(),
            vec![0, 9, 10, 11, 12, 13, 14, 15, 16],
            "the way back stays in the middle of every page"
        );
    }

    #[test]
    fn a_root_orbit_draws_all_of_its_slots_whatever_the_surface_can_hold() {
        let mut surface = Surface::new(9, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = [named("a"), named("b"), named("c")];
        let (layout, pinned) = Layout::orbit(3, Centre::Open, 3, R);
        surface.update(&specs, layout, pinned, &frame(None));

        assert_eq!(surface.drawn(), vec![0, 1, 2]);
        assert_eq!(surface.views().len(), 3, "a root that fits has no tabs");
        assert!(!surface.page().is_paged());
    }

    #[test]
    fn a_level_that_fits_has_no_tabs() {
        let mut surface = surface();
        let specs = [named("a"), named("b")];
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        assert_eq!(tab_roles(&surface), Vec::<Role>::new());
    }

    #[test]
    fn tabs_appear_only_toward_the_pages_that_exist() {
        let mut surface = Surface::new(3, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = (0..9).map(|i| named(&format!("m{i}"))).collect::<Vec<_>>();

        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        assert_eq!(
            tab_roles(&surface),
            vec![Role::Page {
                forward:   true,
                remaining: 2,
            }],
            "nothing is behind the first page"
        );

        surface.turn_by(1);
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        assert_eq!(
            tab_roles(&surface),
            vec![
                Role::Page {
                    forward:   false,
                    remaining: 1,
                },
                Role::Page {
                    forward:   true,
                    remaining: 1,
                }
            ]
        );

        surface.turn_by(1);
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        assert_eq!(
            tab_roles(&surface),
            vec![Role::Page {
                forward:   false,
                remaining: 2,
            }],
            "nothing is ahead of the last page"
        );
    }

    #[test]
    fn a_tab_stands_beside_the_layout_at_rest() {
        let mut surface = Surface::new(3, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = (0..9).map(|i| named(&format!("m{i}"))).collect::<Vec<_>>();
        for _ in 0..60 {
            ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        }
        let (layout, _) = Layout::orbit(9, Centre::Open, 3, R);
        let tab = surface.views()[3];
        assert!((tab.position.truncate() - layout.tab(true, &Tuning::DEFAULT)).length() < 1.0e-5);
        assert!(
            tab.radius < surface.views()[0].radius,
            "smaller than a mote"
        );
    }

    #[test]
    fn pressing_the_forward_tab_turns_the_page_and_activates_nothing() {
        let mut surface = Surface::new(3, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = (0..9).map(|i| named(&format!("m{i}"))).collect::<Vec<_>>();
        let (layout, _) = Layout::orbit(9, Centre::Open, 3, R);
        let tab = layout.tab(true, &Tuning::DEFAULT);
        let aim = aim_at(tab);

        ring_update(&mut surface, &specs, Centre::Open, &frame(Some(aim)));
        let slot = surface.attended().expect("the tab holds attention");
        assert_eq!(slot, 3);
        assert_eq!(surface.spec_index(slot), None, "a tab stands for no mote");
        assert!(surface.views()[slot].attention.is_active());

        surface.press(aim.world, true);
        ring_update(&mut surface, &specs, Centre::Open, &frame(Some(aim)));
        assert_eq!(
            surface.release(),
            Some(Outcome::Tap(slot)),
            "a tab is not takeable, so even a grip is a tap"
        );
        assert!(surface.turn_tab(slot));

        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        assert_eq!(surface.page().index, 1);
        assert_eq!(surface.drawn(), vec![3, 4, 5]);
    }

    #[test]
    fn a_page_turn_leaves_no_warmth_or_lean_on_the_slots() {
        let mut surface = Surface::new(3, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = (0..9).map(|i| named(&format!("m{i}"))).collect::<Vec<_>>();
        let aim = aim_at(Vec2::new(0.0, R));
        for _ in 0..30 {
            ring_update(&mut surface, &specs, Centre::Open, &frame(Some(aim)));
        }
        assert!(surface.views()[0].heat > 0.0);

        surface.turn_by(1);
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        assert!(surface.views().iter().all(|view| view.heat == 0.0));
    }

    fn noticed(label: &str, message: &str) -> MoteSpec {
        MoteSpec {
            notice: Some(SmolStr::new(message)),
            ..named(label)
        }
    }

    #[test]
    fn a_notice_shows_on_its_mote_without_attention() {
        let mut surface = surface();
        let specs = [named("a"), noticed("b", "No home.")];
        for _ in 0..30 {
            ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        }
        assert_eq!(surface.attended(), None);

        let notice = surface.placard().expect("a notice needs no attention");
        assert_eq!(notice.lines[0].text, "No home.");
        assert_eq!(notice.lines[0].emphasis, crate::placard::Emphasis::Warning);
        assert!(
            (notice.position.x - surface.views()[1].position.x).abs() < 1.0e-5,
            "on the body of the mote that posted it"
        );
    }

    #[test]
    fn a_notice_takes_precedence_over_the_hover_placard() {
        let mut surface = surface();
        let specs = [named("a"), noticed("b", "Nope.")];
        dwell(&mut surface, &specs, aim_at(Vec2::new(0.0, R)));

        assert_eq!(surface.attended(), Some(0));
        let shown = surface.placard().expect("placard");
        assert_eq!(shown.lines[0].text, "Nope.");
    }

    #[test]
    fn a_surface_on_its_way_out_shows_no_notice() {
        let mut surface = surface();
        let specs = [named("a"), noticed("b", "No home.")];
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        assert!(surface.placard().is_some());

        surface.set_open(false);
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        assert!(surface.placard().is_none());
    }

    #[test]
    fn a_notice_gives_way_to_a_mote_in_hand() {
        let mut surface = surface();
        let specs = [named("a"), noticed("b", "No home.")];
        let aim = aim_at(Vec2::new(0.0, R));
        ring_update(&mut surface, &specs, Centre::Open, &frame(Some(aim)));
        assert!(surface.placard().is_some());

        surface.press(aim.world, true);
        ring_update(&mut surface, &specs, Centre::Open, &frame(Some(aim)));
        assert!(surface.is_seized());
        assert!(surface.placard().is_none());
    }

    #[test]
    fn a_mote_that_is_not_drawn_shows_no_notice() {
        let mut surface = Surface::new(2, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = [named("a"), named("b"), noticed("c", "Unseen.")];
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        assert_eq!(surface.drawn(), vec![0, 1]);
        assert!(surface.placard().is_none());
    }

    #[test]
    fn a_mote_slot_is_not_a_tab() {
        let mut surface = Surface::new(3, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = (0..9).map(|i| named(&format!("m{i}"))).collect::<Vec<_>>();
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        assert!(!surface.turn_tab(0));
        assert!(!surface.turn_tab(99));
        assert_eq!(surface.page().index, 0);
    }

    #[test]
    fn a_nested_grid_tab_is_reached_beside_its_outer_column() {
        let mut surface = Surface::new(9, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = nested_specs(20);
        let layout = Layout::held_grid(3, 3, PITCH);
        let aim = aim_at(layout.tab(true, &Tuning::DEFAULT));

        surface.update(&specs, layout, 1, &frame(Some(aim)));
        assert_eq!(surface.attended(), Some(9));
        assert_eq!(surface.spec_index(9), None);
    }

    #[test]
    fn a_new_level_starts_from_a_clean_surface_but_an_open_form() {
        let mut surface = Surface::new(9, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = nested_specs(20);
        let layout = Layout::held_grid(3, 3, PITCH);
        let aim = aim_at(layout.slot(1).expect("cell"));

        surface.turn_by(1);
        for _ in 0..60 {
            surface.update(&specs, layout, 1, &frame(Some(aim)));
        }
        assert_eq!(surface.page().index, 1);
        assert!(surface.attended().is_some());
        assert!(surface.views()[surface.attended().expect("attended")].heat > 0.0);

        surface.relevel();
        surface.update(&specs, layout, 1, &frame(None));

        assert_eq!(surface.page().index, 0);
        assert_eq!(surface.attended(), None);
        assert!(surface.views().iter().all(|view| view.heat == 0.0));
        assert_eq!(
            surface.views()[1].position.truncate(),
            layout.slot(1).expect("cell"),
            "the form did not fall back to the anchor"
        );
        let (first, last) = (surface.views()[0].bloom, surface.views()[8].bloom);
        assert!(
            first < 1.0 && last < first,
            "the motes arrive again in turn: {first} then {last}"
        );
    }

    fn lightness(color: Color) -> f32 {
        (color.r + color.g + color.b) / 3.0
    }

    fn chroma(color: Color) -> f32 {
        color.r.max(color.g).max(color.b) - color.r.min(color.g).min(color.b)
    }

    #[test]
    fn a_grab_on_a_later_grid_page_reports_the_item_it_landed_on() {
        let mut surface = Surface::new(4, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = (0..9).map(|i| named(&format!("m{i}"))).collect::<Vec<_>>();
        let layout = Layout::grid(2, 2, PITCH);
        let cell = layout.slot(1).expect("cell");
        let aim = Aim {
            local: cell,
            world: Vec3::new(cell.x, cell.y, 0.0),
        };

        surface.turn_to(1);
        surface.update(&specs, layout, 0, &frame(Some(aim)));
        surface.press(aim.world, true);
        surface.update(&specs, layout, 0, &frame(Some(aim)));

        let Some(Outcome::Tap(slot)) = surface.release() else {
            panic!("a tap");
        };
        assert_eq!(surface.spec_index(slot), Some(5));
    }
}
