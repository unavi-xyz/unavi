//! The trace: a line of light from the socket down the stone's face, out
//! along the plinths to the doorway and down the plinth's end, then around
//! the opening's front edge, up the left jamb, across the soffit, down the
//! right jamb and back across the threshold.

use wired_guest::math::{
    Quat,
    Transform,
    Vec3,
};

use super::{
    FRONT,
    PILLAR_INNER,
    PLINTH_HEIGHT,
    PLINTH_PROUD,
    PORTAL_HEIGHT,
    Run,
    place,
    socket::CROSS_FOOT,
};
use crate::{
    Batch,
    unavi::shapes::api::Cuboid,
    wired::scene::{
        document::{
            Document,
            Layer,
        },
        properties::{
            Material,
            Property,
        },
    },
};

const WIDTH: f32 = 0.018;
/// Half the trace's width: how far its centre stands off the face it runs on,
/// and how far each run reaches past its corners so neighbours meet square.
const HALF: f32 = WIDTH / 2.0;

/// The corners of the trace, in the gate's frame.
pub fn corners(seat: Vec3) -> [Vec3; 10] {
    let face = FRONT + HALF;
    // Along the plinths' tops, in front of the pillars' base bands.
    let ledge = FRONT + PLINTH_PROUD * 0.7;
    let plinth = PLINTH_HEIGHT + HALF;
    let jamb = FRONT - HALF;
    let left = -PILLAR_INNER + HALF;
    let right = PILLAR_INNER - HALF;
    let top = PORTAL_HEIGHT - HALF;
    [
        Vec3::new(seat.x, seat.y - CROSS_FOOT, face),
        Vec3::new(seat.x, plinth, face),
        Vec3::new(seat.x, plinth, ledge),
        Vec3::new(left, plinth, ledge),
        Vec3::new(left, HALF, ledge),
        Vec3::new(left, HALF, jamb),
        Vec3::new(left, top, jamb),
        Vec3::new(right, top, jamb),
        Vec3::new(right, HALF, jamb),
        Vec3::new(left, HALF, jamb),
    ]
}

pub fn build<'a>(
    doc: &Document,
    mut batch: Batch<'a>,
    root: (u64, u64),
    seat: Vec3,
    dark: Material,
) -> anyhow::Result<(Batch<'a>, Vec<Run>)> {
    let corners = corners(seat);
    let mut runs = Vec::with_capacity(corners.len() - 1);
    for pair in corners.windows(2) {
        let (start, end) = (pair[0], pair[1]);
        let axis = (end - start).normalize_or_zero();
        // Each run reaches half a width past both of its corners.
        let from = start - axis * HALF;
        let length = start.distance(end) + WIDTH;
        let pivot = doc.create_prim(Layer::Local, Some(root))?;
        let size = Vec3::splat(WIDTH) + axis.abs() * (1.0 - WIDTH);
        let line = Cuboid::new(size).mesh()?;
        batch = batch.set(
            pivot,
            Property::Transform(Transform::new(from, Quat::IDENTITY, Vec3::splat(1.0e-4))),
        );
        batch = place(batch, line, pivot, axis * 0.5, dark);
        runs.push(Run {
            pivot,
            from,
            line,
            axis,
            length,
        });
    }
    Ok((batch, runs))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEAT: Vec3 = Vec3::new(-1.319_017, 0.9975, 0.12);

    #[test]
    fn the_trace_turns_only_at_right_angles() {
        for pair in corners(SEAT).windows(2) {
            let run = pair[1] - pair[0];
            let axes = [run.x, run.y, run.z]
                .iter()
                .filter(|v| v.abs() > 1.0e-5)
                .count();
            assert_eq!(
                axes, 1,
                "a run from {:?} to {:?} is diagonal",
                pair[0], pair[1]
            );
        }
    }

    #[test]
    fn the_trace_closes_its_loop_around_the_opening() {
        let corners = corners(SEAT);
        assert_eq!(
            corners[5], corners[9],
            "the threshold run ends where the loop began"
        );
    }
}
