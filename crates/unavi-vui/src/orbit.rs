use wired_math::types::Vec3;

use crate::{
    grasp::Outcome,
    layout::Layout,
    mote::MoteSpec,
    palette::Palette,
    placard::PlacardView,
    surface::Surface,
    tuning::Tuning,
    view::{
        Frame,
        PageView,
        SlotView,
        Style,
    },
};

/// What, if anything, holds the middle of a ring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Centre {
    /// Every mote takes a direction; a null deflection selects nothing.
    Open,
    /// The first mote sits in the middle and stays there on every page. The
    /// way back and a level's own subject are both this; which one it is
    /// belongs to whoever supplies the specs.
    Held,
}

/// Motes arranged around an anchor, selected by direction.
pub struct Orbit {
    surface: Surface,
}

impl Orbit {
    #[must_use]
    pub fn new(capacity: usize, tuning: Tuning, palette: Palette) -> Self {
        Self {
            surface: Surface::new(capacity, tuning, palette),
        }
    }

    #[must_use]
    pub const fn tuning(&self) -> &Tuning {
        &self.surface.tuning
    }

    #[must_use]
    pub const fn palette(&self) -> &Palette {
        &self.surface.palette
    }

    /// The ring `count` motes take, shrinking to the level rather than leaving
    /// gaps, and paginating once the level outgrows the pool.
    #[must_use]
    pub fn ring(&self, count: usize, centre: Centre) -> Layout {
        let held = usize::from(centre == Centre::Held).min(count);
        let points = (count - held).min(self.surface.capacity() - held);
        let radius = self.surface.tuning.orbit_radius;
        match centre {
            Centre::Open => Layout::star(points, radius),
            Centre::Held => Layout::centred(points, radius),
        }
    }

    /// How far from the anchor the orbit still answers for. A binding's hit
    /// surface is this size.
    #[must_use]
    pub fn reach(&self) -> f32 {
        self.surface.tuning.orbit_radius * self.surface.tuning.reach_frac
    }

    pub fn update(&mut self, specs: &[MoteSpec], centre: Centre, frame: &Frame) {
        let layout = self.ring(specs.len(), centre);
        let pinned = usize::from(centre == Centre::Held);
        self.surface.update(specs, layout, pinned, frame);
    }

    #[must_use]
    pub fn views(&self) -> &[SlotView] {
        self.surface.views()
    }

    /// The collection index each drawn slot stands for, parallel to
    /// [`Orbit::views`].
    #[must_use]
    pub fn drawn(&self) -> &[usize] {
        self.surface.drawn()
    }

    #[must_use]
    pub fn spec_index(&self, slot: usize) -> Option<usize> {
        self.surface.spec_index(slot)
    }

    #[must_use]
    pub const fn page(&self) -> PageView {
        self.surface.page()
    }

    pub const fn turn_by(&mut self, delta: isize) {
        self.surface.turn_by(delta);
    }

    pub const fn turn_to(&mut self, page: usize) {
        self.surface.turn_to(page);
    }

    #[must_use]
    pub const fn attended(&self) -> Option<usize> {
        self.surface.attended()
    }

    #[must_use]
    pub const fn is_seized(&self) -> bool {
        self.surface.is_seized()
    }

    #[must_use]
    pub fn displaced(&self) -> Option<usize> {
        self.surface.displaced()
    }

    #[must_use]
    pub const fn placard(&self) -> Option<&PlacardView> {
        self.surface.placard()
    }

    #[must_use]
    pub const fn resting_style(&self, spec: &MoteSpec) -> Style {
        self.surface.resting_style(spec)
    }

    pub fn press(&mut self, at: Vec3) {
        self.surface.press(at);
    }

    pub fn release(&mut self) -> Option<Outcome> {
        self.surface.release()
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
        mote::{
            PipPlacement,
            Role,
        },
        view::Aim,
    };

    const R: f32 = Tuning::DEFAULT.orbit_radius;

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

    /// Holds attention past the placard delay.
    fn dwell(orbit: &mut Orbit, specs: &[MoteSpec], aim: Aim) {
        let frame = Frame {
            delta: Tuning::DEFAULT.placard_delay + Tuning::DEFAULT.placard_fade,
            ..frame(Some(aim))
        };
        orbit.update(specs, Centre::Open, &frame);
        orbit.update(specs, Centre::Open, &frame);
    }

