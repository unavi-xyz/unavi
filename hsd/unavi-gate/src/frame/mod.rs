//! The gate's body, built once at start: white pillars under a stacked
//! lintel, and a black stone beside the left pillar with the socket sunk into
//! its top.
//!
//! Every part shares one front plane. The socket prim is authored at the seat
//! point, so everything of the socket is built beneath it.

mod facade;
pub mod material;
mod socket;
mod trace;

use std::f32::consts::GOLDEN_RATIO;

use wired_guest::{
    beacon::{
        BEACON_SIZE,
        Backing,
    },
    math::{
        Transform,
        Vec3,
    },
};

use self::material::{
    NEUTRAL,
    glow,
};
use crate::{
    Batch,
    unavi::shapes::api::Cuboid,
    wired::scene::{
        document::Document,
        properties::{
            Material,
            Property,
            RigidBody,
        },
    },
};

pub const PORTAL_WIDTH: f32 = GOLDEN_RATIO;
pub const PORTAL_HEIGHT: f32 = PORTAL_WIDTH * GOLDEN_RATIO;

const PILLAR_WIDTH: f32 = 0.32;
const PILLAR_DEPTH: f32 = 0.5;
/// The front plane every part of the facade shares, on the gate's Z axis.
const FRONT: f32 = PILLAR_DEPTH / 2.0;
const PLINTH_HEIGHT: f32 = 0.12;
/// How far each plinth stands out past what it carries, except at the jambs.
const PLINTH_PROUD: f32 = 0.05;
const LINTEL_HEIGHT: f32 = 0.34;
/// Each pillar's inner and outer face, on the gate's X axis, right side.
const PILLAR_INNER: f32 = PORTAL_WIDTH / 2.0;
const PILLAR_OUTER: f32 = PILLAR_INNER + PILLAR_WIDTH;

const STONE_WIDTH: f32 = 0.26;
const STONE_GAP: f32 = 0.06;
/// Seat point down to the recess floor: half a beacon.
const SEAT_FLOOR: f32 = BEACON_SIZE / 2.0;

pub const ORBITERS: usize = 3;
/// Seat point to the foot of the shaft, which rises from the recess floor.
pub const SHAFT_FOOT: f32 = -SEAT_FLOOR;

/// The socket's backing as a beacon drawn in sees it: the left pillar, past
/// the gap beside the stone, as deep as the pillar about the seat.
pub const fn backing() -> Backing {
    Backing {
        from:       STONE_WIDTH / 2.0 + STONE_GAP,
        half_depth: PILLAR_DEPTH - STONE_WIDTH / 2.0,
    }
}

/// One straight run of the trace: a unit line under a pivot at its start,
/// scaled along `axis` to grow.
pub struct Run {
    pub pivot:  (u64, u64),
    /// Where the pivot stands, in the gate's frame.
    pub from:   Vec3,
    pub line:   (u64, u64),
    /// Unit length, pointing along the run.
    pub axis:   Vec3,
    pub length: f32,
}

/// The prims the look animates.
pub struct Parts {
    /// The square around the recess and both crosses.
    pub marks:    Vec<(u64, u64)>,
    pub trace:    Vec<Run>,
    pub shaft:    (u64, u64),
    pub orbiters: [(u64, u64); ORBITERS],
    pub ghost:    (u64, u64),
}

fn place(
    batch: Batch<'_>,
    prim: (u64, u64),
    parent: (u64, u64),
    at: Vec3,
    material: Material,
) -> Batch<'_> {
    batch
        .set(prim, Property::Parent(Some(parent)))
        .set(prim, Property::Material(material))
        .set(prim, Property::Transform(Transform::from_translation(at)))
}

/// A box of `size` centred on `at`, seen but not solid.
fn trim(
    batch: Batch<'_>,
    parent: (u64, u64),
    size: Vec3,
    at: Vec3,
    material: Material,
) -> anyhow::Result<Batch<'_>> {
    let prim = Cuboid::new(size).mesh()?;
    Ok(place(batch, prim, parent, at, material))
}

/// A static box of `size` centred on `at`.
fn block(
    batch: Batch<'_>,
    parent: (u64, u64),
    size: Vec3,
    at: Vec3,
    material: Material,
) -> anyhow::Result<Batch<'_>> {
    let shape = Cuboid::new(size);
    let prim = shape.mesh()?;
    Ok(place(batch, prim, parent, at, material)
        .set(prim, Property::Collider(shape.collider()))
        .set(prim, Property::RigidBody(RigidBody::static_body())))
}

/// A static box spanning the corners `a` and `b`.
fn span(
    batch: Batch<'_>,
    parent: (u64, u64),
    a: Vec3,
    b: Vec3,
    material: Material,
) -> anyhow::Result<Batch<'_>> {
    block(batch, parent, (b - a).abs(), a.midpoint(b), material)
}

/// A seen, not solid, box spanning the corners `a` and `b`.
fn trim_span(
    batch: Batch<'_>,
    parent: (u64, u64),
    a: Vec3,
    b: Vec3,
    material: Material,
) -> anyhow::Result<Batch<'_>> {
    trim(batch, parent, (b - a).abs(), a.midpoint(b), material)
}

/// Builds the gate and answers the prims the look animates. `seat` is where
/// the socket prim stands in the gate's frame.
pub fn build(
    doc: &Document,
    root: (u64, u64),
    socket: (u64, u64),
    seat: Vec3,
) -> anyhow::Result<Parts> {
    let batch = facade::build(doc.local(), root)?;
    let dark = glow(NEUTRAL, 0.0);
    let (batch, socket_parts) = socket::build(doc, batch, root, socket, seat, dark)?;
    let (batch, trace) = trace::build(doc, batch, root, seat, dark)?;
    batch.flush()?;
    Ok(Parts {
        marks: socket_parts.marks,
        trace,
        shaft: socket_parts.shaft,
        orbiters: socket_parts.orbiters,
        ghost: socket_parts.ghost,
    })
}
