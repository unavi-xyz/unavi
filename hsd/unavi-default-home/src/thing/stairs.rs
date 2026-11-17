//! Five steps that end in air. They rise away from the viewer, solid down to
//! the ground, and the top tread alone carries the space's colour.

use wired_guest::math::Vec3;

use super::part::{
    Paint,
    Part,
};

pub const STEPS: usize = 5;
pub const RISE: f32 = 0.18;
pub const RUN: f32 = 0.28;
pub const WIDTH: f32 = 1.2;
const INLAY: f32 = 0.01;

pub fn parts() -> Vec<Part> {
    let mut parts: Vec<Part> = (0..STEPS)
        .map(|i| {
            let height = RISE * (i + 1) as f32;
            Part::solid(
                Vec3::new(WIDTH, height, RUN),
                Vec3::new(0.0, height / 2.0, -RUN * (i as f32 + 0.5)),
                Paint::White,
            )
        })
        .collect();

    let top = RISE * STEPS as f32;
    parts.push(Part::trim(
        Vec3::new(WIDTH, INLAY, RUN),
        Vec3::new(0.0, top + INLAY / 2.0, -RUN * (STEPS as f32 - 0.5)),
        Paint::Hue,
    ));
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_are_five_steps_and_one_hued_face() {
        let parts = parts();
        let solid = parts.iter().filter(|p| p.solid).count();
        let hued = parts.iter().filter(|p| p.paint == Paint::Hue).count();
        assert_eq!((solid, hued), (STEPS, 1));
    }

    #[test]
    fn each_step_rises_by_one_rise() {
        let heights: Vec<f32> = parts()
            .iter()
            .filter(|p| p.solid)
            .map(|p| p.size.y)
            .collect();
        for pair in heights.windows(2) {
            assert!((pair[1] - pair[0] - RISE).abs() < 1.0e-5);
        }
    }
}
