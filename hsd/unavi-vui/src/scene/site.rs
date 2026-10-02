//! Draws a cast site: a ring on the mote that is filling.

use std::cell::Cell;

use wired_guest::math::{
    Quat,
    Transform,
    Vec3,
};

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
    wired::{
        scene::{
            document::{
                Document,
                Layer,
            },
            properties::Property,
        },
        shading::graph::{
            GraphValue,
            set_overrides,
        },
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
    prim:     (u64, u64),
    progress: Cell<Option<f32>>,
    /// How much of the ring an abandoned cast has left to unwind.
    recoil:   Cell<Option<f32>>,
    speed:    f32,
    palette:  Palette,
}

impl Site {
    pub fn new(
        doc: &Document,
        parent: (u64, u64),
        tuning: &Tuning,
        palette: Palette,
    ) -> anyhow::Result<Self> {
        let prim = doc.create_prim(Layer::Local, Some(parent))?;
        let batch = draw::mesh(&mesh::annulus(INNER, OUTER, SEGMENTS))
            .into_iter()
            .fold(doc.local(), |batch, property| batch.set(prim, property));
        batch
            .set(prim, Property::Transform(draw::hidden()))
            .flush()?;
        graphs::set_ring(doc, prim)?;

        Ok(Self {
            prim,
            progress: Cell::new(None),
            recoil: Cell::new(None),
            speed: tuning.cast_recoil,
            palette,
        })
    }

    pub fn hide(&self, doc: &Document) -> anyhow::Result<()> {
        self.progress.set(None);
        self.recoil.set(None);
        doc.local()
            .set(self.prim, Property::Transform(draw::hidden()))
            .flush()?;
        Ok(())
    }

    /// `at` is the mote being cast on, in the surface's own coordinates.
    pub fn apply(&self, doc: &Document, at: Vec3, state: State) -> anyhow::Result<()> {
        match state {
            // The ring stays where it stood and unwinds from there, so what
            // draws is the fill running backwards.
            State::Aborted => {
                let Some(left) = self.progress.get().filter(|left| *left > 0.0) else {
                    return self.hide(doc);
                };
                self.recoil.set(Some(left));
                return Ok(());
            }
            State::Committed => return self.hide(doc),
            State::Filling(_) => self.recoil.set(None),
        }

        doc.local()
            .set(
                self.prim,
                Property::Transform(Transform {
                    translation: Vec3::new(at.x, at.y, at.z + LIFT),
                    rotation:    Quat::IDENTITY,
                    scale:       Vec3::ONE,
                }),
            )
            .flush()?;
        self.draw(doc, state.progress())
    }

    /// Unwinds an abandoned cast, and stands the ring down once it is empty.
    /// Runs whether or not a cast is live, because an abort is exactly the
    /// moment the cast stops being one.
    pub fn step(&self, doc: &Document, delta: f32) -> anyhow::Result<()> {
        let Some(left) = self.recoil.get() else {
            return Ok(());
        };
        let Some(left) = cast::recoil(left, delta, self.speed) else {
            return self.hide(doc);
        };
        self.recoil.set(Some(left));
        self.draw(doc, left)
    }

    fn draw(&self, doc: &Document, progress: f32) -> anyhow::Result<()> {
        if self.progress.get() == Some(progress) {
            return Ok(());
        }
        self.progress.set(Some(progress));
        // The accent is spent here rather than on hover: a cast is rare, and
        // it is the one thing that must not be missed.
        set_overrides(
            doc,
            self.prim,
            Layer::Local,
            &[
                (graphs::RING_TINT, GraphValue::Color(self.palette.accent)),
                (graphs::RING_PROGRESS, GraphValue::Float(progress)),
            ],
        )?;
        Ok(())
    }
}
