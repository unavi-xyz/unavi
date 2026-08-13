use wired_math::types::Vec3;

use crate::tuning::Tuning;

/// A mote currently in hand.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Seized {
    pub slot:      usize,
    pub origin:    Vec3,
    pub at:        Vec3,
    /// Set once the pointer leaves the tap threshold, and never cleared.
    pub displaced: bool,
    /// A fixed mote never displaces, however far the pointer travels.
    pub takeable:  bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Released without meaningful travel; fires the mote's primary action.
    Tap(usize),
    /// Released after travel; the mote is already in the engine's hands.
    Place(usize),
}

#[derive(Debug, Default)]
pub struct Grasp {
    seized: Option<Seized>,
}

impl Grasp {
    #[must_use]
    pub const fn new() -> Self {
        Self { seized: None }
    }

    pub const fn press(&mut self, slot: usize, at: Vec3, takeable: bool) {
        self.seized = Some(Seized {
            slot,
            origin: at,
            at,
            displaced: false,
            takeable,
        });
    }

    pub fn track(&mut self, at: Vec3, tuning: &Tuning) {
        let Some(seized) = &mut self.seized else {
            return;
        };
        seized.at = at;
        if seized.takeable && (at - seized.origin).length() > tuning.seize_threshold {
            seized.displaced = true;
        }
    }

    pub fn release(&mut self) -> Option<Outcome> {
        let seized = self.seized.take()?;
        Some(if seized.displaced {
            Outcome::Place(seized.slot)
        } else {
            Outcome::Tap(seized.slot)
        })
    }

    #[must_use]
    pub const fn seized(&self) -> Option<&Seized> {
        self.seized.as_ref()
    }

    #[must_use]
    pub const fn is_seized(&self) -> bool {
        self.seized.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tuning() -> Tuning {
        Tuning::DEFAULT
    }

    fn beyond_threshold() -> Vec3 {
        Vec3::new(tuning().seize_threshold * 2.0, 0.0, 0.0)
    }

    #[test]
    fn a_release_without_travel_is_a_tap() {
        let mut grasp = Grasp::new();
        grasp.press(3, Vec3::ZERO, true);
        grasp.track(Vec3::new(0.001, 0.0, 0.0), &tuning());
        assert_eq!(grasp.release(), Some(Outcome::Tap(3)));
    }

    #[test]
    fn a_release_after_travel_is_a_place() {
        let mut grasp = Grasp::new();
        grasp.press(1, Vec3::ZERO, true);
        grasp.track(beyond_threshold(), &tuning());
        assert_eq!(grasp.release(), Some(Outcome::Place(1)));
    }

    #[test]
    fn returning_to_the_origin_cancels_rather_than_taps() {
        let mut grasp = Grasp::new();
        grasp.press(2, Vec3::ZERO, true);
        grasp.track(beyond_threshold(), &tuning());
        grasp.track(Vec3::ZERO, &tuning());

        assert_eq!(
            grasp.release(),
            Some(Outcome::Place(2)),
            "a mote dragged out and back is a place, which the caller cancels \
             against where it actually ended up — not a tap that fires the \
             action"
        );
    }

    #[test]
    fn a_fixed_mote_never_displaces_however_far_the_pointer_goes() {
        let mut grasp = Grasp::new();
        grasp.press(1, Vec3::ZERO, false);
        grasp.track(Vec3::new(5.0, 5.0, 5.0), &tuning());
        assert!(!grasp.seized().expect("held").displaced);
        assert_eq!(
            grasp.release(),
            Some(Outcome::Tap(1)),
            "a mote that cannot be taken has no drag to resolve"
        );
    }

    #[test]
    fn releasing_without_a_press_does_nothing() {
        let mut grasp = Grasp::new();
        assert_eq!(grasp.release(), None);
    }

    #[test]
    fn tracking_without_a_press_does_nothing() {
        let mut grasp = Grasp::new();
        grasp.track(beyond_threshold(), &tuning());
        assert!(!grasp.is_seized());
    }

    #[test]
    fn a_press_replaces_any_previous_hold() {
        let mut grasp = Grasp::new();
        grasp.press(1, Vec3::ZERO, true);
        grasp.press(2, Vec3::ZERO, true);
        assert_eq!(grasp.seized().map(|s| s.slot), Some(2));
    }
}
