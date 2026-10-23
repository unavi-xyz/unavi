//! Where the trace runs: from the foot of the cross on the stone's front face
//! to the loop around the opening, as right-angled runs.
//!
//! Every centre lies on the surface it is inlaid in, so a vertical run is
//! half a width proud of its face and a horizontal run half a width proud of
//! its top. A stone standing on the stylobate has its runs in the stylobate's
//! top. A stone moved off it has them in a paving strip on the ground, which
//! climb the stylobate's front face to its top.

use wired_guest::math::Vec3;

use super::{
    BASE_FRONT,
    FRONT,
    PILLAR_INNER,
    PORTAL_HEIGHT,
    STONE_WIDTH,
    STYLOBATE_FRONT,
    STYLOBATE_HEIGHT,
    STYLOBATE_LEFT,
    STYLOBATE_RIGHT,
    socket::CROSS_FOOT,
};

pub const WIDTH: f32 = 0.018;
/// Half the trace's width: how far a centre stands off the face it lies in,
/// and how far each run reaches past its corners so neighbours meet square.
pub const HALF: f32 = WIDTH / 2.0;
/// Height of the paving strip a moved stone's runs are inlaid in.
pub const PAVING_HEIGHT: f32 = 0.01;

/// Where the route joins the loop: the sill's left corner, on the front edge
/// of the opening.
pub fn join() -> Vec3 {
    Vec3::new(HALF - PILLAR_INNER, STYLOBATE_HEIGHT, FRONT - HALF)
}

/// The loop around the opening, from and back to [`join`]: up the left jamb,
/// across the soffit, down the right jamb and across the sill.
pub fn opening() -> [Vec3; 5] {
    let start = join();
    let top = PORTAL_HEIGHT - HALF;
    let right = PILLAR_INNER - HALF;
    [
        start,
        Vec3::new(start.x, top, start.z),
        Vec3::new(right, top, start.z),
        Vec3::new(right, start.y, start.z),
        start,
    ]
}

/// Whether a stone whose socket stands at `seat` stands on the stylobate's
/// top, with room for the trace down its front face.
pub fn on_stylobate(seat: Vec3) -> bool {
    let half = STONE_WIDTH / 2.0;
    seat.x - half >= STYLOBATE_LEFT
        && seat.x + half <= STYLOBATE_RIGHT
        && seat.z - half >= -STYLOBATE_FRONT
        && seat.z + half + WIDTH <= STYLOBATE_FRONT
}

/// The height the stone's foot stands at.
pub fn foot(seat: Vec3) -> f32 {
    if on_stylobate(seat) {
        STYLOBATE_HEIGHT
    } else {
        0.0
    }
}

/// The corners from the foot of the cross at `seat` to [`join`].
///
/// The stone's front face is followed down to its foot, then along depth only
/// when the stone's front is not already on the line the trace runs along, then
/// along X to the left jamb's line. The last run is the short step back to the
/// opening's front edge.
pub fn route(seat: Vec3) -> Vec<Vec3> {
    let on = on_stylobate(seat);
    let floor = if on { STYLOBATE_HEIGHT } else { PAVING_HEIGHT };
    let face = seat.z + STONE_WIDTH / 2.0 + HALF;
    let line = if on {
        BASE_FRONT + HALF
    } else {
        STYLOBATE_FRONT + HALF
    };
    let run = face.max(line);
    let join = join();

    let mut corners = vec![
        Vec3::new(seat.x, (seat.y - CROSS_FOOT).max(floor), face),
        Vec3::new(seat.x, floor, face),
        Vec3::new(seat.x, floor, run),
        Vec3::new(join.x, floor, run),
    ];
    if !on {
        corners.push(Vec3::new(join.x, floor, line));
        corners.push(Vec3::new(join.x, STYLOBATE_HEIGHT, line));
    }
    corners.push(join);
    corners.dedup_by(|b, a| a.distance(*b) < 1.0e-4);
    corners
}

