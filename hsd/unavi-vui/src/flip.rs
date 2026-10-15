//! A face changing what it shows: which glyph is leaving, which is arriving,
//! and how far along the change is.
//!
//! The shader keeps no clock of its own, so the run is driven from here and
//! handed over as a single number. That keeps a flip reproducible and lets a
//! settled face stop being written at all.

use std::rc::Rc;

use crate::glyph::Glyph;

/// What one slot's face is showing.
#[derive(Debug)]
pub struct Flip {
    /// Whose face this is. A slot another mote moves into starts afresh
    /// rather than flipping from a stranger's glyph.
    owner:    Option<u64>,
    shown:    Option<Rc<Glyph>>,
    /// What the run is leaving, while there is one.
    leaving:  Option<Rc<Glyph>>,
    /// How far through the run, `1` once it is over.
    progress: f32,
}

impl Flip {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            owner:    None,
            shown:    None,
            leaving:  None,
            progress: 1.0,
        }
    }

    /// Takes the glyph `owner` wants shown.
    ///
    /// A change to a glyph of the same size on the same mote starts a run.
    /// Anything else appears at once: a different mote in the slot has
    /// nothing to flip from, a glyph that was not there before has no
    /// predecessor, and a grid of another size has no cells to line up.
    pub fn show(&mut self, owner: u64, glyph: Option<&Rc<Glyph>>) {
        let same_owner = self.owner == Some(owner);
        self.owner = Some(owner);

        if same_owner && self.shown.as_ref() == glyph {
            return;
        }

        let from = self
            .shown
            .clone()
            .filter(|from| same_owner && glyph.is_some_and(|to| to.size() == from.size()));
        self.progress = if from.is_some() { 0.0 } else { 1.0 };
        self.leaving = from;
        self.shown = glyph.cloned();
    }

    /// Advances the run by `delta` seconds of a run `duration` seconds long.
    pub fn step(&mut self, delta: f32, duration: f32) {
        if self.leaving.is_none() {
            return;
        }
        self.progress = if duration > 0.0 {
            (self.progress + delta / duration).min(1.0)
        } else {
            1.0
        };
        if self.progress >= 1.0 {
            self.leaving = None;
        }
    }

    /// The glyph the face is settling on, or none for a face with nothing
    /// to show.
    #[must_use]
    pub const fn shown(&self) -> Option<&Rc<Glyph>> {
        self.shown.as_ref()
    }

    /// The glyph the run started from, which is the shown one once settled.
    #[must_use]
    pub fn leaving(&self) -> Option<&Rc<Glyph>> {
        self.leaving.as_ref().or(self.shown.as_ref())
    }

    /// How far through the run the face is, `0` just started and `1` over.
    #[must_use]
    pub const fn progress(&self) -> f32 {
        self.progress
    }

    /// Whether a run is under way. Only a check on the state machine: the
    /// renderer reads [`Flip::progress`] and writes when it changes.
    #[cfg(test)]
    pub const fn is_flipping(&self) -> bool {
        self.leaving.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glyph(size: u8, lit: &[usize]) -> Rc<Glyph> {
        let mut bits = vec![0_u8; (usize::from(size) * usize::from(size)).div_ceil(8)];
        for cell in lit {
            bits[cell / 8] |= 1 << (cell % 8);
        }
        Rc::new(Glyph::new(size, &bits).expect("a well-formed glyph"))
    }

    /// A flip already showing `glyph` for mote 1.
    fn settled(glyph: &Rc<Glyph>) -> Flip {
        let mut flip = Flip::new();
        flip.show(1, Some(glyph));
        flip
    }

    #[test]
    fn a_new_face_appears_without_a_flip() {
        let mut flip = Flip::new();
        flip.show(1, Some(&glyph(5, &[12])));
        assert!(!flip.is_flipping());
        assert!((flip.progress() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn a_changed_glyph_on_the_same_mote_runs_from_the_old_to_the_new() {
        let (old, new) = (glyph(5, &[12]), glyph(5, &[6, 12]));
        let mut flip = settled(&old);
        flip.show(1, Some(&new));

        assert!(flip.is_flipping());
        assert!(flip.progress().abs() < f32::EPSILON);
        assert_eq!(flip.leaving(), Some(&old));
        assert_eq!(flip.shown(), Some(&new));
    }

    #[test]
    fn the_same_glyph_again_starts_nothing() {
        let face = glyph(5, &[12]);
        let mut flip = settled(&face);
        flip.show(1, Some(&glyph(5, &[12])));
        assert!(!flip.is_flipping(), "equal by what is drawn");
    }

    #[test]
    fn another_mote_arriving_in_the_slot_appears_without_a_flip() {
        let mut flip = settled(&glyph(5, &[12]));
        flip.show(2, Some(&glyph(5, &[6])));
        assert!(!flip.is_flipping());
    }

    #[test]
    fn a_grid_of_another_size_appears_without_a_flip() {
        let mut flip = settled(&glyph(5, &[12]));
        flip.show(1, Some(&glyph(7, &[24])));
        assert!(!flip.is_flipping());
    }

    #[test]
    fn losing_the_glyph_clears_the_face_and_gaining_one_back_does_not_flip() {
        let mut flip = settled(&glyph(5, &[12]));
        flip.show(1, None);
        assert_eq!(flip.shown(), None);
        flip.show(1, Some(&glyph(5, &[6])));
        assert!(!flip.is_flipping());
    }

    #[test]
    fn a_run_takes_its_duration_and_then_settles() {
        let mut flip = settled(&glyph(5, &[12]));
        flip.show(1, Some(&glyph(5, &[6])));

        flip.step(0.1, 0.3);
        assert!((flip.progress() - 1.0 / 3.0).abs() < 1.0e-6);
        assert!(flip.is_flipping());

        flip.step(0.1, 0.3);
        flip.step(0.1, 0.3);
        // Three thirds in floats can land a hair short; the next frame, however
        // short, finishes it.
        assert!(
            (flip.progress() - 1.0).abs() < 1.0e-5,
            "{}",
            flip.progress()
        );
        flip.step(0.001, 0.3);
        assert!(!flip.is_flipping());
        assert!((flip.progress() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn a_long_frame_ends_the_run_at_one_and_no_further() {
        let mut flip = settled(&glyph(5, &[12]));
        flip.show(1, Some(&glyph(5, &[6])));
        flip.step(5.0, 0.3);
        assert!((flip.progress() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn a_run_of_no_length_is_over_at_once() {
        let mut flip = settled(&glyph(5, &[12]));
        flip.show(1, Some(&glyph(5, &[6])));
        flip.step(0.016, 0.0);
        assert!(!flip.is_flipping());
    }

    #[test]
    fn a_change_mid_run_leaves_from_what_was_arriving() {
        let (a, b, c) = (glyph(5, &[12]), glyph(5, &[6]), glyph(5, &[18]));
        let mut flip = settled(&a);
        flip.show(1, Some(&b));
        flip.step(0.1, 0.3);
        flip.show(1, Some(&c));

        assert!(flip.progress().abs() < f32::EPSILON, "the run starts over");
        assert_eq!(flip.leaving(), Some(&b));
        assert_eq!(flip.shown(), Some(&c));
    }
}
