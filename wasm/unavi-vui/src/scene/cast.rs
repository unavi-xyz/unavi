//! Draws a cast site: a ring on the mote that is filling.

use std::cell::Cell;

use wired_prelude::prelude::*;

use crate::{
    circle::Cast,
    mesh,
    palette::Palette,
    scene::{
        draw,
        graphs,
    },
    wired::scene::types::{
        Document,
        GraphValue,
        Prim,
        Xform,
    },
};

const SEGMENTS: usize = 40;
const INNER: f32 = 0.026;
const OUTER: f32 = 0.034;
/// Stands clear of the mote it rings without reaching the hit surface.
const LIFT: f32 = 0.006;

/// A ring filled by a sweep around it.
///
/// The mesh is a full annulus and the fill travels in the shader, so a cast
/// says how far along it is without also saying how big it is — which growing
/// the ring could not help doing.
pub struct Site {
    prim:     Prim,
    progress: Cell<Option<f32>>,
    palette:  Palette,
}

impl Site {
    pub fn new(doc: &Document, parent: &Prim, palette: Palette) -> anyhow::Result<Self> {
        let prim = doc.create_prim()?;
        draw::mesh(&prim, &mesh::annulus(INNER, OUTER, SEGMENTS))?;
        prim.set_xform(Some(draw::hidden()))?;
        graphs::bind_ring(&prim)?;
        parent.add_child(&prim)?;

        Ok(Self {
            prim,
            progress: Cell::new(None),
            palette,
        })
    }

    pub fn hide(&self) -> anyhow::Result<()> {
        self.prim.set_xform(Some(draw::hidden()))?;
        Ok(())
    }

    /// `at` is the mote being cast on, in the surface's own coordinates.
    pub fn apply(&self, at: Vec3, cast: Cast) -> anyhow::Result<()> {
        if cast.is_settled() {
            return self.hide();
        }
        self.prim.set_xform(Some(Xform {
            translation: Vec3::new(at.x, at.y, at.z + LIFT),
            rotation:    Quat::IDENTITY,
            scale:       Vec3::ONE,
        }))?;

        let progress = cast.progress();
        if self.progress.get() != Some(progress) {
            self.progress.set(Some(progress));
            // The accent is spent here rather than on hover: a cast is rare,
            // and it is the one thing that must not be missed.
            self.prim.set_graph_overrides(&[
                (graphs::RING_TINT, GraphValue::Color(self.palette.accent)),
                (graphs::RING_PROGRESS, GraphValue::Float(progress)),
            ])?;
        }
        Ok(())
    }
}
