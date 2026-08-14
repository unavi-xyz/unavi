use crate::tuning::Tuning;

/// Where a cast is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    /// Filling; `0.0..1.0`.
    Filling(f32),
    /// Held to the end. The action fires exactly once, on the frame this is
    /// first reported.
    Committed,
    /// Attention left the site before it filled.
    Aborted,
}

impl State {
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

/// The hold-to-confirm FSM for a consequential action: somewhere to draw, a
/// duration to fill, and an abort by pulling away.
///
/// Not for anything reversible — a cast on a cheap action is a nuisance, and
/// its overuse destroys the signal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cast {
    duration: f32,
    elapsed:  f32,
    state:    State,
}

impl Cast {
    #[must_use]
    pub const fn new(duration: f32) -> Self {
        Self {
            duration,
            elapsed: 0.0,
            state: State::Filling(0.0),
        }
    }

    #[must_use]
    pub const fn standard(tuning: &Tuning) -> Self {
        Self::new(tuning.cast_duration)
    }

    #[must_use]
    pub const fn state(&self) -> State {
        self.state
    }

    /// Advances the fill. `held` is whether attention is still on the site;
    /// losing it aborts immediately, with no grace period to learn.
    ///
    /// Settled casts stay settled, so a caller may keep polling until it
    /// takes the cast away.
    pub fn update(&mut self, held: bool, delta: f32) -> State {
        if self.state.is_settled() {
            return self.state;
        }
        if !held {
            self.state = State::Aborted;
            return self.state;
        }
        self.elapsed += delta;
        self.state = if self.duration <= f32::EPSILON || self.elapsed >= self.duration {
            State::Committed
        } else {
            State::Filling((self.elapsed / self.duration).clamp(0.0, 1.0))
        };
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DURATION: f32 = 1.0;

    fn cast() -> Cast {
        Cast::new(DURATION)
    }

    #[test]
    fn a_fresh_cast_has_not_started() {
        assert_eq!(cast().state(), State::Filling(0.0));
    }

    #[test]
    fn holding_fills_it() {
        let mut cast = cast();
        assert_eq!(cast.update(true, 0.25), State::Filling(0.25));
        assert_eq!(cast.update(true, 0.25), State::Filling(0.5));
    }

    #[test]
    fn holding_to_the_end_commits_once() {
        let mut cast = cast();
        cast.update(true, 0.5);
        assert_eq!(cast.update(true, 0.6), State::Committed);
        assert_eq!(
            cast.update(true, 0.6),
            State::Committed,
            "a settled cast stays settled rather than firing again"
        );
    }

    #[test]
    fn pulling_away_aborts_it() {
        let mut cast = cast();
        cast.update(true, 0.9);
        assert_eq!(cast.update(false, 0.016), State::Aborted);
    }

    #[test]
    fn an_abort_is_final_even_if_attention_comes_back() {
        let mut cast = cast();
        cast.update(true, 0.5);
        cast.update(false, 0.016);
        assert_eq!(
            cast.update(true, 0.6),
            State::Aborted,
            "a cast you walked out of is not one you can walk back into"
        );
    }

    #[test]
    fn a_committed_cast_is_not_undone_by_letting_go() {
        let mut cast = cast();
        cast.update(true, DURATION);
        assert_eq!(cast.update(false, 0.016), State::Committed);
    }

    #[test]
    fn a_zero_duration_cast_commits_on_its_first_frame() {
        let mut cast = Cast::new(0.0);
        assert_eq!(cast.update(true, 0.016), State::Committed);
    }

    #[test]
    fn progress_reads_the_same_off_any_state() {
        assert!((State::Filling(0.4).progress() - 0.4).abs() < 1.0e-6);
        assert!((State::Committed.progress() - 1.0).abs() < 1.0e-6);
        assert!(State::Aborted.progress().abs() < 1.0e-6);
    }
}
