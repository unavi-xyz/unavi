//! Where a standing placard's tether lies: a bar from the pane's side to a
//! dot at the thing it labels.

use wired_guest::math::{
    Quat,
    Transform,
    Vec3,
};

use crate::tuning::Tuning;

/// Shorter than this a bar has no direction worth drawing.
const MIN_LENGTH: f32 = 1.0e-4;

/// The pane a tether leaves from, as it stands now.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pane {
    /// Its middle, in the document frame.
    pub centre:     Vec3,
    /// How it is turned, so its width runs along the rotated X axis.
    pub rotation:   Quat,
    pub half_width: f32,
}

/// The transforms of the two prims a tether is drawn with. The bar is a unit
/// box along Z, and the dot a unit sphere.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tether {
    /// Absent when the pane's side is where the tether ends.
    pub bar: Option<Transform>,
    pub dot: Transform,
}

/// The point on `pane`'s side nearer `to`, at the pane's vertical middle.
#[must_use]
pub fn attach(pane: &Pane, to: Vec3) -> Vec3 {
    let right = pane.rotation * Vec3::X;
    let side = if (to - pane.centre).dot(right) >= 0.0 {
        1.0
    } else {
        -1.0
    };
    pane.centre + right * (side * pane.half_width)
}

#[must_use]
pub fn tether(pane: &Pane, to: Vec3, tuning: &Tuning) -> Tether {
    let from = attach(pane, to);
    let across = to - from;
    let length = across.length();
    let bar = (length > MIN_LENGTH).then(|| Transform {
        translation: (from + to) * 0.5,
        rotation:    aligning(across / length),
        scale:       Vec3::new(tuning.tether_width, tuning.tether_width, length),
    });
    Tether {
        bar,
        dot: Transform {
            translation: to,
            rotation:    Quat::IDENTITY,
            scale:       Vec3::splat(tuning.tether_dot),
        },
    }
}

/// The turn that takes +Z to `direction`, a unit vector.
#[must_use]
pub fn aligning(direction: Vec3) -> Quat {
    // The half-way quaternion: the axis is the cross of the two directions
    // and w is one plus their dot. Opposite directions have no unique axis,
    // so any turn half-way round does.
    let w = 1.0 + direction.z;
    if w < 1.0e-6 {
        return Quat::new(1.0, 0.0, 0.0, 0.0);
    }
    Quat::new(-direction.y, direction.x, 0.0, w).normalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: Vec3, b: Vec3) -> bool {
        (a - b).length() < 1.0e-4
    }

    fn pane() -> Pane {
        Pane {
            centre:     Vec3::new(1.0, 2.0, 3.0),
            rotation:   Quat::IDENTITY,
            half_width: 0.5,
        }
    }

    #[test]
    fn it_leaves_from_the_side_nearer_the_target() {
        let pane = pane();
        let right = attach(&pane, Vec3::new(9.0, 0.0, 3.0));
        assert!(near(right, Vec3::new(1.5, 2.0, 3.0)));
        let left = attach(&pane, Vec3::new(-9.0, 5.0, 3.0));
        assert!(near(left, Vec3::new(0.5, 2.0, 3.0)));
    }

    #[test]
    fn it_leaves_at_the_panes_vertical_middle_whatever_the_target_height() {
        let pane = pane();
        for height in [-4.0, 2.0, 40.0] {
            assert!((attach(&pane, Vec3::new(3.0, height, 3.0)).y - 2.0).abs() < 1.0e-6);
        }
    }

    #[test]
    fn the_side_follows_the_turn_of_the_pane() {
        let turned = Pane {
            rotation: Quat::from_rotation_y(1.0),
            ..pane()
        };
        let right = turned.rotation * Vec3::X;
        for side in [1.0, -1.0] {
            let from = attach(&turned, turned.centre + right * (30.0 * side));
            assert!(
                near(from, turned.centre + right * (0.5 * side)),
                "side {side}"
            );
        }
    }

    #[test]
    fn the_bar_runs_from_the_side_to_the_target_and_the_dot_sits_on_it() {
        let to = Vec3::new(5.0, 3.0, -2.0);
        let tuning = Tuning::DEFAULT;
        let laid = tether(&pane(), to, &tuning);
        let from = attach(&pane(), to);

        assert_eq!(laid.dot.translation, to);
        assert!((laid.dot.scale.x - tuning.tether_dot).abs() < 1.0e-6);

        let bar = laid.bar.expect("a bar");
        assert!(near(bar.translation, (from + to) * 0.5), "centred between");
        assert!((bar.scale.z - (to - from).length()).abs() < 1.0e-4);
        assert!((bar.scale.x - tuning.tether_width).abs() < 1.0e-6);
        let along = bar.rotation * Vec3::Z;
        assert!(near(along, (to - from).normalize()), "its length points on");
    }

    #[test]
    fn it_has_no_bar_when_the_target_is_on_the_panes_side() {
        let pane = pane();
        let on_side = attach(&pane, Vec3::new(9.0, 2.0, 3.0));
        let laid = tether(&pane, on_side, &Tuning::DEFAULT);
        assert!(laid.bar.is_none());
        assert_eq!(laid.dot.translation, on_side);
    }

    #[test]
    fn aligning_takes_z_to_any_direction() {
        for direction in [
            Vec3::Z,
            -Vec3::Z,
            Vec3::X,
            -Vec3::Y,
            Vec3::new(1.0, 2.0, -3.0).normalize(),
            Vec3::new(0.0, 0.1, -1.0).normalize(),
        ] {
            assert!(
                near(aligning(direction) * Vec3::Z, direction),
                "{direction:?}"
            );
        }
    }
}
