//! Where a portal's tag stands: at eye height beside the nearer post, tied
//! to the post by its tether, or over the top edge of a portal too short to
//! stand beside.
//!
//! Either way the pane stands off the portal's plane toward the viewer, so it
//! never sinks into the rim, the veil or whatever frame the space built.

use wired_guest::math::{
    Transform,
    Vec2,
    Vec3,
};

/// Metres above a portal's bottom edge the middle of a tag beside it stands:
/// eye height for someone on the portal's floor.
const EYE_HEIGHT: f32 = 1.5;
/// Metres a portal must reach above [`EYE_HEIGHT`] for a tag to stand beside
/// it; anything shorter takes its tag over the top edge.
const HEADROOM: f32 = 0.25;
/// Metres between a portal's side edge and the near edge of a tag beside it,
/// which the tether spans.
const SIDE_GAP: f32 = 0.2;
/// Metres between a portal's top edge and the bottom of a tag over it.
const CLEARANCE: f32 = 0.1;
/// Metres the tag stands off the portal's plane, on the viewer's side.
const OUT: f32 = 0.25;
/// Metres the tether's dot stands off the portal's plane, on the viewer's
/// side, so it sits on the face of a post rather than inside it.
const DOT_OUT: f32 = 0.08;
/// Metres a viewer must cross past the portal's middle before its tag moves
/// to the other post, so one standing in front of it does not see the tag
/// swap sides as they sway.
const SIDE_HOLD: f32 = 0.3;

/// Which post of a portal its tag stands beside, as the portal faces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    const fn sign(self) -> f32 {
        match self {
            Self::Left => -1.0,
            Self::Right => 1.0,
        }
    }
}

/// Where a tag stands and what its tether reaches for, if anything.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    /// The bottom centre of the pane.
    pub at:     Vec3,
    /// The end of the tether, on the portal's frame. A tag over the top
    /// edge has none: it stands on the frame already, and a tether to it
    /// would run behind the pane.
    pub tether: Option<Vec3>,
}

impl Placement {
    /// The middle of the pane, which attention measures to.
    #[must_use]
    pub fn middle(&self, size: Vec2) -> Vec3 {
        self.at + Vec3::Y * (size.y / 2.0)
    }
}

/// A portal's rectangle in the script's document frame.
#[derive(Debug, Clone, Copy)]
pub struct Frame {
    centre: Vec3,
    /// From the centre to the middle of the right edge.
    right:  Vec3,
    /// From the centre to the middle of the top edge.
    up:     Vec3,
    /// The way it faces, a unit vector.
    normal: Vec3,
}

impl Frame {
    /// The rectangle of a `width` by `height` portal in the XY plane of `at`,
    /// facing +Z.
    #[must_use]
    pub fn new(at: &Transform, width: f32, height: f32) -> Self {
        Self {
            centre: at.translation,
            right:  at.rotation * (at.scale * Vec3::new(width / 2.0, 0.0, 0.0)),
            up:     at.rotation * (at.scale * Vec3::new(0.0, height / 2.0, 0.0)),
            normal: at.rotation * Vec3::Z,
        }
    }

    #[must_use]
    pub const fn centre(&self) -> Vec3 {
        self.centre
    }

    /// The post nearer `eye`. `was` is the post the tag stood by last, which
    /// it keeps until the viewer is clearly nearer the other.
    #[must_use]
    pub fn side(&self, eye: Vec3, was: Option<Side>) -> Side {
        let across = (eye - self.centre).dot(self.right.normalize_or_zero());
        match was {
            Some(Side::Left) if across < SIDE_HOLD => Side::Left,
            Some(Side::Right) if across > -SIDE_HOLD => Side::Right,
            _ if across < 0.0 => Side::Left,
            _ => Side::Right,
        }
    }

