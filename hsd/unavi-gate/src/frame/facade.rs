//! The white architecture: pillars on plinths, with a base band, fluting and
//! a capital; the lintel with its keystone and string courses; the cornice
//! and the crown.
//!
//! Only the plinths, pillars, lintel, cornice and crown are solid. The
//! mouldings are trim, so nothing catches on them. No moulding stands on a
//! jamb, which the trace runs along.

use wired_guest::math::Vec3;

use super::{
    FRONT,
    LINTEL_HEIGHT,
    PILLAR_INNER,
    PILLAR_OUTER,
    PLINTH_HEIGHT,
    PLINTH_PROUD,
    PORTAL_HEIGHT,
    material,
    span,
    trim,
    trim_span,
};
use crate::Batch;

/// How far the lintel runs past each pillar's outer face.
const LINTEL_OVERHANG: f32 = 0.28;
const CORNICE_HEIGHT: f32 = 0.035;
const CORNICE_PROUD: f32 = 0.05;
const CROWN_HEIGHT: f32 = 0.16;
const CROWN_SETBACK: f32 = 0.08;
const CROWN_CAP: f32 = 0.025;

const BASE_BAND: f32 = 0.04;
const BASE_PROUD: f32 = 0.02;
const CAPITAL: f32 = 0.16;
const CAPITAL_PROUD: f32 = 0.03;
const NECK: f32 = 0.03;
const NECK_PROUD: f32 = 0.012;

/// Ridges up each pillar's front and back face, in from the inner edge.
const FLUTES: [f32; 3] = [0.1, 0.18, 0.26];
const FLUTE_WIDTH: f32 = 0.016;
const FLUTE_PROUD: f32 = 0.008;

const KEYSTONE_WIDTH: f32 = 0.24;
const KEYSTONE_PROUD: f32 = 0.025;
/// The keystone starts this far above the opening, clear of the trace on the
/// soffit.
const KEYSTONE_LIFT: f32 = 0.04;
const COURSE: f32 = 0.012;
const COURSE_PROUD: f32 = 0.006;
/// String courses across the lintel's faces, up from its foot.
const COURSES: [f32; 2] = [0.07, LINTEL_HEIGHT - 0.07];

pub fn build(batch: Batch<'_>, root: (u64, u64)) -> anyhow::Result<Batch<'_>> {
    let batch = pillars(batch, root)?;
    let batch = pillar_trim(batch, root)?;
    lintel(batch, root)
}

fn pillars(mut batch: Batch<'_>, root: (u64, u64)) -> anyhow::Result<Batch<'_>> {
    let white = material::white();
    for side in [-1.0_f32, 1.0] {
        batch = span(
            batch,
            root,
            Vec3::new(side * PILLAR_INNER, PLINTH_HEIGHT, -FRONT),
            Vec3::new(side * PILLAR_OUTER, PORTAL_HEIGHT, FRONT),
            white,
        )?;
    }
    // Each plinth stands proud of its pillar except at the jamb, which runs
    // straight to the ground.
    for side in [-1.0_f32, 1.0] {
        batch = span(
            batch,
            root,
            Vec3::new(side * PILLAR_INNER, 0.0, -FRONT - PLINTH_PROUD),
            Vec3::new(
                side * (PILLAR_OUTER + PLINTH_PROUD),
                PLINTH_HEIGHT,
                FRONT + PLINTH_PROUD,
            ),
            white,
        )?;
    }
    Ok(batch)
}

/// A band around a pillar's outer, front and back faces, flush with its jamb.
fn band(
    batch: Batch<'_>,
    root: (u64, u64),
    side: f32,
    from: f32,
    height: f32,
    proud: f32,
) -> anyhow::Result<Batch<'_>> {
    trim_span(
        batch,
        root,
        Vec3::new(side * PILLAR_INNER, from, -FRONT - proud),
        Vec3::new(side * (PILLAR_OUTER + proud), from + height, FRONT + proud),
        material::white(),
    )
}

fn pillar_trim(mut batch: Batch<'_>, root: (u64, u64)) -> anyhow::Result<Batch<'_>> {
    let capital_foot = PORTAL_HEIGHT - CAPITAL;
    let neck_foot = capital_foot - NECK;
    let flute_foot = PLINTH_HEIGHT + BASE_BAND;
    let flute_height = neck_foot - flute_foot;
    for side in [-1.0_f32, 1.0] {
        batch = band(batch, root, side, PLINTH_HEIGHT, BASE_BAND, BASE_PROUD)?;
        batch = band(batch, root, side, neck_foot, NECK, NECK_PROUD)?;
        batch = band(batch, root, side, capital_foot, CAPITAL, CAPITAL_PROUD)?;
        for inset in FLUTES {
            for face in [-1.0_f32, 1.0] {
                batch = trim(
                    batch,
                    root,
                    Vec3::new(FLUTE_WIDTH, flute_height, FLUTE_PROUD),
                    Vec3::new(
                        side * (PILLAR_INNER + inset),
                        flute_foot + flute_height / 2.0,
                        face * (FRONT + FLUTE_PROUD / 2.0),
                    ),
                    material::white(),
                )?;
            }
        }
    }
    Ok(batch)
}

fn lintel(mut batch: Batch<'_>, root: (u64, u64)) -> anyhow::Result<Batch<'_>> {
    let white = material::white();
    let reach = PILLAR_OUTER + LINTEL_OVERHANG;
    let mut top = PORTAL_HEIGHT;
    batch = span(
        batch,
        root,
        Vec3::new(-reach, top, -FRONT),
        Vec3::new(reach, top + LINTEL_HEIGHT, FRONT),
        white,
    )?;
    for face in [-1.0_f32, 1.0] {
        batch = trim_span(
            batch,
            root,
            Vec3::new(-KEYSTONE_WIDTH / 2.0, top + KEYSTONE_LIFT, face * FRONT),
            Vec3::new(
                KEYSTONE_WIDTH / 2.0,
                top + LINTEL_HEIGHT,
                face * (FRONT + KEYSTONE_PROUD),
            ),
            white,
        )?;
        for course in COURSES {
            batch = trim(
                batch,
                root,
                Vec3::new(2.0 * reach, COURSE, COURSE_PROUD),
                Vec3::new(0.0, top + course, face * (FRONT + COURSE_PROUD / 2.0)),
                white,
            )?;
        }
    }
    top += LINTEL_HEIGHT;

    let ledge = reach + CORNICE_PROUD;
    let ledge_depth = FRONT + CORNICE_PROUD;
    batch = span(
        batch,
        root,
        Vec3::new(-ledge, top, -ledge_depth),
        Vec3::new(ledge, top + CORNICE_HEIGHT, ledge_depth),
        white,
    )?;
    top += CORNICE_HEIGHT;

    let crown = PILLAR_OUTER - CROWN_SETBACK;
    let crown_depth = FRONT - CROWN_SETBACK;
    batch = span(
        batch,
        root,
        Vec3::new(-crown, top, -crown_depth),
        Vec3::new(crown, top + CROWN_HEIGHT, crown_depth),
        white,
    )?;
    top += CROWN_HEIGHT;
    let cap = crown + CORNICE_PROUD / 2.0;
    let cap_depth = crown_depth + CORNICE_PROUD / 2.0;
    span(
        batch,
        root,
        Vec3::new(-cap, top, -cap_depth),
        Vec3::new(cap, top + CROWN_CAP, cap_depth),
        white,
    )
}
