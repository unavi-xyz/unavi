use wired_math::types::{
    Transform,
    Vec2,
    Vec3,
};
use wired_scene::types::Color;

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
        Pips,
        Role,
    },
    palette::Palette,
    placard::{
        self,
        Placard,
        PlacardView,
    },
    tuning::Tuning,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub color:    Color,
    pub alpha:    f32,
    pub emissive: f32,
}

/// Everything a renderer needs for one slot. Deliberately concrete values
/// rather than state to interpret, so a binding is a transcription and two
/// bindings cannot drift apart.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlotView {
    /// Orbit-local, lean already applied.
    pub position:     Vec3,
    pub radius:       f32,
    pub style:        Style,
    pub role:         Role,
    pub attention:    Attention,
    pub pips:         Pips,
    /// Taken: the body has left its slot and is following the hand. A merely
    /// pressed mote is not this — see [`Attention::Engaged`].
    pub seized:       bool,
    /// Where this mote's name goes, slot-local. Labels are drawn always
    /// rather than on attention: a menu you have to hover to read is a menu
    /// you cannot learn. The text itself comes from the spec — it is passed
    /// through, not computed, and keeping it out of here keeps a `SlotView`
    /// `Copy`.
    pub label_offset: Vec3,
    pub label_size:   f32,
}

/// Where the pointer is, in the orbit's own coordinates and in the world.
/// [`crate::pointer::aim`] produces it from a ray; a platform that aims some
/// other way supplies its own.
#[derive(Debug, Clone, Copy)]
pub struct Aim {
    pub local: Vec2,
    pub world: Vec3,
}

#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub eye:    Vec3,
    pub anchor: Transform,
    /// Resolves which slot is targeted, and what the attended one leans
    /// toward. Never where a held mote goes — that is [`Frame::hand`].
    pub aim:    Option<Aim>,
    /// Free world-space grab point. A held mote follows this rather than
    /// [`Frame::aim`], so a pickup is not confined to the orbit's plane.
    pub hand:   Option<Vec3>,
    pub delta:  f32,
}

/// One orbit's live state: what has attention, what is held, and where every
/// body should currently be drawn.
pub struct Orbit {
    pub tuning:  Tuning,
    pub palette: Palette,
    tracker:     Tracker,
    grasp:       Grasp,
    lean:        Vec<Vec3>,
    views:       Vec<SlotView>,
    placard:     Option<PlacardView>,
}

