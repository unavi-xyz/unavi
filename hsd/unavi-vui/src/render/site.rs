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
    render::{
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
/// The ring's radii as fractions of the body it rings, so it stands just
/// outside the shell at whatever size the body has grown to.
const INNER: f32 = 1.15;
const OUTER: f32 = 1.4;
/// Stands clear of the mote it rings without reaching the hit surface.
const LIFT: f32 = 0.006;

/// Where the ring stands for a body at `at` of `radius`: the mesh is drawn in
/// body units, so the body's radius is its scale.
fn ring_pose(at: Vec3, radius: f32) -> Transform {
    Transform {
        translation: Vec3::new(at.x, at.y, at.z + LIFT),
        rotation:    Quat::IDENTITY,
        scale:       Vec3::splat(radius),
    }
}

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

    /// `at` is the mote being cast on, in the surface's own coordinates, and
    /// `radius` is its body's.
    pub fn apply(&self, doc: &Document, at: Vec3, radius: f32, state: State) -> anyhow::Result<()> {
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
            .set(self.prim, Property::Transform(ring_pose(at, radius)))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The radii the ring is actually drawn at: its mesh through its pose.
    fn drawn_radii(radius: f32) -> (f32, f32) {
        let pose = ring_pose(Vec3::ZERO, radius);
        let mesh = mesh::annulus(INNER, OUTER, SEGMENTS);
        mesh.positions
            .as_chunks::<3>()
            .0
            .iter()
            .map(|point| point[0].hypot(point[1]) * pose.scale.x)
            .fold((f32::MAX, f32::MIN), |(low, high), r| {
                (low.min(r), high.max(r))
            })
    }

    #[test]
    fn the_ring_stands_just_outside_whatever_size_the_body_is() {
        for radius in [0.02, 0.032, 0.05] {
            let (inner, outer) = drawn_radii(radius);
            assert!((inner / radius - 1.15).abs() < 1.0e-4);
            assert!((outer / radius - 1.4).abs() < 1.0e-4);
            assert!(inner > radius, "clear of the shell");
        }
    }

    #[test]
    fn the_ring_stands_clear_of_the_surface_it_rings() {
        assert!(ring_pose(Vec3::new(0.1, 0.2, 0.0), 0.04).translation.z > 0.0);
    }
}
