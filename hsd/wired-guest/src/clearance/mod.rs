//! Moves an upright pane facing an eye out of whatever it stands in or behind.
//!
//! It rises off a floor beneath it, comes toward the eye past anything in
//! front of it, and steps toward the eye again while it still cuts into
//! something.
//!
//! The caller owns the physics. A cast from one point toward another answers
//! the distance to the first collider on the way, or `None` for a clear line.

use crate::math::{
    Vec2,
    Vec3,
};

/// The middle and four corners of a pane.
const PROBES: usize = 5;
/// Metres between a pane and the nearest thing in front of or beneath it.
const MARGIN: f32 = 0.02;
/// Metres from the eye a pane is never pulled closer than, so a pane is never
/// pushed into the viewer's face.
const NEAREST: f32 = 0.8;
/// Metres a pane cutting into something steps toward the eye at a time, and
/// the most steps it takes.
const SETTLE_STEP: f32 = 0.15;
const SETTLE_STEPS: usize = 6;
/// Metres a moved position is rounded to, so small changes of eye do not
/// resend it.
const STEP: f32 = 0.005;

/// Where a `size` pane wanted with its bottom centre at `foot` stands as seen
/// from `eye`, as its new bottom centre.
pub fn clear(
    eye: Vec3,
    foot: Vec3,
    size: Vec2,
    mut cast: impl FnMut(Vec3, Vec3) -> Option<f32>,
) -> Vec3 {
    let lifted = lifted(eye, foot, size, &mut cast);
    let probes = probes(eye, lifted, size);
    let mut hits = [None; PROBES];
    for (hit, probe) in hits.iter_mut().zip(&probes) {
        *hit = cast(eye, *probe);
    }
    let seen = pulled(eye, lifted, &probes, &hits);
    settle(eye, seen, size, &mut cast)
}

/// `foot` raised until a margin of air lies under each of the pane's bottom
/// corners. Each cast runs down from the top corner above it, so a floor the
/// pane's lower edge sits in is found from outside it.
fn lifted(
    eye: Vec3,
    foot: Vec3,
    size: Vec2,
    cast: &mut impl FnMut(Vec3, Vec3) -> Option<f32>,
) -> Vec3 {
    let [_, low_left, low_right, high_left, high_right] = probes(eye, foot, size);
    let reach = size.y + MARGIN;
    let mut rise = 0.0_f32;
    for (high, low) in [(high_left, low_left), (high_right, low_right)] {
        if let Some(hit) = cast(high, low - Vec3::Y * MARGIN) {
            rise = rise.max(reach - hit);
        }
    }
    if rise <= 0.0 {
        return foot;
    }
    snap(foot + Vec3::Y * rise)
}

/// Steps a pane that cuts into something toward the eye until it no longer
/// does. Rays from the eye miss a pane that slices into a wall at a slant, so
/// this casts from the pane's middle out to its edges.
fn settle(
    eye: Vec3,
    foot: Vec3,
    size: Vec2,
    cast: &mut impl FnMut(Vec3, Vec3) -> Option<f32>,
) -> Vec3 {
    let offset = foot - eye;
    let length = offset.length();
    if length < f32::EPSILON {
        return foot;
    }
    let floor = (NEAREST / length).min(1.0);
    let mut ratio = 1.0_f32;
    let mut placed = foot;
    for _ in 0..SETTLE_STEPS {
        if !cuts(eye, placed, size, cast) || ratio <= floor {
            return placed;
        }
        ratio = (ratio - SETTLE_STEP / length).max(floor);
        placed = snap(eye + offset * ratio);
    }
    placed
}

/// Whether anything lies between the pane's middle and its edges.
fn cuts(
    eye: Vec3,
    foot: Vec3,
    size: Vec2,
    cast: &mut impl FnMut(Vec3, Vec3) -> Option<f32>,
) -> bool {
    let [middle, corners @ ..] = probes(eye, foot, size);
    let sides = [
        corners[0].midpoint(corners[2]),
        corners[1].midpoint(corners[3]),
    ];
    corners
        .into_iter()
        .chain(sides)
        .any(|edge| cast(middle, edge).is_some())
}

/// The middle, then the bottom left, bottom right, top left and top right
/// corners of a `size` pane with its bottom centre at `foot`, facing `eye`
/// about the vertical axis.
fn probes(eye: Vec3, foot: Vec3, size: Vec2) -> [Vec3; PROBES] {
    let ahead = Vec3::new(foot.x - eye.x, 0.0, foot.z - eye.z).normalize_or_zero();
    let ahead = if ahead == Vec3::ZERO {
        Vec3::NEG_Z
    } else {
        ahead
    };
    let half = ahead.cross(Vec3::Y) * (size.x / 2.0);
    let top = Vec3::Y * size.y;
    [
        foot + top / 2.0,
        foot - half,
        foot + half,
        foot - half + top,
        foot + half + top,
    ]
}

/// `foot` moved toward `eye` along the line between them, so that every probe
/// a hit lies on or in front of ends up a margin short of it. `hits` are
/// distances from `eye` along the way to each probe.
fn pulled(eye: Vec3, foot: Vec3, probes: &[Vec3; PROBES], hits: &[Option<f32>; PROBES]) -> Vec3 {
    let mut ratio = 1.0_f32;
    for (probe, hit) in probes.iter().zip(hits) {
        let Some(hit) = hit else {
            continue;
        };
        let reach = eye.distance(*probe);
        // A hit at the probe is a pane flush with or already into the surface.
        // Asking only for hits well in front leaves a sliver of the pane
        // buried, so pull for anything short of a margin clear of it.
        let wanted = hit - MARGIN;
        if wanted < reach {
            ratio = ratio.min(wanted / reach);
        }
    }
    if ratio >= 1.0 {
        return foot;
    }
    let offset = foot - eye;
    let floor = (NEAREST / offset.length()).min(1.0);
    snap(eye + offset * ratio.max(floor))
}

fn snap(at: Vec3) -> Vec3 {
    let round = |metres: f32| (metres / STEP).round() * STEP;
    Vec3::new(round(at.x), round(at.y), round(at.z))
}

#[cfg(test)] mod tests;