impl Orbit {
    #[must_use]
    pub fn new(capacity: usize, tuning: Tuning, palette: Palette) -> Self {
        Self {
            tuning,
            palette,
            tracker: Tracker::new(),
            grasp: Grasp::new(),
            lean: vec![Vec3::ZERO; capacity],
            views: Vec::with_capacity(capacity),
            placard: None,
        }
    }

    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.lean.len()
    }

    /// A ring of `count` slots, gaining a centre only when nested — where the
    /// parent mote goes. The centre is never a child, so the way back is the
    /// one position that means the same thing at every level.
    #[must_use]
    pub const fn layout(&self, count: usize, nested: bool) -> Layout {
        if nested {
            Layout::centred(count.saturating_sub(1), self.tuning.orbit_radius)
        } else {
            Layout::star(count, self.tuning.orbit_radius)
        }
    }

    #[must_use]
    pub const fn attended(&self) -> Option<usize> {
        self.tracker.current()
    }

    /// How far from the anchor an orbit still answers for.
    ///
    /// A binding's hit surface is this size, and that is not a coincidence:
    /// what lights up and what a press lands on have to be the same region or
    /// the interface promises motes it will not give you.
    #[must_use]
    pub fn reach(&self) -> f32 {
        self.tuning.orbit_radius * self.tuning.reach_frac
    }

    #[must_use]
    pub const fn is_seized(&self) -> bool {
        self.grasp.is_seized()
    }

    /// The held slot, once the pointer has travelled far enough that the hold
    /// is a take rather than a tap. The moment a binding can hand the mote to
    /// whatever moves real objects.
    #[must_use]
    pub fn displaced(&self) -> Option<usize> {
        self.grasp
            .seized()
            .filter(|held| held.displaced)
            .map(|held| held.slot)
    }

    /// The attended mote's placard, or `None` while nothing has held
    /// attention long enough to have earned one.
    #[must_use]
    pub const fn placard(&self) -> Option<&PlacardView> {
        self.placard.as_ref()
    }

    #[must_use]
    pub fn views(&self) -> &[SlotView] {
        &self.views
    }

    /// A mote's style with no attention on it. What a body handed to the
    /// world should wear — it is no longer the one you are pointing at.
    #[must_use]
    pub const fn resting_style(&self, spec: &MoteSpec) -> Style {
        self.style(spec.role, Attention::Idle)
    }

    /// Presses whatever currently holds attention, which is the only thing a
    /// press can mean: the lit mote is the one being pointed at.
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

    /// A fixed mote behaves like a button: releasing after the pointer has
    /// wandered off it cancels rather than activating.
    pub fn release(&mut self) -> Option<Outcome> {
        let wandered = self
            .grasp
            .seized()
            .is_some_and(|held| !held.takeable && self.tracker.current() != Some(held.slot));
        let outcome = self.grasp.release();
        if wandered { None } else { outcome }
    }

    pub fn update(&mut self, specs: &[MoteSpec], nested: bool, frame: &Frame) {
        let count = specs.len().min(self.capacity());
        let layout = self.layout(count, nested);

        // A mote in hand holds attention; nothing else is a drop target.
        let dragging = self.grasp.seized().is_some_and(|held| held.takeable);
        if dragging {
            self.tracker
                .update(self.grasp.seized().map(|held| held.slot), frame.delta);
        } else {
            let candidate = frame
                .aim
                .and_then(|aim| layout.resolve(aim.local, self.tracker.current(), &self.tuning));
            self.tracker.update(candidate, frame.delta);
        }

        if let (Some(hand), true) = (frame.hand, self.grasp.is_seized()) {
            self.grasp.track(hand, &self.tuning);
        }

        let seized = self.grasp.seized().map(|held| held.slot);
        let attended = self.tracker.current();

        self.views.clear();
        for (index, spec) in specs.iter().take(count).enumerate() {
            let Some(plane) = layout.slot(index) else {
                continue;
            };
            let local = Vec3::new(plane.x, plane.y, 0.0);
            let world = frame.anchor.translation + frame.anchor.rotation * local;
            let is_seized = seized == Some(index);

            let attention =
                self.tracker
                    .state(index, is_seized, attended.is_some_and(|slot| slot != index));

            let target = frame.aim.map_or(Vec3::ZERO, |aim| {
                frame.anchor.rotation.inverse()
                    * assist::lean(world, aim.world, attention, &self.tuning)
            });
            self.lean[index] = assist::approach(
                self.lean[index],
                target,
                self.tuning.lean_speed,
                frame.delta,
            );

            let position = match (is_seized && dragging, frame.hand) {
                (true, Some(hand)) => {
                    frame.anchor.rotation.inverse() * (hand - frame.anchor.translation)
                }
                _ => local + self.lean[index],
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

    /// The attended mote's placard, mounted on wherever its body currently is
    /// — including the hand, when it is being carried.
    fn build_placard(&self, specs: &[MoteSpec]) -> Option<PlacardView> {
        // Nothing explains itself while it is being handled. A card riding a
        // mote through the air is unreadable anyway, and by the time you have
        // grabbed something you are past wanting to be told what it is.
        if self.grasp.is_seized() {
            return None;
        }
        let slot = self.tracker.current()?;
        let spec = specs.get(slot)?;
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
            // A container is see-through because you are meant to see into
            // it; a leaf is solid because there is nothing inside to look at.
            // This is the branch/leaf distinction at any distance, before
            // attention and without hovering.
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

#[cfg(test)]
mod tests {
    use smol_str::SmolStr;
    use wired_math::types::Quat;

    use super::*;
    use crate::mote::PipPlacement;

    const R: f32 = Tuning::DEFAULT.orbit_radius;

    fn spec(role: Role) -> MoteSpec {
        MoteSpec {
            role,
            label: SmolStr::new_static("Citrus"),
            description: None,
        }
    }

    /// Holds attention past the placard delay.
    fn dwell(orbit: &mut Orbit, specs: &[MoteSpec], aim: Aim) {
        let frame = Frame {
            delta: Tuning::DEFAULT.placard_delay + Tuning::DEFAULT.placard_fade,
            ..frame(Some(aim))
        };
        orbit.update(specs, false, &frame);
        orbit.update(specs, false, &frame);
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
    fn a_root_orbit_has_no_centre_and_a_nested_one_does() {
        let orbit = orbit();
        assert!(!orbit.layout(4, false).has_centre());
        assert!(orbit.layout(5, true).has_centre());
        assert_eq!(
            orbit.layout(5, true).len(),
            5,
            "parent mote plus 4 children"
        );
    }

    #[test]
    fn pips_report_the_real_child_count() {
        let mut orbit = orbit();
        orbit.update(&[group(3, 0)], false, &frame(None));
        assert_eq!(orbit.views()[0].pips.count, 3);
        assert!(!orbit.views()[0].pips.overflow);
    }

    #[test]
    fn container_children_are_marked_for_see_through_drawing() {
        let mut orbit = orbit();
        orbit.update(&[group(5, 2)], false, &frame(None));
        assert_eq!(orbit.views()[0].pips.groups(), 2);
    }

    #[test]
    fn an_oversized_branch_reports_overflow_rather_than_lying() {
        let mut orbit = orbit();
        orbit.update(
            &[group(Tuning::DEFAULT.pip_cap + 5, 0)],
            false,
            &frame(None),
        );
        assert_eq!(orbit.views()[0].pips.count, Tuning::DEFAULT.pip_cap);
        assert!(orbit.views()[0].pips.overflow);
    }

    #[test]
    fn leaves_carry_no_pips() {
        let mut orbit = orbit();
        orbit.update(&[spec(Role::Action), spec(Role::Cast)], false, &frame(None));
        assert!(orbit.views().iter().all(|view| view.pips.count == 0));
    }

    #[test]
    fn a_branch_reads_bigger_and_more_transparent_than_a_leaf() {
        let mut orbit = orbit();
        orbit.update(&[group(2, 0), spec(Role::Action)], false, &frame(None));
        let (branch, leaf) = (orbit.views()[0], orbit.views()[1]);
        assert!(branch.radius > leaf.radius, "containers read as containers");
        assert!(branch.style.alpha < leaf.style.alpha);
    }

    #[test]
    fn the_parent_mote_is_small_dim_and_marked_around_itself() {
        let mut orbit = orbit();
        orbit.update(
            &[spec(Role::Parent { depth: 2 }), spec(Role::Action)],
            true,
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
        orbit.update(&specs, false, &frame(Some(aim_at(Vec2::new(0.0, R)))));
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
        let layout = orbit.layout(4, false);
        assert!(
            layout
                .resolve(Vec2::new(0.0, orbit.reach() * 0.99), None, &orbit.tuning)
                .is_some()
        );
        assert!(
            layout
                .resolve(Vec2::new(0.0, orbit.reach() * 1.01), None, &orbit.tuning)
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
        orbit.update(&specs, false, &frame(Some(aim)));
        orbit.press(aim.world);
        orbit.update(&specs, false, &frame(Some(aim)));
        assert_eq!(orbit.release(), Some(Outcome::Tap(0)));
    }

    #[test]
    fn a_held_mote_leaves_its_slot_and_follows_the_hand_in_three_dimensions() {
        let mut orbit = orbit();
        let specs = [takeable(), spec(Role::Action)];
        let start = aim_at(Vec2::new(0.0, R));
        orbit.update(&specs, false, &frame(Some(start)));
        let resting = orbit.views()[0].position;

        orbit.press(start.world);
        // Off the orbit plane entirely: a pickup is not a slider.
        let hand = Vec3::new(R * 2.0, -R, 0.6);
        orbit.update(&specs, false, &frame_with_hand(Some(start), hand));

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
        orbit.update(&specs, false, &frame(Some(start)));
        let resting = orbit.views()[0].position;

        orbit.press(start.world);
        orbit.update(
            &specs,
            false,
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
        orbit.update(&specs, false, &frame(Some(start)));
        orbit.press(start.world);

        // Sweep the aim onto a different slot while holding the first.
        let over_another = aim_at(Vec2::new(0.0, -R));
        orbit.update(&specs, false, &frame(Some(over_another)));

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
        orbit.update(&specs, false, &frame(Some(start)));
        orbit.press(start.world);

        orbit.update(&specs, false, &frame(Some(aim_at(Vec2::new(0.0, -R)))));
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
        orbit.update(&specs, false, &frame(Some(start)));
        let other = orbit.views()[1].position;

        orbit.press(start.world);
        orbit.update(
            &specs,
            false,
            &frame_with_hand(Some(start), Vec3::new(R, -R, 0.3)),
        );
        assert!(!orbit.views()[1].seized);
        assert!((orbit.views()[1].position - other).length() < 0.02);
    }

    #[test]
    fn every_mote_says_where_its_name_goes() {
        let mut orbit = orbit();
        orbit.update(&[spec(Role::Action)], false, &frame(None));
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

        orbit.update(&specs, false, &frame(None));
        assert!(orbit.placard().is_none(), "nothing is attended");

        orbit.update(&specs, false, &frame(Some(aim)));
        assert!(orbit.placard().is_none(), "the delay has not passed");

        dwell(&mut orbit, &specs, aim);
        let placard = orbit.placard().expect("attended long enough");
        assert!(placard.opacity > 0.0);
        assert_eq!(placard.lines[0].text, "Citrus", "titled with its own name");
    }

    /// Every mote explains itself, so a consumer cannot ship one that stays
    /// silent when you hold your aim on it.
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
        orbit.update(&specs, false, &frame(Some(aim)));
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

        orbit.update(&specs, false, &frame(None));
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
    fn views_never_exceed_capacity() {
        let mut orbit = Orbit::new(3, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = vec![spec(Role::Action); 10];
        orbit.update(&specs, false, &frame(None));
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
        orbit.update(&specs, false, &frame(None));
        assert_eq!(orbit.views()[0].style.color, custom);
    }

    #[test]
    fn hovering_brightens_a_mote_without_repainting_it() {
        let mut orbit = orbit();
        let specs = [spec(Role::Action), spec(Role::Action)];
        orbit.update(&specs, false, &frame(None));
        let resting = orbit.views()[0].style;

        orbit.update(&specs, false, &frame(Some(aim_at(Vec2::new(0.0, R)))));
        let hovered = orbit.views()[0].style;

        assert_ne!(hovered.color, Palette::DEFAULT.accent, "hover stays quiet");
        assert!(hovered.color.r > resting.color.r);
        assert!(hovered.emissive > resting.emissive);
    }
}
