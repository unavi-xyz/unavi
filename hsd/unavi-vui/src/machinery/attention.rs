//! Which slot has attention.

use crate::tokens::state::Attention;

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
}
