use wired_math::types::Vec3;

use crate::tuning::Tuning;

/// A mote currently in hand. Nothing has committed: returning to `origin`
/// and releasing is always an abort.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Seized {
    pub slot:   usize,
    pub origin: Vec3,
    pub at:     Vec3,
    /// Set once the pointer has left the tap threshold, and never cleared —
    /// a release back at the origin is a cancelled place, not a tap.
    pub displaced: bool,
    pub velocity:  Vec3,
    /// A fixed mote never displaces however far the pointer travels, so it
    /// cannot be dragged out of its orbit.
    pub takeable:  bool,
}

/// What a release meant. Displacement decides, not a timer: a timer has to be
/// learned, and this is the same rule as click-versus-drag.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Outcome {
    /// Released without meaningful travel — fire the mote's primary action.
    Tap(usize),
    /// Released after travel. Where it landed decides what that means, which
    /// is the caller's to resolve against the world.
    Place {
        slot:     usize,
        at:       Vec3,
        velocity: Vec3,
    },
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
            velocity: Vec3::ZERO,
            takeable,
        });
    }

    /// Advances the held mote's tracking by `delta` seconds.
    pub fn track(&mut self, at: Vec3, delta: f32, tuning: &Tuning) {
        let Some(seized) = &mut self.seized else {
            return;
        };
        if delta > f32::EPSILON {
            let instant = (at - seized.at) / delta;
            let blend = (delta * tuning.lean_speed).clamp(0.0, 1.0);
            seized.velocity = seized.velocity.lerp(instant, blend);
        }
        seized.at = at;
        if seized.takeable && (at - seized.origin).length() > tuning.seize_threshold {
            seized.displaced = true;
        }
    }

    pub fn release(&mut self) -> Option<Outcome> {
        let seized = self.seized.take()?;
        if seized.displaced {
            Some(Outcome::Place {
                slot:     seized.slot,
                at:       seized.at,
                velocity: seized.velocity,
            })
        } else {
            Some(Outcome::Tap(seized.slot))
        }
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
        grasp.track(Vec3::new(0.001, 0.0, 0.0), 0.016, &tuning());
        assert_eq!(grasp.release(), Some(Outcome::Tap(3)));
    }

    #[test]
    fn a_release_after_travel_is_a_place() {
        let mut grasp = Grasp::new();
        grasp.press(1, Vec3::ZERO, true);
        grasp.track(beyond_threshold(), 0.016, &tuning());
        let outcome = grasp.release().expect("outcome");
        assert!(matches!(outcome, Outcome::Place { slot: 1, .. }));
    }

    #[test]
    fn returning_to_the_origin_cancels_rather_than_taps() {
        let mut grasp = Grasp::new();
        grasp.press(2, Vec3::ZERO, true);
        grasp.track(beyond_threshold(), 0.016, &tuning());
        grasp.track(Vec3::ZERO, 0.016, &tuning());

        let outcome = grasp.release().expect("outcome");
        assert!(
            matches!(outcome, Outcome::Place { at, .. } if at == Vec3::ZERO),
            "a mote dragged out and back is a place at the origin, which the \
             caller cancels — not a tap that fires the action"
        );
    }

    #[test]
    fn a_fixed_mote_never_displaces_however_far_the_pointer_goes() {
        let mut grasp = Grasp::new();
        grasp.press(1, Vec3::ZERO, false);
        grasp.track(Vec3::new(5.0, 5.0, 5.0), 0.016, &tuning());
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
        grasp.track(beyond_threshold(), 0.016, &tuning());
        assert!(!grasp.is_seized());
    }

    #[test]
    fn a_throw_carries_velocity() {
        let mut grasp = Grasp::new();
        grasp.press(0, Vec3::ZERO, true);
        for step in 1..=8 {
            grasp.track(Vec3::new(step as f32 * 0.05, 0.0, 0.0), 0.016, &tuning());
        }
        let Some(Outcome::Place { velocity, .. }) = grasp.release() else {
            panic!("expected a place");
        };
        assert!(velocity.x > 0.0, "velocity points along the throw");
    }

    #[test]
    fn a_press_replaces_any_previous_hold() {
        let mut grasp = Grasp::new();
        grasp.press(1, Vec3::ZERO, true);
        grasp.press(2, Vec3::ZERO, true);
        assert_eq!(grasp.seized().map(|s| s.slot), Some(2));
    }
}
