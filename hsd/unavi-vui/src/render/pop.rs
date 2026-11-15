//! How a pane arrives and leaves, for every pane VUI draws: it grows a
//! little while it unrolls down from its top edge, each line of text showing
//! as the front passes it, and rolls back up to go. A progress value stepped
//! over time, and what it stands for.

/// Fraction of full size a pane starts from: enough growth to read as
/// arriving, little enough that it unrolls rather than zooms.
const START: f32 = 0.94;
/// Seconds to unroll.
const GROW: f32 = 0.24;
/// Seconds to roll back up, quicker than unrolling: leaving should not hold
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

/// How far down the pane the front has unrolled at `progress`, 0 to 1,
/// easing out so it slows as it reaches the bottom.
#[must_use]
pub fn front(progress: f32) -> f32 {
    let left = 1.0 - progress.clamp(0.0, 1.0);
    (left * left).mul_add(-left, 1.0)
}

/// How much a line `height` tall whose top is `top` below the pane's top
/// edge shows, with the front `reached` below that edge: none until the
/// front reaches it, whole once the front has passed it.
#[must_use]
pub fn line(reached: f32, top: f32, height: f32) -> f32 {
    if height <= f32::EPSILON {
        return if reached >= top { 1.0 } else { 0.0 };
    }
    ((reached - top) / height).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_front_runs_from_the_top_edge_to_the_bottom() {
        assert!(front(0.0).abs() < f32::EPSILON);
        assert!((front(1.0) - 1.0).abs() < f32::EPSILON);
        assert!(front(0.5) > 0.5, "eases out");
    }

    #[test]
    fn a_line_shows_as_the_front_passes_it() {
        assert!(line(0.1, 0.2, 0.05).abs() < f32::EPSILON);
        assert!((line(0.225, 0.2, 0.05) - 0.5).abs() < 1.0e-5);
        assert!((line(0.3, 0.2, 0.05) - 1.0).abs() < f32::EPSILON);
    }

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
