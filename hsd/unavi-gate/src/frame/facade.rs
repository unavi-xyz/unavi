//! The white architecture, Refined Doric: one stylobate, two plain shafts
//! each with a base block and a capital slab, and an entablature of
//! architrave, recessed frieze and cornice.
//!
//! The stylobate, shafts and entablature bands are solid. The base blocks and
//! capitals are trim, so nothing catches on them. The jambs carry no
//! moulding, which the trace runs along.

use wired_guest::math::Vec3;

use super::{
    BASE_PROUD,
    FRONT,
    PILLAR_INNER,
    PILLAR_OUTER,
    PORTAL_HEIGHT,
    STYLOBATE_FRONT,
    STYLOBATE_HEIGHT,
    STYLOBATE_LEFT,
    STYLOBATE_RIGHT,
    material,
    span,
    trim_span,
};
use crate::Batch;

const BASE_HEIGHT: f32 = 0.1;
const CAPITAL_HEIGHT: f32 = 0.12;
const CAPITAL_PROUD: f32 = 0.03;

/// How far the architrave runs past each pillar's outer face.
const ARCHITRAVE_OVERHANG: f32 = 0.12;
const ARCHITRAVE_HEIGHT: f32 = 0.3;
/// How far the frieze is set back from the architrave's faces.
const FRIEZE_RECESS: f32 = 0.08;
const FRIEZE_HEIGHT: f32 = 0.26;
/// How far the cornice runs past each pillar's outer face.
const CORNICE_OVERHANG: f32 = 0.2;
const CORNICE_PROUD: f32 = 0.1;
const CORNICE_HEIGHT: f32 = 0.1;

pub fn build(batch: Batch<'_>, root: (u64, u64)) -> anyhow::Result<Batch<'_>> {
    let batch = pillars(batch, root)?;
    entablature(batch, root)
}

fn pillars(mut batch: Batch<'_>, root: (u64, u64)) -> anyhow::Result<Batch<'_>> {
    let white = material::white();
    batch = span(
        batch,
        root,
        Vec3::new(STYLOBATE_LEFT, 0.0, -STYLOBATE_FRONT),
        Vec3::new(STYLOBATE_RIGHT, STYLOBATE_HEIGHT, STYLOBATE_FRONT),
        white,
    )?;
    let capital_foot = PORTAL_HEIGHT - CAPITAL_HEIGHT;
    for side in [-1.0_f32, 1.0] {
        batch = span(
            batch,
            root,
            Vec3::new(side * PILLAR_INNER, STYLOBATE_HEIGHT, -FRONT),
            Vec3::new(side * PILLAR_OUTER, PORTAL_HEIGHT, FRONT),
            white,
        )?;
        batch = trim_span(
            batch,
            root,
            Vec3::new(side * PILLAR_INNER, STYLOBATE_HEIGHT, -FRONT - BASE_PROUD),
            Vec3::new(
                side * (PILLAR_OUTER + BASE_PROUD),
                STYLOBATE_HEIGHT + BASE_HEIGHT,
                FRONT + BASE_PROUD,
            ),
            white,
        )?;
        batch = trim_span(
            batch,
            root,
            Vec3::new(side * PILLAR_INNER, capital_foot, -FRONT - CAPITAL_PROUD),
            Vec3::new(
                side * (PILLAR_OUTER + CAPITAL_PROUD),
                PORTAL_HEIGHT,
                FRONT + CAPITAL_PROUD,
            ),
            white,
        )?;
    }
    Ok(batch)
}

fn entablature(mut batch: Batch<'_>, root: (u64, u64)) -> anyhow::Result<Batch<'_>> {
    let white = material::white();
    let mut top = PORTAL_HEIGHT;
    let reach = PILLAR_OUTER + ARCHITRAVE_OVERHANG;
    batch = span(
        batch,
        root,
        Vec3::new(-reach, top, -FRONT),
        Vec3::new(reach, top + ARCHITRAVE_HEIGHT, FRONT),
        white,
    )?;
    top += ARCHITRAVE_HEIGHT;

    let depth = FRONT - FRIEZE_RECESS;
    batch = span(
        batch,
        root,
        Vec3::new(-reach, top, -depth),
        Vec3::new(reach, top + FRIEZE_HEIGHT, depth),
        white,
    )?;
    top += FRIEZE_HEIGHT;

    let ledge = PILLAR_OUTER + CORNICE_OVERHANG;
    let ledge_depth = FRONT + CORNICE_PROUD;
    span(
        batch,
        root,
        Vec3::new(-ledge, top, -ledge_depth),
        Vec3::new(ledge, top + CORNICE_HEIGHT, ledge_depth),
        white,
    )
}
