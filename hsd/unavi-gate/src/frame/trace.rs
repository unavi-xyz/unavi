//! The trace's prims: the routed runs from the stone, the loop around the
//! opening, and the paving a moved stone's runs are inlaid in.

use wired_guest::math::{
    Quat,
    Transform,
    Vec3,
};

use super::{
    Run,
    material,
    place,
    route::{
        self,
        HALF,
        PAVING_HEIGHT,
        WIDTH,
    },
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

/// How far paving reaches past the trace on each side.
const PAVING_MARGIN: f32 = 0.02;

/// The runs in drawing order, routed ones first, and the paving under them.
#[derive(Default)]
pub struct Trace {
    pub runs: Vec<Run>,
    paving:   Vec<(u64, u64)>,
}

impl Trace {
    /// Queues taking every prim of this trace out of the scene.
    pub fn remove<'a>(&self, batch: Batch<'a>) -> Batch<'a> {
        let batch = self
            .runs
            .iter()
            .fold(batch, |batch, run| batch.remove(run.pivot));
        self.paving
            .iter()
            .fold(batch, |batch, &slab| batch.remove(slab))
    }
}

pub fn build<'a>(
    doc: &Document,
    mut batch: Batch<'a>,
    root: (u64, u64),
    seat: Vec3,
    dark: Material,
) -> anyhow::Result<(Batch<'a>, Trace)> {
    let route = route::route(seat);
    let mut paving = Vec::new();
    for (low, high) in route::grounded(&route) {
        let from = Vec3::new(low.x - PAVING_MARGIN, 0.0, low.z - PAVING_MARGIN);
        let to = Vec3::new(high.x + PAVING_MARGIN, PAVING_HEIGHT, high.z + PAVING_MARGIN);
        let slab = Cuboid::new((to - from).abs()).mesh()?;
        batch = place(batch, slab, root, from.midpoint(to), material::white());
        paving.push(slab);
    }
    let corners = route.iter().chain(&route::opening()[1..]).copied().collect::<Vec<_>>();
    let mut runs = Vec::with_capacity(corners.len());
    for pair in corners.windows(2) {
        let (start, end) = (pair[0], pair[1]);
        let axis = (end - start).normalize_or_zero();
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
    Ok((batch, Trace { runs, paving }))
}
