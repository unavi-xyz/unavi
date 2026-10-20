//! Where a tag stands beside a point.

use wired_guest::math::{
    Vec2,
    Vec3,
};

use super::frame::Placement;

/// Metres between the point and the near edge of the tag.
const GAP: f32 = 0.12;
/// Metres the middle of the tag stands above the point.
const RISE: f32 = 0.05;
/// Metres the tag stands off the point's plane, on the viewer's side, so a tag
/// beside something set into a wall is not set into it too.
const OUT: f32 = 0.12;

/// A tag of `size` standing to the right of `target` as `eye` sees it, at the
/// point's height, with its tether running short and level from its near
/// side to the point.
#[must_use]
pub fn beside(eye: Vec3, target: Vec3, size: Vec2) -> Placement {
    let ahead = Vec3::new(target.x - eye.x, 0.0, target.z - eye.z).normalize_or_zero();
    let ahead = if ahead == Vec3::ZERO {
        Vec3::NEG_Z
    } else {
        ahead
    };
    let right = ahead.cross(Vec3::Y);
    Placement {
        at:     target + right * (GAP + size.x / 2.0) + Vec3::Y * (RISE - size.y / 2.0)
            - ahead * OUT,
        tether: Some(target),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tag_stands_clear_of_the_point_on_the_viewers_right() {
        let eye = Vec3::new(0.0, 1.6, 0.0);
        let target = Vec3::new(0.0, 1.0, -3.0);
        let size = Vec2::new(0.8, 0.3);
        let placed = beside(eye, target, size);

        let near_edge = placed.at.x - size.x / 2.0;
        assert!(
            (near_edge - target.x - GAP).abs() < 1.0e-5,
            "right, a gap off"
        );
        let middle = placed.middle(size);
        assert!((middle.y - target.y - RISE).abs() < 1.0e-5);
        let tether = placed.tether.expect("a tether");
        assert!((middle.y - tether.y).abs() <= RISE + 1.0e-5, "level tether");
        assert!(
            (placed.at.z - target.z - OUT).abs() < 1.0e-5,
            "a step toward the viewer, clear of the point's plane"
        );
    }

    #[test]
    fn a_viewer_directly_above_still_gets_a_side() {
        let placed = beside(Vec3::new(0.0, 5.0, 0.0), Vec3::ZERO, Vec2::new(0.8, 0.3));
        assert!(placed.at.is_finite());
    }
}