    /// Where a tag of `size` stands, seen from `eye`, beside `post` when the
    /// portal is tall enough to stand beside.
    #[must_use]
    pub fn place(&self, eye: Vec3, post: Side, size: Vec2) -> Placement {
        let toward = if (eye - self.centre).dot(self.normal) < 0.0 {
            -self.normal
        } else {
            self.normal
        };
        let bottom = self.centre.y - self.up.y.abs();
        let top = self.centre.y + self.up.y.abs();

        if top - bottom < EYE_HEIGHT + HEADROOM {
            let edge = self.centre + self.up;
            return Placement {
                at:     edge + Vec3::Y * CLEARANCE + toward * OUT,
                tether: None,
            };
        }

        let outward = self.right.normalize_or_zero() * post.sign();
        let edge = self.centre + self.right * post.sign();
        let middle = (bottom + EYE_HEIGHT).min(top - size.y / 2.0);
        let mut at = edge + outward * (SIDE_GAP + size.x / 2.0) + toward * OUT;
        at.y = middle - size.y / 2.0;
        let mut tether = edge + toward * DOT_OUT;
        tether.y = middle;
        Placement {
            at,
            tether: Some(tether),
        }
    }
}

#[cfg(test)]
mod tests {
    use wired_guest::math::Quat;

    use super::*;

    const TAG: Vec2 = Vec2::new(0.8, 0.3);

    /// The gate: 2 m wide and 2.6 m tall, standing on the floor at the origin,
    /// facing +Z.
    fn gate() -> Frame {
        let at = Transform {
            translation: Vec3::new(0.0, 1.3, 0.0),
            rotation:    Quat::IDENTITY,
            scale:       Vec3::ONE,
        };
        Frame::new(&at, 2.0, 2.6)
    }

    #[test]
    fn beside_it_stands_at_eye_height_clear_of_the_opening_on_the_viewer_s_side() {
        let eye = Vec3::new(0.5, 1.6, 3.0);
        let placed = gate().place(eye, Side::Right, TAG);
        let middle = placed.middle(TAG);

        assert!((middle.y - EYE_HEIGHT).abs() < 1.0e-5);
        assert!(
            placed.at.x - TAG.x / 2.0 > 1.0,
            "the pane overlaps the opening"
        );
        assert!(placed.at.z > 0.0);
        let tether = placed.tether.expect("a tag beside a post is tied to it");
        assert!((tether.y - middle.y).abs() < 1.0e-5);
        assert!((tether.x - 1.0).abs() < 1.0e-5);
    }

    #[test]
    fn seen_from_behind_it_stands_behind() {
        let placed = gate().place(Vec3::new(0.0, 1.6, -3.0), Side::Left, TAG);
        assert!(placed.at.z < 0.0);
        assert!(placed.at.x < -1.0);
    }

    #[test]
    fn it_takes_the_nearer_post_and_holds_it_while_the_viewer_sways() {
        let gate = gate();
        assert_eq!(gate.side(Vec3::new(-0.5, 1.6, 3.0), None), Side::Left);
        assert_eq!(gate.side(Vec3::new(0.5, 1.6, 3.0), None), Side::Right);
        assert_eq!(
            gate.side(Vec3::new(0.2, 1.6, 3.0), Some(Side::Left)),
            Side::Left
        );
        assert_eq!(
            gate.side(Vec3::new(0.5, 1.6, 3.0), Some(Side::Left)),
            Side::Right
        );
    }

    #[test]
    fn a_portal_too_short_to_stand_beside_takes_its_tag_over_the_top() {
        let at = Transform {
            translation: Vec3::new(0.0, 0.6, 0.0),
            rotation:    Quat::IDENTITY,
            scale:       Vec3::ONE,
        };
        let hatch = Frame::new(&at, 1.0, 1.2);
        let placed = hatch.place(Vec3::new(0.0, 1.6, 2.0), Side::Right, TAG);
        assert!(placed.at.y > 1.2);
        assert!(placed.at.x.abs() < 1.0e-5);
        assert_eq!(placed.tether, None);
    }

    #[test]
    fn a_tall_tag_never_rises_past_the_top_edge() {
        let placed = gate().place(Vec3::new(1.0, 1.6, 3.0), Side::Right, Vec2::new(0.8, 2.4));
        assert!(placed.at.y + 2.4 <= 2.6 + 1.0e-5);
    }
}
