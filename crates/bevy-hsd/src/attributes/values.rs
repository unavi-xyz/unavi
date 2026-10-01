use std::ops::RangeInclusive;

use bevy::prelude::*;
use hsd::attributes::material::ColorVec;

/// Reads up to four linear channels as `r, g, b, a`. Missing channels pad
/// with `1.0`. Any non-finite channel yields `fallback`.
pub fn from_color_vec(vec: Option<&ColorVec>, fallback: Color) -> Color {
    let Some(vec) = vec else { return fallback };
    let channel = |index: usize| vec.0.get(index).copied().unwrap_or(1.0) as f32;
    let channels = [channel(0), channel(1), channel(2), channel(3)];
    if channels.iter().all(|c| c.is_finite()) {
        let [r, g, b, a] = channels;
        Color::linear_rgba(r, g, b, a)
    } else {
        fallback
    }
}

/// `value` held to `range`, or `fallback` when absent or non-finite.
pub fn clamped(value: Option<f64>, fallback: f32, range: RangeInclusive<f32>) -> f32 {
    value
        .map(|value| value as f32)
        .filter(|value| value.is_finite())
        .unwrap_or(fallback)
        .clamp(*range.start(), *range.end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_short_colour_vector_pads_rather_than_panicking() {
        let padded = from_color_vec(Some(&ColorVec(vec![0.5])), Color::WHITE);
        assert_eq!(padded, Color::linear_rgba(0.5, 1.0, 1.0, 1.0));
    }

    #[test]
    fn a_non_finite_channel_rejects_the_whole_colour() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let rejected = from_color_vec(Some(&ColorVec(vec![bad, 0.0, 0.0])), Color::WHITE);
            assert_eq!(rejected, Color::WHITE);
        }
    }

    #[test]
    fn no_vector_falls_back() {
        assert_eq!(from_color_vec(None, Color::BLACK), Color::BLACK);
    }

    #[test]
    fn a_non_finite_value_falls_back() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!((clamped(Some(value), 1.0, 0.0..=10.0) - 1.0).abs() < 1.0e-9);
        }
    }

    #[test]
    fn an_absurd_value_is_held_to_the_range() {
        assert!((clamped(Some(1.0e30), 1.0, 0.0..=10.0) - 10.0).abs() < 1.0e-6);
        assert!(clamped(Some(-4.0), 1.0, 0.0..=10.0).abs() < 1.0e-9);
        assert!((clamped(None, 1.0, 0.0..=10.0) - 1.0).abs() < 1.0e-9);
    }
}
