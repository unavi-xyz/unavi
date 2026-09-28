//! Draws a cast site: a ring on the mote that is filling.

use std::cell::Cell;

use wired_prelude::prelude::*;

use crate::{
    cast::{
        self,
        State,
    },
    mesh,
    palette::Palette,
    scene::{
        draw,
        graphs,
    },
    tuning::Tuning,
    wired::scene::types::{
        Document,
        GraphValue,
        Prim,
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
    /// How much of the ring an abandoned cast has left to unwind.
    recoil:   Cell<Option<f32>>,
    speed:    f32,
    palette:  Palette,
}

impl Site {
    pub fn new(
        doc: &Document,
        parent: &Prim,
        tuning: &Tuning,
        palette: Palette,
    ) -> anyhow::Result<Self> {
        let prim = doc.create_prim()?;
        draw::mesh(&prim, &mesh::annulus(INNER, OUTER, SEGMENTS))?;
        prim.set_xform(Some(draw::hidden()))?;
        graphs::bind_ring(&prim)?;
        parent.add_child(&prim)?;

        Ok(Self {
            prim,
            progress: Cell::new(None),
            recoil: Cell::new(None),
            speed: tuning.cast_recoil,
            palette,
        })
    }

    pub fn hide(&self) -> anyhow::Result<()> {
        self.progress.set(None);
        self.recoil.set(None);
        self.prim.set_xform(Some(draw::hidden()))?;
        Ok(())
    }

    /// `at` is the mote being cast on, in the surface's own coordinates.
    pub fn apply(&self, at: Vec3, state: State) -> anyhow::Result<()> {
        match state {
            // The ring stays where it stood and unwinds from there, so what
            // draws is the fill running backwards.
            State::Aborted => {
                let Some(left) = self.progress.get().filter(|left| *left > 0.0) else {
                    return self.hide();
                };
                self.recoil.set(Some(left));
                return Ok(());
            }
            State::Committed => return self.hide(),
            State::Filling(_) => self.recoil.set(None),
        }

        self.prim.set_xform(Some(Transform {
            translation: Vec3::new(at.x, at.y, at.z + LIFT),
            rotation:    Quat::IDENTITY,
            scale:       Vec3::ONE,
        }))?;
        self.draw(state.progress())
    }

    /// Unwinds an abandoned cast, and stands the ring down once it is empty.
    /// Runs whether or not a cast is live, because an abort is exactly the
    /// moment the cast stops being one.
    pub fn step(&self, delta: f32) -> anyhow::Result<()> {
        let Some(left) = self.recoil.get() else {
            return Ok(());
        };
        let Some(left) = cast::recoil(left, delta, self.speed) else {
            return self.hide();
        };
        self.recoil.set(Some(left));
        self.draw(left)
    }

    fn draw(&self, progress: f32) -> anyhow::Result<()> {
        if self.progress.get() == Some(progress) {
            return Ok(());
        }
        self.progress.set(Some(progress));
        // The accent is spent here rather than on hover: a cast is rare, and
        // it is the one thing that must not be missed.
        self.prim.set_graph_overrides(&[
            (graphs::RING_TINT, GraphValue::Color(self.palette.accent)),
            (graphs::RING_PROGRESS, GraphValue::Float(progress)),
        ])?;
        Ok(())
    }
}
