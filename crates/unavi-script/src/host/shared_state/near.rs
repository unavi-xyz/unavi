//! Picking the nearest of a snapshot, for queries by radius.

use bevy::prelude::*;

/// The most items one query answers.
pub const MAX_NEAR: usize = 64;

/// Up to [`MAX_NEAR`] of `items` whose point `at` lies within `radius` of
/// `centre`, nearest first.
pub fn nearest<T: Clone>(
    items: &[T],
    centre: Vec3,
    radius: f32,
    at: impl Fn(&T) -> Vec3,
) -> Vec<T> {
    let mut found = items
        .iter()
        .map(|item| (at(item).distance(centre), item))
        .filter(|(distance, _)| *distance <= radius)
        .collect::<Vec<_>>();
    found.sort_by(|a, b| a.0.total_cmp(&b.0));
    found.truncate(MAX_NEAR);
    found.into_iter().map(|(_, item)| item.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn xs(found: &[Vec3]) -> Vec<f32> {
        found.iter().map(|at| at.x).collect()
    }

    fn along_x(xs: &[f32]) -> Vec<Vec3> {
        xs.iter().map(|x| Vec3::new(*x, 0.0, 0.0)).collect()
    }

    #[test]
    fn keeps_what_is_in_range_nearest_first() {
        let found = nearest(&along_x(&[5.0, 1.0, 20.0, 3.0]), Vec3::ZERO, 5.0, |at| *at);
        assert_eq!(xs(&found), [1.0, 3.0, 5.0], "the edge is in range");
    }

    #[test]
    fn is_measured_from_the_centre_given() {
        let found = nearest(&along_x(&[5.0, 1.0]), Vec3::new(6.0, 0.0, 0.0), 2.0, |at| {
            *at
        });
        assert_eq!(xs(&found), [5.0]);
    }

    #[test]
    fn stops_at_the_cap_keeping_the_nearest() {
        let all = (0..100).rev().map(|i| i as f32).collect::<Vec<_>>();
        let found = nearest(&along_x(&all), Vec3::ZERO, 1000.0, |at| *at);
        assert_eq!(found.len(), MAX_NEAR);
        assert_eq!(found[0].x, 0.0);
        assert_eq!(found[MAX_NEAR - 1].x, 63.0);
    }
}