/// The stretches of `route` that lie on the ground, as the lowest and
/// highest corner of each.
pub fn grounded(corners: &[Vec3]) -> Vec<(Vec3, Vec3)> {
    corners
        .windows(2)
        .filter(|pair| pair.iter().all(|c| (c.y - PAVING_HEIGHT).abs() < 1.0e-4))
        .map(|pair| (pair[0].min(pair[1]), pair[0].max(pair[1])))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT: Vec3 = Vec3::new(-1.319_017, 0.9975, 0.14);
    /// Behind the stylobate's back edge and left of its end.
    const OFF: Vec3 = Vec3::new(-2.4, 0.9975, -0.8);
    /// In front of the stylobate.
    const FORWARD: Vec3 = Vec3::new(-1.0, 0.9975, 0.9);
    /// On the stylobate, further back than the pillars' fronts.
    const BACK: Vec3 = Vec3::new(-1.3, 1.2, 0.0);

    const SEATS: [Vec3; 4] = [DEFAULT, OFF, FORWARD, BACK];

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1.0e-5
    }

    fn axes(run: Vec3) -> usize {
        [run.x, run.y, run.z]
            .iter()
            .filter(|v| v.abs() > 1.0e-5)
            .count()
    }

    #[test]
    fn the_route_turns_only_at_right_angles() {
        for seat in SEATS {
            for pair in route(seat).windows(2) {
                assert_eq!(
                    axes(pair[1] - pair[0]),
                    1,
                    "a run from {:?} to {:?} is diagonal for {seat:?}",
                    pair[0],
                    pair[1]
                );
            }
        }
    }

    #[test]
    fn the_route_ends_where_the_loop_begins() {
        let opening = opening();
        assert_eq!(opening[0], opening[4], "the loop closes on itself");
        for seat in SEATS {
            assert_eq!(route(seat).last().copied(), Some(opening[0]));
        }
    }

    #[test]
    fn nothing_dips_below_the_ground() {
        for seat in SEATS {
            for corner in route(seat).into_iter().chain(opening()) {
                assert!(
                    corner.y - HALF >= 0.0,
                    "{corner:?} dips below the ground for {seat:?}"
                );
            }
        }
    }

    #[test]
    fn the_default_seat_runs_in_the_stylobate_top_without_a_depth_jog() {
        let corners = route(DEFAULT);
        assert!(on_stylobate(DEFAULT));
        let last = corners.len() - 1;
        let run = &corners[..last];
        assert!(
            corners[last].z < run[0].z,
            "the join steps back to the front edge"
        );
        assert!(
            run.iter().all(|c| (c.z - run[0].z).abs() < 1.0e-5),
            "the route jogs in depth before the join: {run:?}"
        );
        for pair in corners.windows(2) {
            if near(pair[0].y, pair[1].y) {
                assert!(near(pair[0].y, STYLOBATE_HEIGHT), "a run is off the top");
            }
        }
        assert_eq!(grounded(&corners).len(), 0);
    }

    #[test]
    fn a_stone_set_back_on_the_stylobate_jogs_once_in_depth_before_the_join() {
        let corners = route(BACK);
        assert!(on_stylobate(BACK));
        let depth = corners
            .windows(2)
            .filter(|p| (p[1].z - p[0].z).abs() > 1.0e-5)
            .count();
        assert_eq!(depth, 2, "one depth run, then the join");
        assert_eq!(grounded(&corners).len(), 0);
    }

    #[test]
    fn a_stone_off_the_stylobate_runs_on_paving_and_climbs_its_face() {
        for seat in [OFF, FORWARD] {
            assert!(!on_stylobate(seat));
            let corners = route(seat);
            assert_eq!(grounded(&corners).len(), 2, "{seat:?}");
            let climb = corners
                .windows(2)
                .find(|p| p[0].y < p[1].y && near(p[0].x, p[1].x) && near(p[0].z, p[1].z));
            assert!(climb.is_some(), "no climb for {seat:?}");
            if let Some(climb) = climb {
                assert!(near(climb[0].y, PAVING_HEIGHT));
                assert!(near(climb[1].y, STYLOBATE_HEIGHT));
                assert!(
                    climb[1].z >= STYLOBATE_FRONT,
                    "the climb is not up the stylobate's front face"
                );
            }
        }
    }
}
