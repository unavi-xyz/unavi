use std::f32::consts::TAU;

use wired_math::types::Vec2;

use crate::tuning::Tuning;

/// How an orbit's sibling slots are arranged.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutKind {
    /// `points` slots evenly spaced, slot 0 at the top, advancing clockwise.
    Star { points: usize },
    /// [`LayoutKind::Star`] plus a centre slot at index 0.
    Centred { points: usize },
    /// A vertical run, centred on the anchor.
    Column { count: usize, pitch: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub kind:   LayoutKind,
    pub radius: f32,
}

impl Layout {
    #[must_use]
    pub const fn star(points: usize, radius: f32) -> Self {
        Self {
            kind: LayoutKind::Star { points },
            radius,
        }
    }

    #[must_use]
    pub const fn centred(points: usize, radius: f32) -> Self {
        Self {
            kind: LayoutKind::Centred { points },
            radius,
        }
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        match self.kind {
            LayoutKind::Star { points } => points,
            LayoutKind::Centred { points } => points + 1,
            LayoutKind::Column { count, .. } => count,
        }
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub const fn has_centre(&self) -> bool {
        matches!(self.kind, LayoutKind::Centred { .. })
    }

    /// Slots arranged radially, excluding any centre slot.
    #[must_use]
    pub const fn ring_len(&self) -> usize {
        match self.kind {
            LayoutKind::Star { points } | LayoutKind::Centred { points } => points,
            LayoutKind::Column { .. } => 0,
        }
    }

    #[must_use]
    pub fn slot(&self, index: usize) -> Option<Vec2> {
        if index >= self.len() {
            return None;
        }
        match self.kind {
            LayoutKind::Star { points } => Some(ring_position(index, points, self.radius)),
            LayoutKind::Centred { points } => match index {
                0 => Some(Vec2::ZERO),
                _ => Some(ring_position(index - 1, points, self.radius)),
            },
            LayoutKind::Column { count, pitch } => {
                let centred = ((count - 1) as f32).mul_add(0.5, -(index as f32));
                Some(Vec2::new(0.0, centred * pitch))
            }
        }
    }

    /// The slot a local-plane point falls in.
    ///
    /// `current` is the slot already holding attention; its wedge is widened
    /// by [`Tuning::stick`].
    #[must_use]
    pub fn resolve(&self, local: Vec2, current: Option<usize>, tuning: &Tuning) -> Option<usize> {
        if self.is_empty() {
            return None;
        }
        match self.kind {
            LayoutKind::Column { count, pitch } => resolve_column(local, count, pitch),
            LayoutKind::Star { .. } | LayoutKind::Centred { .. } => {
                self.resolve_radial(local, current, tuning)
            }
        }
    }

    fn resolve_radial(
        &self,
        local: Vec2,
        current: Option<usize>,
        tuning: &Tuning,
    ) -> Option<usize> {
        let distance = local.length();
        if self.has_centre() && distance <= self.radius * tuning.centre_frac {
            return Some(0);
        }
        if distance > self.radius * tuning.reach_frac {
            return None;
        }

        let points = self.ring_len();
        if points == 0 {
            return None;
        }

        let offset = usize::from(self.has_centre());
        let angle = local.x.atan2(local.y);
        (0..points)
            .map(|i| {
                let slot = i + offset;
                let mut delta = angular_delta(angle, i as f32 * TAU / points as f32);
                if current == Some(slot) {
                    delta -= tuning.stick;
                }
                (slot, delta)
            })
            .min_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(slot, _)| slot)
    }
}

fn ring_position(index: usize, points: usize, radius: f32) -> Vec2 {
    let angle = index as f32 * TAU / points as f32;
    Vec2::new(radius * angle.sin(), radius * angle.cos())
}

fn resolve_column(local: Vec2, count: usize, pitch: f32) -> Option<usize> {
    if pitch <= f32::EPSILON {
        return None;
    }
    let top = (count - 1) as f32 * 0.5;
    let index = (top - local.y / pitch).round();
    if index < 0.0 || index >= count as f32 {
        return None;
    }
    Some(index as usize)
}

