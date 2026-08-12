use std::f32::consts::TAU;

use arrayvec::ArrayVec;

/// Deflections a sigil may carry. Past this, eyes-free recall stops being
/// reliable at any breadth.
pub const MAX_DEFLECTIONS: usize = 4;

/// Steps a sigil records. Centre selections cost no deflection, so a sigil
/// can be longer than it is deep.
pub const MAX_STEPS: usize = 8;

/// The fixed directional semantics of a four-point level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinal {
    /// Return, home, the fixed point.
    Up,
    /// Outward — the world, places, others.
    Right,
    /// What is carried, held, at hand below.
    Down,
    /// Inward — self, identity, state.
    Left,
}

/// One selection along the way to a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    /// Slot within the level, as its layout orders it.
    pub slot:    usize,
    /// Directions the level offers, excluding any centre.
    pub points:  usize,
    /// Whether the level reserves a centre, making slot 0 a null deflection.
    pub centred: bool,
}

impl Step {
    #[must_use]
    pub const fn centre() -> Self {
        Self {
            slot:    0,
            points:  0,
            centred: true,
        }
    }

    /// A centre selection requires no deflection, which makes it the single
    /// most reliable target there is.
    #[must_use]
    pub const fn is_centre(&self) -> bool {
        self.centred && self.slot == 0
    }

    /// Position within the ring, ignoring any centre slot.
    #[must_use]
    pub const fn ring_slot(&self) -> usize {
        self.slot - if self.centred { 1 } else { 0 }
    }

    /// The direction to deflect, measured like slot 0 of a ring: up, advancing
    /// clockwise. `None` for the centre.
    #[must_use]
    pub fn angle(&self) -> Option<f32> {
        (!self.is_centre() && self.points > 0)
            .then(|| self.ring_slot() as f32 * TAU / self.points as f32)
    }

    /// Named only where the count makes the names true.
    #[must_use]
    pub const fn cardinal(&self) -> Option<Cardinal> {
        if self.is_centre() || self.points != 4 {
            return None;
        }
        match self.ring_slot() {
            0 => Some(Cardinal::Up),
            1 => Some(Cardinal::Right),
            2 => Some(Cardinal::Down),
            3 => Some(Cardinal::Left),
            _ => None,
        }
    }
}

/// Deflections a level of `points` directions can carry beneath it before
/// eyes-free accuracy falls off.
///
/// Kurtenbach & Buxton's numbers, as a table rather than a formula: the curve
/// they measured is empirical, and six sits in its dead zone.
#[must_use]
pub const fn budget(points: usize) -> usize {
    match points {
        0..=4 => 4,
        5..=6 => 3,
        7..=8 => 2,
        _ => 1,
    }
}

/// The directional sequence naming a command.
///
/// Derived from where the command sits, never authored, so it cannot drift out
/// of sync with the tree and nobody maintains a shortcut table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sigil {
    steps: ArrayVec<Step, MAX_STEPS>,
}

impl Sigil {
    /// The sigil reaching a command, or `None` when the path outruns its
    /// breadth's budget.
    ///
    /// A command with no sigil is reachable by navigation like any other; the
    /// UI says so rather than offering an unreliable one.
    #[must_use]
    pub fn for_path(steps: &[Step]) -> Option<Self> {
        if steps.len() > MAX_STEPS {
            return None;
        }
        let deflections = steps.iter().filter(|step| !step.is_centre()).count();
        let budget = steps
            .iter()
            .filter(|step| !step.is_centre())
            .map(|step| budget(step.points))
            .min()
            .unwrap_or(MAX_DEFLECTIONS);

        (deflections <= budget.min(MAX_DEFLECTIONS)).then(|| Self {
            steps: steps.iter().copied().collect(),
        })
    }

    #[must_use]
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// How many deflections the motion actually costs.
    #[must_use]
    pub fn deflections(&self) -> usize {
        self.steps.iter().filter(|step| !step.is_centre()).count()
    }

