#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Attention {
    #[default]
    Idle,
    /// As warm as a mote gets without being chosen. Heat climbs toward this
    /// by how near the pointer has come; the tracker never reports it,
    /// because being approached is a matter of degree and this is only where
    /// it tops out.
    Near,
    /// The best candidate for a pointer.
    Attended,
    /// Grasped.
    Engaged,
}

impl Attention {
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Attended | Self::Engaged)
    }

    /// Where this state sits on the continuous scale a surface eases along.
    ///
    /// Attention is graded, so what a mote *reads* as has to be too: a body
    /// that jumps to its attended size the frame a pointer crosses it reads as
    /// a widget lighting up, where one that arrives there reads as a thing
    /// noticing. The state itself stays discrete — targeting, grasp and the
    /// event stream all turn on which mote is attended, not on how far.
    #[must_use]
    pub const fn heat(self) -> f32 {
        match self {
            Self::Idle => 0.0,
            Self::Near => 0.22,
            Self::Attended => 0.62,
            Self::Engaged => 1.0,
        }
    }
}

/// The two states `heat` falls between, and how far between them it is.
///
/// Everything attention-driven is sampled through this rather than
/// interpolated field by field, so a state's own values stay exactly what
/// [`Attention`] says they are at its own heat, and role-specific rules — a
/// parent mote receding, a container's glass, an item's warmth — keep working
/// without any of them learning about heat.
#[must_use]
pub fn bracket(heat: f32) -> (Attention, Attention, f32) {
    const STEPS: [Attention; 4] = [
        Attention::Idle,
        Attention::Near,
        Attention::Attended,
        Attention::Engaged,
    ];

    let heat = heat.clamp(0.0, 1.0);
    for pair in STEPS.windows(2) {
        let (low, high) = (pair[0], pair[1]);
        if heat <= high.heat() {
            let span = high.heat() - low.heat();
            let t = if span > f32::EPSILON {
                (heat - low.heat()) / span
            } else {
                0.0
            };
            return (low, high, t.clamp(0.0, 1.0));
        }
    }
    (Attention::Engaged, Attention::Engaged, 0.0)
}

/// Holds which slot has attention and how long it has held it.
#[derive(Debug, Default)]
pub struct Tracker {
    current: Option<usize>,
    dwell:   f32,
}

impl Tracker {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            current: None,
            dwell:   0.0,
        }
    }

    /// Advances by `delta` seconds, adopting `candidate` as the attended slot.
    pub fn update(&mut self, candidate: Option<usize>, delta: f32) -> Option<usize> {
        if candidate == self.current {
            self.dwell += delta;
        } else {
            self.current = candidate;
            self.dwell = 0.0;
        }
        self.current
    }

    #[must_use]
    pub const fn current(&self) -> Option<usize> {
        self.current
    }

    #[must_use]
    pub const fn dwell(&self) -> f32 {
        self.dwell
    }

    /// Only the attended slot has a state of its own. Everything else rests,
    /// and warms by proximity rather than by anything the tracker knows —
    /// which is what keeps a sibling from being told it is half-selected
    /// merely because something else was.
    #[must_use]
    pub fn state(&self, slot: usize, engaged: bool) -> Attention {
        if self.current == Some(slot) {
            if engaged {
                Attention::Engaged
            } else {
                Attention::Attended
            }
        } else {
            Attention::Idle
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dwell_starts_at_zero_when_attention_is_acquired() {
        let mut tracker = Tracker::new();
        tracker.update(Some(2), 0.1);
        assert!(tracker.dwell().abs() < 1.0e-5);
    }

    #[test]
    fn dwell_accumulates_while_the_slot_holds() {
        let mut tracker = Tracker::new();
        tracker.update(Some(2), 0.1);
        tracker.update(Some(2), 0.1);
        tracker.update(Some(2), 0.1);
        assert!((tracker.dwell() - 0.2).abs() < 1.0e-5);
    }

    #[test]
    fn dwell_resets_when_attention_moves() {
        let mut tracker = Tracker::new();
        tracker.update(Some(2), 0.3);
        tracker.update(Some(3), 0.1);
        assert_eq!(tracker.current(), Some(3));
        assert!(tracker.dwell().abs() < 1.0e-5);
    }

    /// The dwell this accumulates is what a placard fades in against
    /// (`placard::opacity`).
    #[test]
    fn a_mote_reacts_before_it_has_dwelt_at_all() {
        let mut tracker = Tracker::new();
        tracker.update(Some(1), 0.1);
        assert_eq!(tracker.state(1, false), Attention::Attended);
        assert!(tracker.dwell().abs() < 1.0e-5);
    }

    #[test]
    fn only_the_attended_slot_is_active() {
        let mut tracker = Tracker::new();
        tracker.update(Some(1), 0.5);
        assert!(tracker.state(1, false).is_active());
        assert!(!tracker.state(2, false).is_active());
    }

    /// A sibling rests. It is warmed by how near the pointer is, which the
    /// tracker knows nothing about — being next to the chosen one is not
    /// itself a kind of attention.
    #[test]
    fn a_slot_that_is_not_attended_is_idle_whatever_else_is_going_on() {
        let mut tracker = Tracker::new();
        tracker.update(Some(1), 0.5);
        assert_eq!(tracker.state(2, false), Attention::Idle);
        assert_eq!(tracker.state(2, true), Attention::Idle);
    }

    #[test]
    fn engagement_only_applies_to_the_attended_slot() {
        let mut tracker = Tracker::new();
        tracker.update(Some(1), 0.5);
        assert_eq!(tracker.state(1, true), Attention::Engaged);
        assert_eq!(tracker.state(2, true), Attention::Idle);
    }

    const STATES: [Attention; 4] = [
        Attention::Idle,
        Attention::Near,
        Attention::Attended,
        Attention::Engaged,
    ];

    #[test]
    fn heat_rises_with_attention_and_spans_the_whole_scale() {
        for pair in STATES.windows(2) {
            assert!(
                pair[0].heat() < pair[1].heat(),
                "{:?} must sit below {:?}",
                pair[0],
                pair[1]
            );
        }
        assert!(Attention::Idle.heat().abs() < f32::EPSILON);
        assert!((Attention::Engaged.heat() - 1.0).abs() < f32::EPSILON);
    }

    /// The property everything downstream leans on: at a state's own heat the
    /// bracket collapses onto that state, so sampling a settled mote returns
    /// exactly what the state says and nothing is rewritten.
    #[test]
    fn a_states_own_heat_brackets_onto_that_state() {
        for state in STATES {
            let (low, high, t) = bracket(state.heat());
            let landed = if t >= 1.0 { high } else { low };
            assert_eq!(landed, state, "{state:?} did not land on itself");
            assert!(t <= 0.0 || t >= 1.0, "{state:?} landed mid-blend at {t}");
        }
    }

    #[test]
    fn a_heat_between_two_states_brackets_between_them() {
        let (low, high, t) = bracket(0.42);
        assert_eq!((low, high), (Attention::Near, Attention::Attended));
        assert!(t > 0.0 && t < 1.0);
    }

    #[test]
    fn heat_outside_the_scale_is_clamped_rather_than_extrapolated() {
        for (heat, expected) in [(-5.0, Attention::Idle), (5.0, Attention::Engaged)] {
            let (low, high, t) = bracket(heat);
            let landed = if t >= 1.0 { high } else { low };
            assert_eq!(landed, expected);
        }
    }
}
