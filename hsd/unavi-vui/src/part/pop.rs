//! How a pane arrives and leaves, for every pane VUI draws: the glass is
//! whole from the first frame and grows a little to full size while its type
//! fades in, and shrinks back as it goes. A progress value stepped over time,
//! and what it stands for.

/// Fraction of full size a pane starts from: enough growth to read as
/// arriving, little enough that it settles rather than zooms.
const START: f32 = 0.94;
/// Seconds to arrive.
const GROW: f32 = 0.24;
/// Seconds to go, quicker than arriving: leaving should not hold
/// anything up.
const SHRINK: f32 = 0.16;

/// Moves `progress` toward 1 while `shown` and toward 0 otherwise, at the
/// speed of that direction.
#[must_use]
pub fn step(progress: f32, shown: bool, delta: f32) -> f32 {
    if shown {
        (progress + delta / GROW).min(1.0)
    } else {
        (progress - delta / SHRINK).max(0.0)
    }
}

/// Size at `progress`: [`START`] at 0, exactly 1 at 1, easing out cubically so
/// it lands without overshooting.
#[must_use]
pub fn scale(progress: f32) -> f32 {
    let left = 1.0 - progress.clamp(0.0, 1.0);
    (1.0 - START).mul_add((left * left).mul_add(-left, 1.0), START)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_starts_small_and_ends_at_exactly_full_size() {
        assert!((scale(0.0) - START).abs() < 1.0e-6);
        assert!((scale(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn it_grows_monotonically_without_overshoot() {
        let mut last = scale(0.0);
        for index in 1..=100 {
            let now = scale(index as f32 / 100.0);
            assert!(now >= last, "shrank at {index}");
            assert!(now <= 1.0, "overshot at {index}");
            last = now;
        }
    }

    #[test]
    fn it_eases_out_so_most_of_the_growth_comes_first() {
        assert!(scale(0.5) > f32::midpoint(START, 1.0));
    }

    #[test]
    fn progress_stays_within_bounds() {
        assert!((step(0.95, true, 1.0) - 1.0).abs() < f32::EPSILON);
        assert!(step(0.05, false, 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn leaving_is_quicker_than_arriving() {
        let frames = |shown: bool| {
            let mut progress = if shown { 0.0 } else { 1.0 };
            (0..1000)
                .take_while(|_| {
                    progress = step(progress, shown, 1.0 / 60.0);
                    progress > 0.0 && progress < 1.0
                })
                .count()
        };
        assert!(frames(false) < frames(true));
    }

    #[test]
    fn a_settled_pop_does_not_move() {
        assert!((step(1.0, true, 0.016) - 1.0).abs() < f32::EPSILON);
        assert!(step(0.0, false, 0.016).abs() < f32::EPSILON);
    }
}