    fn takeable() -> MoteSpec {
        spec(Role::Item)
    }

    fn group(children: usize, groups: usize) -> MoteSpec {
        spec(Role::Group { children, groups })
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

    fn orbit() -> Orbit {
        Orbit::new(12, Tuning::DEFAULT, Palette::DEFAULT)
    }

    #[test]
    fn an_open_ring_has_no_centre_and_a_held_one_does() {
        let orbit = orbit();
        assert!(!orbit.ring(4, Centre::Open).has_centre());
        assert!(orbit.ring(5, Centre::Held).has_centre());
        assert_eq!(
            orbit.ring(5, Centre::Held).len(),
            5,
            "the held centre plus 4 around it"
        );
    }

    #[test]
    fn the_centre_holds_the_first_mote_whatever_it_stands_for() {
        let mut orbit = Orbit::new(5, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = [
            named("Hand"),
            named("Home"),
            named("Places"),
            named("Pocket"),
            named("Self"),
        ];
        orbit.update(&specs, Centre::Held, &frame(None));

        assert_eq!(orbit.views().len(), 5);
        assert_eq!(
            orbit.views()[0].position.truncate(),
            Vec2::ZERO,
            "a frozen root can put its own subject in the middle, not only \
             the way back"
        );
        assert_eq!(orbit.spec_index(0), Some(0));
        assert!(!orbit.page().is_paged());
    }

    #[test]
    fn pips_report_the_real_child_count() {
        let mut orbit = orbit();
        orbit.update(&[group(3, 0)], Centre::Open, &frame(None));
        assert_eq!(orbit.views()[0].pips.count, 3);
        assert!(!orbit.views()[0].pips.overflow);
    }

    #[test]
    fn container_children_are_marked_for_see_through_drawing() {
        let mut orbit = orbit();
        orbit.update(&[group(5, 2)], Centre::Open, &frame(None));
        assert_eq!(orbit.views()[0].pips.groups(), 2);
    }

    #[test]
    fn an_oversized_branch_reports_overflow_rather_than_lying() {
        let mut orbit = orbit();
        orbit.update(
            &[group(Tuning::DEFAULT.pip_cap + 5, 0)],
            Centre::Open,
            &frame(None),
        );
        assert_eq!(orbit.views()[0].pips.count, Tuning::DEFAULT.pip_cap);
        assert!(orbit.views()[0].pips.overflow);
    }

    #[test]
    fn leaves_carry_no_pips() {
        let mut orbit = orbit();
        orbit.update(
            &[spec(Role::Action), spec(Role::Cast)],
            Centre::Open,
            &frame(None),
        );
        assert!(orbit.views().iter().all(|view| view.pips.count == 0));
    }

    #[test]
    fn a_branch_reads_bigger_and_more_transparent_than_a_leaf() {
        let mut orbit = orbit();
        orbit.update(
            &[group(2, 0), spec(Role::Action)],
            Centre::Open,
            &frame(None),
        );
        let (branch, leaf) = (orbit.views()[0], orbit.views()[1]);
        assert!(branch.radius > leaf.radius, "containers read as containers");
        assert!(branch.style.alpha < leaf.style.alpha);
    }

    #[test]
    fn the_parent_mote_is_small_dim_and_marked_around_itself() {
        let mut orbit = orbit();
        orbit.update(
            &[spec(Role::Parent { depth: 2 }), spec(Role::Action)],
            Centre::Held,
            &frame(None),
        );
        let parent = orbit.views()[0];
        assert!(parent.radius < orbit.views()[1].radius);
        assert_eq!(parent.style.color, Palette::DEFAULT.dim);
        assert_eq!(parent.pips.count, 2, "depth is legible without text");
        assert_eq!(parent.pips.placement, PipPlacement::Around);
    }

    #[test]
    fn aiming_at_a_slot_attends_it_and_only_it() {
        let mut orbit = orbit();
        let specs = [spec(Role::Action), spec(Role::Action), spec(Role::Action)];
        orbit.update(
            &specs,
            Centre::Open,
            &frame(Some(aim_at(Vec2::new(0.0, R)))),
        );
        assert_eq!(orbit.attended(), Some(0));
        let active = orbit
            .views()
            .iter()
            .filter(|view| view.attention.is_active())
            .count();
        assert_eq!(active, 1, "only one mote can be the one you will get");
    }

    #[test]
    fn reach_bounds_exactly_what_resolves() {
        let orbit = orbit();
        let layout = orbit.ring(4, Centre::Open);
        assert!(
            layout
                .resolve(Vec2::new(0.0, orbit.reach() * 0.99), None, orbit.tuning())
                .is_some()
        );
        assert!(
            layout
                .resolve(Vec2::new(0.0, orbit.reach() * 1.01), None, orbit.tuning())
                .is_none(),
            "a binding sizes its hit surface from reach(); anything attended \
             past it is a mote that lights up and cannot be pressed"
        );
    }

    #[test]
    fn a_tap_reports_the_slot_it_started_on() {
        let mut orbit = orbit();
        let specs = [spec(Role::Action), spec(Role::Action)];
        let aim = aim_at(Vec2::new(0.0, R));
        orbit.update(&specs, Centre::Open, &frame(Some(aim)));
        orbit.press(aim.world);
        orbit.update(&specs, Centre::Open, &frame(Some(aim)));
        assert_eq!(orbit.release(), Some(Outcome::Tap(0)));
    }

    #[test]
    fn a_held_mote_leaves_its_slot_and_follows_the_hand_in_three_dimensions() {
        let mut orbit = orbit();
        let specs = [takeable(), spec(Role::Action)];
        let start = aim_at(Vec2::new(0.0, R));
        orbit.update(&specs, Centre::Open, &frame(Some(start)));
        let resting = orbit.views()[0].position;

        orbit.press(start.world);
        // Off the orbit plane entirely: a pickup is not a slider.
        let hand = Vec3::new(R * 2.0, -R, 0.6);
        orbit.update(&specs, Centre::Open, &frame_with_hand(Some(start), hand));

        let held = orbit.views()[0];
        assert!(held.seized);
        assert!(
            (held.position - hand).length() < 1.0e-4,
            "the body follows the hand freely, off-plane included"
        );
        assert!((held.position - resting).length() > 0.01);
    }

    #[test]
    fn a_fixed_mote_never_leaves_its_slot() {
        let mut orbit = orbit();
        let specs = [spec(Role::Action), spec(Role::Action)];
        let start = aim_at(Vec2::new(0.0, R));
        orbit.update(&specs, Centre::Open, &frame(Some(start)));
        let resting = orbit.views()[0].position;

        orbit.press(start.world);
        orbit.update(
            &specs,
            Centre::Open,
            &frame_with_hand(Some(start), Vec3::new(1.0, 1.0, 1.0)),
        );
        assert!(!orbit.views()[0].seized);
        assert!((orbit.views()[0].position - resting).length() < 0.02);
    }

    #[test]
    fn dragging_does_not_light_up_whatever_it_passes_over() {
        let mut orbit = orbit();
        let specs = [takeable(), spec(Role::Action), spec(Role::Action)];
        let start = aim_at(Vec2::new(0.0, R));
        orbit.update(&specs, Centre::Open, &frame(Some(start)));
        orbit.press(start.world);

        // Sweep the aim onto a different slot while holding the first.
        let over_another = aim_at(Vec2::new(0.0, -R));
        orbit.update(&specs, Centre::Open, &frame(Some(over_another)));

        assert_eq!(
            orbit.attended(),
            Some(0),
            "attention stays with what is in hand; nothing else is a target"
        );
        assert!(!orbit.views()[1].attention.is_active());
        assert!(!orbit.views()[2].attention.is_active());
    }

    #[test]
    fn a_fixed_mote_still_tracks_attention_and_cancels_if_you_slide_off() {
        let mut orbit = orbit();
        let specs = [spec(Role::Action), spec(Role::Action), spec(Role::Action)];
        let start = aim_at(Vec2::new(0.0, R));
        orbit.update(&specs, Centre::Open, &frame(Some(start)));
        orbit.press(start.world);

        orbit.update(
            &specs,
            Centre::Open,
            &frame(Some(aim_at(Vec2::new(0.0, -R)))),
        );
        assert_ne!(
            orbit.attended(),
            Some(0),
            "a button is not holding anything"
        );
        assert_eq!(
            orbit.release(),
            None,
            "releasing off a button cancels rather than activating it"
        );
    }

    #[test]
    fn unheld_motes_stay_in_their_slots_while_another_is_dragged() {
        let mut orbit = orbit();
        let specs = [takeable(), spec(Role::Action)];
        let start = aim_at(Vec2::new(0.0, R));
        orbit.update(&specs, Centre::Open, &frame(Some(start)));
        let other = orbit.views()[1].position;

        orbit.press(start.world);
        orbit.update(
            &specs,
            Centre::Open,
            &frame_with_hand(Some(start), Vec3::new(R, -R, 0.3)),
        );
        assert!(!orbit.views()[1].seized);
        assert!((orbit.views()[1].position - other).length() < 0.02);
    }

    #[test]
    fn every_mote_says_where_its_name_goes() {
        let mut orbit = orbit();
        orbit.update(&[spec(Role::Action)], Centre::Open, &frame(None));
        let view = orbit.views()[0];
        assert!(
            view.label_offset.y < -view.radius,
            "the label clears the body rather than sitting inside it"
        );
        assert!(view.label_size > 0.0);
    }

    #[test]
    fn only_a_mote_that_has_held_attention_earns_a_placard() {
        let mut orbit = orbit();
        let specs = [spec(Role::Action), spec(Role::Action)];
        let aim = aim_at(Vec2::new(0.0, R));

        orbit.update(&specs, Centre::Open, &frame(None));
        assert!(orbit.placard().is_none(), "nothing is attended");

        orbit.update(&specs, Centre::Open, &frame(Some(aim)));
        assert!(orbit.placard().is_none(), "the delay has not passed");

        dwell(&mut orbit, &specs, aim);
        let placard = orbit.placard().expect("attended long enough");
        assert!(placard.opacity > 0.0);
        assert_eq!(placard.lines[0].text, "Citrus", "titled with its own name");
    }

    #[test]
    fn a_mote_told_nothing_still_says_how_to_use_it() {
        let mut orbit = orbit();
        let specs = [spec(Role::Action)];
        dwell(&mut orbit, &specs, aim_at(Vec2::new(0.0, R)));

        let placard = orbit.placard().expect("placard");
        let texts = placard
            .lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>();
        assert_eq!(texts, vec!["Citrus", "Grab to activate"]);
    }

    #[test]
    fn grabbing_a_mote_puts_its_placard_away() {
        let mut orbit = orbit();
        let specs = [takeable(), spec(Role::Action)];
        let aim = aim_at(Vec2::new(0.0, R));
        dwell(&mut orbit, &specs, aim);
        assert!(orbit.placard().is_some());

        orbit.press(aim.world);
        orbit.update(&specs, Centre::Open, &frame(Some(aim)));
        assert!(
            orbit.placard().is_none(),
            "a card riding a mote through the air is unreadable, and by then \
             you are past wanting to be told what it is"
        );
    }

    #[test]
    fn looking_away_takes_the_placard_with_it() {
        let mut orbit = orbit();
        let specs = [spec(Role::Action), spec(Role::Action)];
        dwell(&mut orbit, &specs, aim_at(Vec2::new(0.0, R)));
        assert!(orbit.placard().is_some());

        orbit.update(&specs, Centre::Open, &frame(None));
        assert!(orbit.placard().is_none());
    }

    #[test]
    fn the_placard_rides_the_body_it_describes() {
        let mut orbit = orbit();
        let specs = [spec(Role::Action), spec(Role::Action)];
        let aim = aim_at(Vec2::new(0.0, R));
        dwell(&mut orbit, &specs, aim);

        let view = orbit.views()[0];
        let placard = orbit.placard().expect("placard");
        assert!((placard.position.x - view.position.x).abs() < 1.0e-5);
        assert!(
            placard.position.y > view.position.y,
            "mounted above, so it never covers the mote it is about"
        );
    }

    #[test]
    fn an_oversized_level_paginates_rather_than_being_truncated() {
        let mut orbit = Orbit::new(3, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = (0..10).map(|i| named(&format!("m{i}"))).collect::<Vec<_>>();

        orbit.update(&specs, Centre::Open, &frame(None));
        assert_eq!(orbit.views().len(), 3);
        assert_eq!(orbit.page().count, 4, "10 motes over 3 slots");
        assert_eq!(orbit.page().total, 10);
        assert_eq!(orbit.drawn(), vec![0, 1, 2]);

        orbit.turn_by(1);
        orbit.update(&specs, Centre::Open, &frame(None));
        assert_eq!(orbit.drawn(), vec![3, 4, 5]);

        orbit.turn_by(2);
        orbit.update(&specs, Centre::Open, &frame(None));
        assert_eq!(
            orbit.drawn(),
            vec![9],
            "a short last page draws what is left rather than wrapping"
        );
        assert!(!orbit.page().has_next());
    }

    #[test]
    fn a_held_centre_stays_put_across_every_page() {
        let mut orbit = Orbit::new(3, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = (0..7).map(|i| named(&format!("m{i}"))).collect::<Vec<_>>();

        orbit.update(&specs, Centre::Held, &frame(None));
        assert_eq!(orbit.drawn(), vec![0, 1, 2]);
        assert_eq!(orbit.page().count, 3, "6 children over 2 free slots");

        orbit.turn_by(1);
        orbit.update(&specs, Centre::Held, &frame(None));
        assert_eq!(
            orbit.drawn(),
            vec![0, 3, 4],
            "the way back is not something you can page away from"
        );
    }

    #[test]
    fn turning_the_page_does_not_carry_attention_onto_a_stranger() {
        let mut orbit = Orbit::new(3, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = (0..9).map(|i| named(&format!("m{i}"))).collect::<Vec<_>>();
        let aim = aim_at(Vec2::new(0.0, R));

        orbit.update(&specs, Centre::Open, &frame(Some(aim)));
        assert_eq!(orbit.attended(), Some(0));

        orbit.turn_by(1);
        assert_eq!(
            orbit.attended(),
            None,
            "the slot under the pointer stands for something else now"
        );
    }

    #[test]
    fn paging_past_the_end_clamps_rather_than_emptying_the_ring() {
        let mut orbit = Orbit::new(3, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = (0..5).map(|i| named(&format!("m{i}"))).collect::<Vec<_>>();
        orbit.update(&specs, Centre::Open, &frame(None));
        orbit.turn_by(99);
        orbit.update(&specs, Centre::Open, &frame(None));
        assert_eq!(orbit.page().index, 1);
        assert_eq!(orbit.drawn(), vec![3, 4]);
    }

    #[test]
    fn a_level_that_fits_is_not_paged() {
        let mut orbit = orbit();
        let specs = [named("a"), named("b")];
        orbit.update(&specs, Centre::Open, &frame(None));
        assert!(!orbit.page().is_paged());
        assert!(!orbit.page().has_next());
        assert!(!orbit.page().has_previous());
    }

    #[test]
    fn a_tap_on_a_later_page_reports_the_mote_it_landed_on() {
        let mut orbit = Orbit::new(3, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = (0..9).map(|i| named(&format!("m{i}"))).collect::<Vec<_>>();
        let aim = aim_at(Vec2::new(0.0, R));

        orbit.turn_by(2);
        orbit.update(&specs, Centre::Open, &frame(Some(aim)));
        orbit.press(aim.world);
        orbit.update(&specs, Centre::Open, &frame(Some(aim)));

        let Some(Outcome::Tap(slot)) = orbit.release() else {
            panic!("a tap");
        };
        assert_eq!(
            orbit.spec_index(slot),
            Some(6),
            "a drawn slot is not a collection index once anything paginates"
        );
    }

    #[test]
    fn views_never_exceed_capacity() {
        let mut orbit = Orbit::new(3, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = vec![spec(Role::Action); 10];
        orbit.update(&specs, Centre::Open, &frame(None));
        assert_eq!(orbit.views().len(), 3);
    }

    #[test]
    fn a_custom_palette_reaches_every_mote() {
        let custom = crate::palette::rgb(0.0, 0.4, 0.2);
        let mut orbit = Orbit::new(
            4,
            Tuning::DEFAULT,
            Palette {
                base: custom,
                ..Palette::DEFAULT
            },
        );
        let specs = [spec(Role::Action)];
        orbit.update(&specs, Centre::Open, &frame(None));
        assert_eq!(orbit.views()[0].style.color, custom);
    }

    #[test]
    fn hovering_brightens_a_mote_without_repainting_it() {
        let mut orbit = orbit();
        let specs = [spec(Role::Action), spec(Role::Action)];
        orbit.update(&specs, Centre::Open, &frame(None));
        let resting = orbit.views()[0].style;

        orbit.update(
            &specs,
            Centre::Open,
            &frame(Some(aim_at(Vec2::new(0.0, R)))),
        );
        let hovered = orbit.views()[0].style;

        assert_ne!(hovered.color, Palette::DEFAULT.accent, "hover stays quiet");
        assert!(hovered.color.r > resting.color.r);
        assert!(hovered.emissive > resting.emissive);
    }
}
