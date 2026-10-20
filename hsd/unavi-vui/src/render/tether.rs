//! Transcribes a [`Tether`] into the two prims it is drawn with.

use wired_guest::math::Color;

use crate::{
    mesh,
    render::{
        draw,
        shadow::Shadow,
    },
    tether::Tether,
    wired::scene::{
        document::{
            Document,
            Layer,
        },
        properties::Property,
    },
};

/// Rings and segments of the dot's sphere: small, since it is a few
/// centimetres across.
const DOT_DETAIL: (usize, usize) = (8, 12);

pub struct TetherPrims {
    bar:  (u64, u64),
    dot:  (u64, u64),
    last: Shadow<Option<Tether>>,
    tint: Shadow<Color>,
}

impl TetherPrims {
    /// Two prims in the document's own frame, since a tether is placed in it.
    pub fn new(doc: &Document) -> anyhow::Result<Self> {
        let bar = doc.create_prim(Layer::Local, None)?;
        let dot = doc.create_prim(Layer::Local, None)?;
        let mut batch = doc.local();
        for (prim, data) in [
            (bar, mesh::bar()),
            (dot, mesh::sphere(1.0, DOT_DETAIL.0, DOT_DETAIL.1)),
        ] {
            batch = draw::mesh(&data)
                .into_iter()
                .fold(batch, |batch, property| batch.set(prim, property))
                .set(prim, Property::Transform(draw::hidden()));
        }
        batch.flush()?;
        Ok(Self {
            bar,
            dot,
            last: Shadow::new(),
            tint: Shadow::new(),
        })
    }

    /// Draws `tether` in `color`, or puts it away.
    pub fn update(
        &self,
        doc: &Document,
        tether: Option<Tether>,
        color: Color,
        glow: f32,
    ) -> anyhow::Result<()> {
        let mut batch = doc.local();
        if let Some(color) = self.tint.diff(color) {
            for prim in [self.bar, self.dot] {
                batch = batch.set(prim, Property::Material(draw::beam(color, glow)));
            }
        }
        if let Some(tether) = self.last.diff(tether) {
            let (bar, dot) = tether.map_or_else(
                || (draw::hidden(), draw::hidden()),
                |tether| (tether.bar.unwrap_or_else(draw::hidden), tether.dot),
            );
            batch = batch
                .set(self.bar, Property::Transform(bar))
                .set(self.dot, Property::Transform(dot));
        }
        batch.flush()?;
        Ok(())
    }

    pub fn remove(&self, doc: &Document) -> anyhow::Result<()> {
        doc.local().remove(self.bar).remove(self.dot).flush()?;
        Ok(())
    }
}
