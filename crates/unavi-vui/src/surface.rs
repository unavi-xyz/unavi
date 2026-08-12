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
    tuning:   Tuning,
    palette:  Palette,
    tracker:  Tracker,
    grasp:    Grasp,
    lean:     Vec<Vec3>,
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
    pub const fn tuning(&self) -> &Tuning {
        &self.tuning
    }

    #[must_use]
    pub const fn palette(&self) -> &Palette {
        &self.palette
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
        let color = match role {
            Role::Parent { .. } if !attention.is_active() => self.palette.dim,
            Role::Item { .. } => self.palette.item(attention, role.is_source()),
            _ => self.palette.tint(attention),
        };
        Style {
            color,
            alpha: match role {
                Role::Group { .. } => self.palette.glass(attention),
                Role::Action | Role::Item { .. } | Role::Cast | Role::Parent { .. } => {
                    self.palette.solid_alpha
                }
            },
            emissive: self.palette.emissive(attention),
        }
    }
}

#[cfg(test)]
mod tests {
    use smol_str::SmolStr;
    use wired_math::types::{
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
        }
    }

    fn named(label: &str) -> MoteSpec {
        MoteSpec {
            role:        Role::Action,
            label:       SmolStr::new(label),
            description: None,
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
            eye: Vec3::new(0.0, 0.0, 1.0),
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
            delta: Tuning::DEFAULT.placard_delay + Tuning::DEFAULT.placard_fade,
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
    fn a_branch_reads_bigger_and_more_transparent_than_a_leaf() {
        let mut surface = surface();
        ring_update(
            &mut surface,
            &[group(2, 0), spec(Role::Action)],
            Centre::Open,
            &frame(None),
        );
        let (branch, leaf) = (surface.views()[0], surface.views()[1]);
        assert!(branch.radius > leaf.radius, "containers read as containers");
        assert!(branch.style.alpha < leaf.style.alpha);
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
        surface.press(aim.world);
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

        surface.press(start.world);
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

        surface.press(start.world);
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
        surface.press(start.world);

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
        surface.press(start.world);

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

        surface.press(start.world);
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
        assert!(placard.opacity > 0.0);
        assert_eq!(placard.lines[0].text, "Citrus", "titled with its own name");
    }

    #[test]
    fn a_mote_told_nothing_still_says_what_it_is() {
        let mut surface = surface();
        let specs = [spec(Role::Action)];
        dwell(&mut surface, &specs, aim_at(Vec2::new(0.0, R)));

        let placard = surface.placard().expect("placard");
        let texts = placard
            .lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>();
        assert_eq!(texts, vec!["Citrus", "action"]);
    }

    #[test]
    fn grabbing_a_mote_puts_its_placard_away() {
        let mut surface = surface();
        let specs = [takeable(), spec(Role::Action)];
        let aim = aim_at(Vec2::new(0.0, R));
        dwell(&mut surface, &specs, aim);
        assert!(surface.placard().is_some());

        surface.press(aim.world);
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
        assert_eq!(surface.views().len(), 3);
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
        assert!(!surface.page().has_next());
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
        assert!(!surface.page().has_next());
        assert!(!surface.page().has_previous());
    }

    #[test]
    fn a_tap_on_a_later_page_reports_the_mote_it_landed_on() {
        let mut surface = Surface::new(3, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = (0..9).map(|i| named(&format!("m{i}"))).collect::<Vec<_>>();
        let aim = aim_at(Vec2::new(0.0, R));

        surface.turn_by(2);
        ring_update(&mut surface, &specs, Centre::Open, &frame(Some(aim)));
        surface.press(aim.world);
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
    fn views_never_exceed_capacity() {
        let mut surface = Surface::new(3, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = vec![spec(Role::Action); 10];
        ring_update(&mut surface, &specs, Centre::Open, &frame(None));
        assert_eq!(surface.views().len(), 3);
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

    #[test]
    fn a_full_grid_paginates_rather_than_dropping_what_it_cannot_show() {
        let mut surface = Surface::new(4, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = (0..9).map(|i| named(&format!("m{i}"))).collect::<Vec<_>>();

        grid_update(&mut surface, &specs, 2, 2);
        assert_eq!(surface.views().len(), 4);
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
        assert!(!surface.page().has_next());
        assert!(surface.page().has_previous());
    }

    #[test]
    fn an_empty_grid_is_one_empty_page() {
        let mut surface = Surface::new(4, Tuning::DEFAULT, Palette::DEFAULT);
        grid_update(&mut surface, &[], 2, 2);
        assert!(surface.views().is_empty());
        assert_eq!(surface.page().count, 1);
        assert_eq!(surface.page().total, 0);
        assert!(!surface.page().is_paged());
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
        surface.press(aim.world);
        surface.update(&specs, layout, 0, &frame(Some(aim)));

        let Some(Outcome::Tap(slot)) = surface.release() else {
            panic!("a tap");
        };
        assert_eq!(surface.spec_index(slot), Some(5));
    }
}
