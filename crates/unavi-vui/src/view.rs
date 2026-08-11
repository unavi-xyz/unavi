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
        Grab,
        MoteKind,
        MoteSpec,
        Pips,
        Role,
    },
    palette::Palette,
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
    pub position:   Vec3,
    pub radius:     f32,
    /// Collider radius: steady across a hover animation, so a binding writes
    /// it only when the mote's role changes.
    pub hit_radius: f32,
    pub style:      Style,
    pub role:       Role,
    pub grab:       Grab,
    pub attention:  Attention,
    pub pips:       Pips,
    pub detail:     f32,
    /// Taken: the body has left its slot and is following the hand. A merely
    /// pressed mote is not this — see [`Attention::Engaged`].
    pub seized:     bool,
}

/// Where the pointer is, in the orbit plane's own coordinates and in the
/// world. Producing it is the binding's job, since only it knows how the
/// platform aims.
#[derive(Debug, Clone, Copy)]
pub struct Aim {
    pub local: Vec2,
    pub world: Vec3,
}

#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub eye:    Vec3,
    pub anchor: Transform,
    /// Resolves which slot is targeted, and nothing else.
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

    #[must_use]
    pub fn placard_visible(&self) -> bool {
        self.tracker.placard_visible(&self.tuning)
    }

    #[must_use]
    pub fn views(&self) -> &[SlotView] {
        &self.views
    }

    /// A mote's style with no attention on it. What a body handed to the
    /// world should wear — it is no longer the one you are pointing at.
    #[must_use]
    pub const fn resting_style(&self, spec: &MoteSpec) -> Style {
        self.style(spec.role, spec.kind, Attention::Idle)
    }

    pub fn press(&mut self, at: Vec3) {
        if let Some(slot) = self.tracker.current() {
            self.press_slot(slot, at);
        }
    }

    /// Presses a slot the host reported directly, rather than one inferred
    /// from aim.
    pub fn press_slot(&mut self, slot: usize, at: Vec3) {
        let takeable = self
            .views
            .get(slot)
            .is_some_and(|view| matches!(view.grab, Grab::Takeable));
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
            self.grasp.track(hand, frame.delta, &self.tuning);
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

            let distance = (world - frame.eye).length();
            let presentation = mote::present(spec, distance, attention, &self.tuning);

            self.views.push(SlotView {
                position,
                radius: presentation.radius,
                hit_radius: presentation.hit_radius,
                style: self.style(spec.role, spec.kind, attention),
                role: spec.role,
                grab: spec.grab,
                attention,
                pips: presentation.pips,
                detail: presentation.detail,
                seized: is_seized && dragging,
            });
        }
    }

    const fn style(&self, role: Role, kind: MoteKind, attention: Attention) -> Style {
        let color = if matches!(role, Role::Parent { .. }) && !attention.is_active() {
            self.palette.dim
        } else {
            self.palette.tint(kind, attention)
        };
        Style {
            color,
            // A container is see-through because you are meant to see into
            // it; a leaf is solid because there is nothing inside to look at.
            // This is the branch/leaf distinction at any distance, before
            // attention and without hovering.
            alpha: match role {
                Role::Branch { .. } => self.palette.glass(attention),
                Role::Leaf | Role::Cast | Role::Parent { .. } => self.palette.solid_alpha,
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
            kind: MoteKind::Folder,
            role,
            label: SmolStr::new_static("x"),
            grab: Grab::Fixed,
            embodied: false,
        }
    }

    fn takeable(role: Role) -> MoteSpec {
        MoteSpec {
            grab: Grab::Takeable,
            ..spec(role)
        }
    }

    fn branch(children: usize, folders: usize) -> MoteSpec {
        spec(Role::Branch { children, folders })
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
        orbit.update(&[branch(3, 0)], false, &frame(None));
        assert_eq!(orbit.views()[0].pips.count, 3);
        assert!(!orbit.views()[0].pips.overflow);
    }

    #[test]
    fn container_children_are_marked_for_see_through_drawing() {
        let mut orbit = orbit();
        orbit.update(&[branch(5, 2)], false, &frame(None));
        assert_eq!(orbit.views()[0].pips.branches(), 2);
    }

    #[test]
    fn an_oversized_branch_reports_overflow_rather_than_lying() {
        let mut orbit = orbit();
        orbit.update(
            &[branch(Tuning::DEFAULT.pip_cap + 5, 0)],
            false,
            &frame(None),
        );
        assert_eq!(orbit.views()[0].pips.count, Tuning::DEFAULT.pip_cap);
        assert!(orbit.views()[0].pips.overflow);
    }

    #[test]
    fn leaves_carry_no_pips() {
        let mut orbit = orbit();
        orbit.update(&[spec(Role::Leaf), spec(Role::Cast)], false, &frame(None));
        assert!(orbit.views().iter().all(|view| view.pips.count == 0));
    }

    #[test]
    fn a_branch_reads_bigger_and_more_transparent_than_a_leaf() {
        let mut orbit = orbit();
        orbit.update(&[branch(2, 0), spec(Role::Leaf)], false, &frame(None));
        let (branch, leaf) = (orbit.views()[0], orbit.views()[1]);
        assert!(branch.radius > leaf.radius, "containers read as containers");
        assert!(branch.style.alpha < leaf.style.alpha);
    }

    #[test]
    fn the_parent_mote_is_small_dim_and_marked_around_itself() {
        let mut orbit = orbit();
        orbit.update(
            &[spec(Role::Parent { depth: 2 }), spec(Role::Leaf)],
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
        let specs = [spec(Role::Leaf), spec(Role::Leaf), spec(Role::Leaf)];
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
    fn a_tap_reports_the_slot_it_started_on() {
        let mut orbit = orbit();
        let specs = [spec(Role::Leaf), spec(Role::Leaf)];
        let aim = aim_at(Vec2::new(0.0, R));
        orbit.update(&specs, false, &frame(Some(aim)));
        orbit.press(aim.world);
        orbit.update(&specs, false, &frame(Some(aim)));
        assert_eq!(orbit.release(), Some(Outcome::Tap(0)));
    }

    #[test]
    fn a_held_mote_leaves_its_slot_and_follows_the_hand_in_three_dimensions() {
        let mut orbit = orbit();
        let specs = [takeable(Role::Leaf), spec(Role::Leaf)];
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
        let specs = [spec(Role::Leaf), spec(Role::Leaf)];
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
        let specs = [takeable(Role::Leaf), spec(Role::Leaf), spec(Role::Leaf)];
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
        let specs = [spec(Role::Leaf), spec(Role::Leaf), spec(Role::Leaf)];
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
        let specs = [takeable(Role::Leaf), spec(Role::Leaf)];
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
    fn views_never_exceed_capacity() {
        let mut orbit = Orbit::new(3, Tuning::DEFAULT, Palette::DEFAULT);
        let specs = vec![spec(Role::Leaf); 10];
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
                kinds: [custom; MoteKind::COUNT],
                ..Palette::DEFAULT
            },
        );
        let specs = [spec(Role::Leaf)];
        orbit.update(&specs, false, &frame(None));
        assert_eq!(orbit.views()[0].style.color, custom);
    }

    #[test]
    fn hovering_brightens_a_mote_without_repainting_it() {
        let mut orbit = orbit();
        let specs = [spec(Role::Leaf), spec(Role::Leaf)];
        orbit.update(&specs, false, &frame(None));
        let resting = orbit.views()[0].style;

        orbit.update(&specs, false, &frame(Some(aim_at(Vec2::new(0.0, R)))));
        let hovered = orbit.views()[0].style;

        assert_ne!(hovered.color, Palette::DEFAULT.accent, "hover stays quiet");
        assert!(hovered.color.r > resting.color.r);
        assert!(hovered.emissive > resting.emissive);
    }
}
