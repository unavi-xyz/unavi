//! Size steps as physical heights.

/// Em height in metres of a size step of `mrad` milliradians read from
/// `distance` metres.
pub fn em_height(mrad: f32, distance: f32) -> f32 {
    mrad * distance / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_at_the_halos_reading_distance() {
        assert!((em_height(20.0, 0.7) - 0.014).abs() < 1.0e-6);
    }

    #[test]
    fn height_grows_with_distance() {
        assert!(em_height(28.0, 3.0) > em_height(28.0, 0.5));
    }
}
