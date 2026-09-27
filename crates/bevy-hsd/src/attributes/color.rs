use bevy::prelude::*;
use hsd::schema::material::ColorVec;

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
}