    /// The cardinals this sigil traces, when every deflecting level is a
    /// four-point one.
    #[must_use]
    pub fn cardinals(&self) -> Option<ArrayVec<Cardinal, MAX_DEFLECTIONS>> {
        self.steps
            .iter()
            .filter(|step| !step.is_centre())
            .map(Step::cardinal)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(slot: usize, points: usize, centred: bool) -> Step {
        Step {
            slot,
            points,
            centred,
        }
    }

    fn cross(slot: usize) -> Step {
        step(slot, 4, true)
    }

    #[test]
    fn the_budget_table_matches_the_measured_curve() {
        assert_eq!(budget(4), 4, "256 commands");
        assert_eq!(budget(5), 3, "125");
        assert_eq!(budget(7), 2, "49");
        assert_eq!(budget(12), 1, "12");
    }

    #[test]
    fn six_sits_in_the_dead_zone_rather_than_getting_its_own_row() {
        assert_eq!(budget(6), budget(5) );
        assert!(budget(6) < budget(4), "too broad for reliable depth");
    }

    #[test]
    fn a_short_path_at_breadth_four_has_a_sigil() {
        let sigil = Sigil::for_path(&[cross(1), cross(2), cross(3), cross(4)]).expect("sigil");
        assert_eq!(sigil.deflections(), 4);
    }

    #[test]
    fn a_path_past_its_budget_has_none() {
        let deep = [cross(1), cross(2), cross(3), cross(4), cross(1)];
        assert_eq!(
            Sigil::for_path(&deep),
            None,
            "reachable by navigation, but it carries no sigil and says so"
        );
    }

    #[test]
    fn a_wide_level_tightens_the_budget_for_the_whole_path() {
        let wide = step(3, 7, false);
        assert!(Sigil::for_path(&[wide, cross(1)]).is_some(), "two deep");
        assert_eq!(
            Sigil::for_path(&[wide, cross(1), cross(2)]),
            None,
            "the tightest level on the path is the one that binds"
        );
    }

    #[test]
    fn the_centre_is_free() {
        let steps = [
            Step::centre(),
            cross(1),
            Step::centre(),
            cross(2),
            cross(3),
            cross(4),
        ];
        let sigil = Sigil::for_path(&steps).expect("centres cost no deflection");
        assert_eq!(sigil.deflections(), 4);
        assert_eq!(sigil.steps().len(), 6);
    }

    #[test]
    fn a_cross_names_its_directions() {
        assert_eq!(cross(1).cardinal(), Some(Cardinal::Up));
        assert_eq!(cross(2).cardinal(), Some(Cardinal::Right));
        assert_eq!(cross(3).cardinal(), Some(Cardinal::Down));
        assert_eq!(cross(4).cardinal(), Some(Cardinal::Left));
        assert_eq!(Step::centre().cardinal(), None, "no deflection to name");
    }

    #[test]
    fn a_star_of_four_names_the_same_directions_one_slot_earlier() {
        assert_eq!(step(0, 4, false).cardinal(), Some(Cardinal::Up));
        assert_eq!(step(1, 4, false).cardinal(), Some(Cardinal::Right));
    }

    #[test]
    fn a_level_that_is_not_a_cross_has_no_names() {
        assert_eq!(step(0, 5, false).cardinal(), None);
        assert_eq!(step(2, 7, false).cardinal(), None);
    }

    #[test]
    fn a_sigil_over_crosses_reads_as_cardinals() {
        let sigil = Sigil::for_path(&[Step::centre(), cross(2)]).expect("sigil");
        assert_eq!(
            sigil.cardinals().expect("named").as_slice(),
            [Cardinal::Right],
            "hand then one, the shape the halo's centre buys"
        );
    }

    #[test]
    fn a_sigil_crossing_an_unnamed_level_has_no_cardinals() {
        let sigil = Sigil::for_path(&[step(2, 7, false)]).expect("sigil");
        assert_eq!(sigil.cardinals(), None);
    }

    #[test]
    fn angles_run_clockwise_from_up() {
        let up = cross(1).angle().expect("angle");
        let right = cross(2).angle().expect("angle");
        assert!(up.abs() < 1.0e-6);
        assert!((right - TAU / 4.0).abs() < 1.0e-6);
        assert_eq!(Step::centre().angle(), None);
    }

    #[test]
    fn an_empty_path_is_the_root_itself() {
        let sigil = Sigil::for_path(&[]).expect("the root needs no motion");
        assert_eq!(sigil.deflections(), 0);
    }
}
