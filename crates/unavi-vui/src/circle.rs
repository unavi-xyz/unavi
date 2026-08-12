use crate::tuning::Tuning;

/// How far a cast has got.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cast {
    /// Filling; `0.0..1.0`.
    Filling(f32),
    /// Held to the end. The action fires exactly once, on the frame this is
    /// first reported.
    Committed,
    /// Attention left the site before it filled.
    Aborted,
}

impl Cast {
    #[must_use]
    pub const fn is_settled(self) -> bool {
        matches!(self, Self::Committed | Self::Aborted)
    }

    #[must_use]
    pub const fn progress(self) -> f32 {
        match self {
            Self::Filling(progress) => progress,
            Self::Committed => 1.0,
            Self::Aborted => 0.0,
        }
    }
}

/// The cast site for a consequential action: somewhere to draw, a duration to
/// fill, and an abort by pulling away.
///
/// Not for anything reversible — a cast on a cheap action is a nuisance, and
/// its overuse destroys the signal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    duration: f32,
    elapsed:  f32,
    cast:     Cast,
}

impl Circle {
    #[must_use]
    pub const fn new(duration: f32) -> Self {
        Self {
            duration,
            elapsed: 0.0,
            cast: Cast::Filling(0.0),
        }
    }

    #[must_use]
    pub const fn standard(tuning: &Tuning) -> Self {
        Self::new(tuning.cast_duration)
    }

    #[must_use]
    pub const fn cast(&self) -> Cast {
        self.cast
    }

    /// Advances the fill. `held` is whether attention is still on the site;
    /// losing it aborts immediately, with no grace period to learn.
    ///
    /// Settled casts stay settled, so a caller may keep polling until it
    /// takes the circle away.
    pub fn update(&mut self, held: bool, delta: f32) -> Cast {
        if self.cast.is_settled() {
            return self.cast;
        }
        if !held {
            self.cast = Cast::Aborted;
            return self.cast;
        }
        self.elapsed += delta;
        self.cast = if self.duration <= f32::EPSILON || self.elapsed >= self.duration {
            Cast::Committed
        } else {
            Cast::Filling((self.elapsed / self.duration).clamp(0.0, 1.0))
        };
        self.cast
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DURATION: f32 = 1.0;

    fn circle() -> Circle {
        Circle::new(DURATION)
    }

    #[test]
    fn a_fresh_circle_has_not_started() {
        assert_eq!(circle().cast(), Cast::Filling(0.0));
    }

    #[test]
    fn holding_fills_it() {
        let mut circle = circle();
        assert_eq!(circle.update(true, 0.25), Cast::Filling(0.25));
        assert_eq!(circle.update(true, 0.25), Cast::Filling(0.5));
    }

    #[test]
    fn holding_to_the_end_commits_once() {
        let mut circle = circle();
        circle.update(true, 0.5);
        assert_eq!(circle.update(true, 0.6), Cast::Committed);
        assert_eq!(
            circle.update(true, 0.6),
            Cast::Committed,
            "a settled cast stays settled rather than firing again"
        );
    }

    #[test]
    fn pulling_away_aborts_it() {
        let mut circle = circle();
        circle.update(true, 0.9);
        assert_eq!(circle.update(false, 0.016), Cast::Aborted);
    }

    #[test]
    fn an_abort_is_final_even_if_attention_comes_back() {
        let mut circle = circle();
        circle.update(true, 0.5);
        circle.update(false, 0.016);
        assert_eq!(
            circle.update(true, 0.6),
            Cast::Aborted,
            "a cast you walked out of is not one you can walk back into"
        );
    }

    #[test]
    fn a_committed_cast_is_not_undone_by_letting_go() {
        let mut circle = circle();
        circle.update(true, DURATION);
        assert_eq!(circle.update(false, 0.016), Cast::Committed);
    }

    #[test]
    fn a_zero_duration_cast_commits_on_its_first_frame() {
        let mut circle = Circle::new(0.0);
        assert_eq!(circle.update(true, 0.016), Cast::Committed);
    }

    #[test]
    fn progress_reads_the_same_off_any_state() {
        assert!((Cast::Filling(0.4).progress() - 0.4).abs() < 1.0e-6);
        assert!((Cast::Committed.progress() - 1.0).abs() < 1.0e-6);
        assert!(Cast::Aborted.progress().abs() < 1.0e-6);
    }
}