/// Shortest absolute angle between two directions, in `0..=PI`.
fn angular_delta(a: f32, b: f32) -> f32 {
    let delta = (a - b).rem_euclid(TAU);
    delta.min(TAU - delta)
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use super::*;

    const R: f32 = 0.2;

    fn tuning() -> Tuning {
        Tuning::DEFAULT
    }

    #[test]
    fn star_slot_zero_is_up() {
        let layout = Layout::star(5, R);
        let slot = layout.slot(0).expect("slot 0");
        assert!((slot.x - 0.0).abs() < 1.0e-5);
        assert!((slot.y - R).abs() < 1.0e-5);
    }

    #[test]
    fn star_advances_clockwise() {
        let layout = Layout::star(4, R);
        assert!(layout.slot(1).expect("slot 1").x > 0.0, "slot 1 is right");
        assert!(layout.slot(2).expect("slot 2").y < 0.0, "slot 2 is down");
        assert!(layout.slot(3).expect("slot 3").x < 0.0, "slot 3 is left");
    }

    #[test]
    fn centred_puts_the_centre_first_and_shifts_the_ring() {
        let centred = Layout::centred(4, R);
        let star = Layout::star(4, R);
        assert_eq!(centred.len(), 5);
        assert_eq!(centred.slot(0), Some(Vec2::ZERO));
        assert_eq!(centred.slot(1), star.slot(0));
        assert_eq!(centred.slot(4), star.slot(3));
        assert_eq!(centred.slot(5), None);
    }

    #[test]
    fn resolve_picks_the_nearest_direction() {
        let layout = Layout::star(4, R);
        assert_eq!(layout.resolve(Vec2::new(0.0, R), None, &tuning()), Some(0));
        assert_eq!(layout.resolve(Vec2::new(R, 0.0), None, &tuning()), Some(1));
        assert_eq!(layout.resolve(Vec2::new(0.0, -R), None, &tuning()), Some(2));
        assert_eq!(layout.resolve(Vec2::new(-R, 0.0), None, &tuning()), Some(3));
    }

    #[test]
    fn centre_wins_inside_its_radius() {
        let layout = Layout::centred(4, R);
        let inside = R * tuning().centre_frac * 0.5;
        assert_eq!(
            layout.resolve(Vec2::new(inside, 0.0), None, &tuning()),
            Some(0)
        );
    }

    #[test]
    fn a_star_has_no_centre_target() {
        let layout = Layout::star(5, R);
        let resolved = layout.resolve(Vec2::new(0.0, 0.0), None, &tuning());
        assert_ne!(resolved, None, "a null deflection still picks a direction");
    }

    #[test]
    fn nothing_is_attended_past_reach() {
        let layout = Layout::star(4, R);
        let far = R * tuning().reach_frac * 1.1;
        assert_eq!(layout.resolve(Vec2::new(0.0, far), None, &tuning()), None);
    }

    #[test]
    fn stick_holds_the_current_slot_across_the_boundary() {
        let layout = Layout::star(4, R);
        // Just past halfway from slot 0 toward slot 1.
        let past = PI / 4.0 + 0.08;
        let point = Vec2::new(R * past.sin(), R * past.cos());

        assert_eq!(layout.resolve(point, None, &tuning()), Some(1));
        assert_eq!(
            layout.resolve(point, Some(0), &tuning()),
            Some(0),
            "attention sticks until the pointer moves well into the neighbour"
        );
    }

    #[test]
    fn stick_yields_once_the_pointer_commits() {
        let layout = Layout::star(4, R);
        let well_past = PI / 4.0 + 0.4;
        let point = Vec2::new(R * well_past.sin(), R * well_past.cos());
        assert_eq!(layout.resolve(point, Some(0), &tuning()), Some(1));
    }

    #[test]
    fn column_resolves_by_height_and_bounds() {
        let layout = Layout {
            kind:   LayoutKind::Column {
                count: 3,
                pitch: 0.1,
            },
            radius: R,
        };
        assert_eq!(layout.slot(1), Some(Vec2::ZERO));
        assert_eq!(
            layout.resolve(Vec2::new(0.0, 0.1), None, &tuning()),
            Some(0)
        );
        assert_eq!(
            layout.resolve(Vec2::new(0.0, 0.0), None, &tuning()),
            Some(1)
        );
        assert_eq!(
            layout.resolve(Vec2::new(0.0, -0.1), None, &tuning()),
            Some(2)
        );
        assert_eq!(layout.resolve(Vec2::new(0.0, 0.4), None, &tuning()), None);
    }

    #[test]
    fn an_empty_layout_resolves_to_nothing() {
        let layout = Layout::star(0, R);
        assert!(layout.is_empty());
        assert_eq!(layout.resolve(Vec2::new(0.0, R), None, &tuning()), None);
    }

    #[test]
    fn every_slot_resolves_to_itself() {
        for points in [3_usize, 4, 5, 7, 12] {
            let layout = Layout::centred(points, R);
            for index in 0..layout.len() {
                let position = layout.slot(index).expect("slot");
                assert_eq!(
                    layout.resolve(position, None, &tuning()),
                    Some(index),
                    "points={points} index={index}"
                );
            }
        }
    }
}
