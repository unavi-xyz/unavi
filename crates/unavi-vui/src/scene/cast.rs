//! Draws a cast site: a ring on the mote that is filling.

use std::cell::Cell;

use wired_prelude::prelude::*;

use crate::{
    circle::Cast,
    mesh,
    palette::Palette,
    scene::draw,
    wired::scene::types::{
        AlphaMode,
        Document,
        Material,
        Prim,
        Xform,
    },
};

const SEGMENTS: usize = 40;
const INNER: f32 = 0.026;
const OUTER: f32 = 0.034;
/// Stands clear of the mote it rings without reaching the hit surface.
const LIFT: f32 = 0.006;

/// A ring whose drawn size is the fill. The mesh is a full annulus and the
/// progress is carried by scale, so filling re-uploads nothing.
pub struct Site {
    prim:    Prim,
    lit:     Cell<Option<bool>>,
    palette: Palette,
}

impl Site {
    pub fn new(doc: &Document, parent: &Prim, palette: Palette) -> anyhow::Result<Self> {
        let prim = doc.create_prim()?;
        draw::mesh(&prim, &mesh::annulus(INNER, OUTER, SEGMENTS))?;
        prim.set_xform(Some(draw::hidden()))?;
        parent.add_child(&prim)?;

        Ok(Self {
            prim,
            lit: Cell::new(None),
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
        // Growing from nothing to the full ring is the duration made visible;
        // there is no timer to read.
        let grown = 0.35_f32.mul_add(cast.progress(), 0.65);
        self.prim.set_xform(Some(Xform {
            translation: Vec3::new(at.x, at.y, at.z + LIFT),
            rotation:    Quat::IDENTITY,
            scale:       Vec3::splat(grown),
        }))?;

        // The accent is spent here rather than on hover: a cast is rare, and
        // it is the one thing that must not be missed.
        let lit = cast.progress() > 0.0;
        if self.lit.get() != Some(lit) {
            self.lit.set(Some(lit));
            self.prim.set_material(Some(material(&self.palette, lit)))?;
        }
        Ok(())
    }
}

const fn material(palette: &Palette, lit: bool) -> Material {
    let color = if lit { palette.accent } else { palette.base };
    Material {
        alpha_cutoff: None,
        alpha_mode:   Some(AlphaMode::Blend),
        base_color:   Some(draw::with_alpha(color, 0.9)),
        double_sided: Some(true),
        emissive:     Some(draw::scaled(color, 1.4)),
        metallic:     None,
        roughness:    None,
    }
}
