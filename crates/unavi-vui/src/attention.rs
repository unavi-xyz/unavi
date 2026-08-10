use crate::tuning::Tuning;

/// Graded approach. Nothing commits on attention: approach reveals, contact
/// commits.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Attention {
    #[default]
    Idle,
    /// A pointer is within influence, but this is not the best candidate.
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
}

/// Holds which slot has attention and how long it has held it, so a placard
/// can wait without the mote's own reaction waiting with it.
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

    /// Whether the attended slot has held attention long enough to earn its
    /// placard.
    #[must_use]
    pub fn placard_visible(&self, tuning: &Tuning) -> bool {
        self.current.is_some() && self.dwell >= tuning.placard_delay
    }

    #[must_use]
    pub fn state(&self, slot: usize, engaged: bool, near: bool) -> Attention {
        if self.current == Some(slot) {
            if engaged {
                Attention::Engaged
            } else {
                Attention::Attended
            }
        } else if near {
            Attention::Near
        } else {
            Attention::Idle
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tuning() -> Tuning {
        Tuning::DEFAULT
    }

    #[test]
    fn dwell_starts_at_zero_when_attention_is_acquired() {
        let mut tracker = Tracker::new();
        tracker.update(Some(2), 0.1);
        assert!(tracker.dwell().abs() < 1e-5);
    }

    #[test]
    fn dwell_accumulates_while_the_slot_holds() {
        let mut tracker = Tracker::new();
        tracker.update(Some(2), 0.1);
        tracker.update(Some(2), 0.1);
        tracker.update(Some(2), 0.1);
        assert!((tracker.dwell() - 0.2).abs() < 1e-5);
    }

    #[test]
    fn dwell_resets_when_attention_moves() {
        let mut tracker = Tracker::new();
        tracker.update(Some(2), 0.3);
        tracker.update(Some(3), 0.1);
        assert_eq!(tracker.current(), Some(3));
        assert!(tracker.dwell().abs() < 1e-5);
    }

    #[test]
    fn the_placard_waits_but_the_mote_does_not() {
        let mut tracker = Tracker::new();
        tracker.update(Some(1), 0.1);
        assert_eq!(tracker.state(1, false, false), Attention::Attended);
        assert!(!tracker.placard_visible(&tuning()));

        tracker.update(Some(1), tuning().placard_delay);
        assert!(tracker.placard_visible(&tuning()));
    }

    #[test]
    fn losing_attention_hides_the_placard() {
        let mut tracker = Tracker::new();
        tracker.update(Some(1), 1.0);
        tracker.update(Some(1), 1.0);
        assert!(tracker.placard_visible(&tuning()));
        tracker.update(None, 1.0);
        assert!(!tracker.placard_visible(&tuning()));
    }

    #[test]
    fn only_the_attended_slot_is_active() {
        let mut tracker = Tracker::new();
        tracker.update(Some(1), 0.5);
        assert!(tracker.state(1, false, false).is_active());
        assert!(!tracker.state(2, false, true).is_active());
        assert_eq!(tracker.state(2, false, true), Attention::Near);
        assert_eq!(tracker.state(2, false, false), Attention::Idle);
    }

    #[test]
    fn engagement_only_applies_to_the_attended_slot() {
        let mut tracker = Tracker::new();
        tracker.update(Some(1), 0.5);
        assert_eq!(tracker.state(1, true, false), Attention::Engaged);
        assert_eq!(tracker.state(2, true, false), Attention::Idle);
    }
}
