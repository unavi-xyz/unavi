//! The gate's body, built at start: white pillars under a three-band
//! entablature on one stylobate, and a black stone beside the left pillar with
//! the socket sunk into its top.
//!
//! The stone and everything of the socket are built beneath the socket prim,
//! so moving that prim moves the stone. The trace is computed from where the
//! socket stands and rebuilt when it moves.

mod facade;
pub mod material;
mod route;
mod socket;
pub mod trace;

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

use self::{
    material::{
        NEUTRAL,
        glow,
    },
    trace::Trace,
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
/// The front plane of the pillars and the jambs, on the gate's Z axis.
const FRONT: f32 = PILLAR_DEPTH / 2.0;
/// The one datum: the top of the stylobate, which every horizontal run of the
/// trace lies in.
const STYLOBATE_HEIGHT: f32 = 0.12;
/// How far the stylobate stands out past the pillars' base blocks, front and
/// back.
const STYLOBATE_PROUD: f32 = 0.05;
/// How far a pillar's base block and capital stand out from its shaft, except
/// at the jamb.
const BASE_PROUD: f32 = 0.02;
/// The front plane of the pillars' base blocks.
const BASE_FRONT: f32 = FRONT + BASE_PROUD;
/// The stylobate's front and back planes.
const STYLOBATE_FRONT: f32 = FRONT + STYLOBATE_PROUD;
/// Each pillar's inner and outer face, on the gate's X axis, right side.
const PILLAR_INNER: f32 = PORTAL_WIDTH / 2.0;
const PILLAR_OUTER: f32 = PILLAR_INNER + PILLAR_WIDTH;

const STONE_WIDTH: f32 = 0.26;
const STONE_GAP: f32 = 0.06;
/// Seat point down to the recess floor: half a beacon.
const SEAT_FLOOR: f32 = BEACON_SIZE / 2.0;
/// The stylobate's left end, far enough out to carry the stone at its default
/// seat.
const STYLOBATE_LEFT: f32 = -(PILLAR_OUTER + STONE_GAP + STONE_WIDTH + STYLOBATE_PROUD);
const STYLOBATE_RIGHT: f32 = PILLAR_OUTER + STYLOBATE_PROUD;

pub const ORBITERS: usize = 3;
/// Seat point to the foot of the shaft, which rises from the recess floor.
pub const SHAFT_FOOT: f32 = -SEAT_FLOOR;

/// The socket's backing as a beacon drawn in sees it: the left pillar with its
/// base block, as deep as the pillar about the seat.
pub fn backing(seat: Vec3) -> Backing {
    Backing {
        from:       (-PILLAR_OUTER - BASE_PROUD - seat.x).max(0.0),
        half_depth: BASE_FRONT + seat.z.abs(),
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
    pub trace:    Trace,
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
    let (batch, socket_parts) = socket::build(doc, batch, socket, seat, dark)?;
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

/// Replaces `old` with a trace routed from a socket now standing at `seat`.
pub fn retrace(doc: &Document, root: (u64, u64), seat: Vec3, old: &Trace) -> anyhow::Result<Trace> {
    let dark = glow(NEUTRAL, 0.0);
    let (batch, trace) = trace::build(doc, doc.local(), root, seat, dark)?;
    old.remove(batch).flush()?;
    Ok(trace)
}
