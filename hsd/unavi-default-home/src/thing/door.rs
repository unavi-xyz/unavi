//! A door in a frame, standing alone with no wall. The leaf is the one blue
//! face.

use wired_guest::math::Vec3;

use super::part::{
    Paint,
    Part,
};

pub const OPENING_WIDTH: f32 = 0.9;
pub const OPENING_HEIGHT: f32 = 2.0;
const FRAME_WIDTH: f32 = 0.08;
const FRAME_DEPTH: f32 = 0.12;
const LEAF_DEPTH: f32 = 0.04;
/// Clearance between the leaf and the frame on each side.
const LEAF_GAP: f32 = 0.01;

pub fn parts() -> Vec<Part> {
    let jamb_height = OPENING_HEIGHT + FRAME_WIDTH;
    let jamb_x = OPENING_WIDTH / 2.0 + FRAME_WIDTH / 2.0;
    let jamb = Vec3::new(FRAME_WIDTH, jamb_height, FRAME_DEPTH);

    vec![
        Part::solid(jamb, Vec3::new(-jamb_x, jamb_height / 2.0, 0.0), Paint::White),
        Part::solid(jamb, Vec3::new(jamb_x, jamb_height / 2.0, 0.0), Paint::White),
        Part::solid(
            Vec3::new(FRAME_WIDTH.mul_add(2.0, OPENING_WIDTH), FRAME_WIDTH, FRAME_DEPTH),
            Vec3::new(0.0, OPENING_HEIGHT + FRAME_WIDTH / 2.0, 0.0),
            Paint::White,
        ),
        Part::solid(
            Vec3::new(
                LEAF_GAP.mul_add(-2.0, OPENING_WIDTH),
                OPENING_HEIGHT - LEAF_GAP,
                LEAF_DEPTH,
            ),
            Vec3::new(0.0, (OPENING_HEIGHT - LEAF_GAP) / 2.0, 0.0),
            Paint::Blue,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_leaf_is_the_only_blue_part() {
        let blue = parts().iter().filter(|p| p.paint == Paint::Blue).count();
        assert_eq!(blue, 1);
    }

    #[test]
    fn the_leaf_fits_inside_the_frame() {
        let parts = parts();
        let leaf = parts.iter().find(|p| p.paint == Paint::Blue);
        let leaf = leaf.map(|p| p.size).unwrap_or_default();
        assert!(leaf.x < OPENING_WIDTH && leaf.y < OPENING_HEIGHT);
    }
}
