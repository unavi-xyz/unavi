//! Draws the breadcrumb: parent motes stacked out of the plane toward the
//! viewer.

use std::cell::Cell;

use unavi_vui::{
    mesh,
    trail::{
        TrailView,
        MAX_BEADS,
    },
    view::Style,
};
use wired_prelude::prelude::*;

use crate::wired::scene::types::{
    AlphaMode,
    Document,
    Material,
    Prim,
    Text,
    TextAlign,
    TextAnchor,
    Xform,
};

const SPHERE_RINGS: usize = 8;
const SPHERE_SEGMENTS: usize = 12;
/// Clearance between the deepest bead and the count riding behind it.
const COUNT_GAP: f32 = 0.03;
const COUNT_SIZE: f32 = 0.012;

struct Bead {
    prim:  Prim,
    style: Cell<Option<Style>>,
}

pub struct Trail {
    beads:   Vec<Bead>,
    /// How many levels the stack does not show, drawn only when there are any.
    hidden:  Prim,
    written: Cell<usize>,
}

impl Trail {
    pub fn new(doc: &Document, parent: &Prim) -> anyhow::Result<Self> {
        let unit = mesh::sphere(1.0, SPHERE_RINGS, SPHERE_SEGMENTS);
        let mut beads = Vec::with_capacity(MAX_BEADS);

        for _ in 0..MAX_BEADS {
            let prim = doc.create_prim()?;
            prim.set_mesh_stream("POSITION", Some(&unit.positions))?;
            prim.set_mesh_stream("NORMAL", Some(&unit.normals))?;
            prim.set_mesh_indices_u32(Some(&unit.indices))?;
            prim.set_xform(Some(hidden()))?;
            parent.add_child(&prim)?;
            beads.push(Bead {
                prim,
                style: Cell::new(None),
            });
        }

        let hidden_count = doc.create_prim()?;
        hidden_count.set_xform(Some(hidden()))?;
        parent.add_child(&hidden_count)?;

        Ok(Self {
            beads,
            hidden: hidden_count,
            written: Cell::new(usize::MAX),
        })
    }

    pub fn apply(&self, view: &TrailView) -> anyhow::Result<()> {
        for (bead, drawn) in self.beads.iter().zip(&view.beads) {
            bead.prim.set_xform(Some(Xform {
                translation: drawn.position,
                rotation:    Quat::IDENTITY,
                scale:       Vec3::splat(drawn.radius),
            }))?;
            if bead.style.get() != Some(drawn.style) {
                bead.style.set(Some(drawn.style));
                bead.prim.set_material(Some(material(drawn.style)))?;
            }
        }
        for bead in self.beads.iter().skip(view.beads.len()) {
            bead.prim.set_xform(Some(hidden()))?;
        }

        self.apply_count(view)
    }

    /// Unbounded depth, bounded stack: the rest is a number.
    fn apply_count(&self, view: &TrailView) -> anyhow::Result<()> {
        let deepest = view
            .beads
            .last()
            .map_or(0.0, |bead| bead.position.z + COUNT_GAP);
        self.hidden.set_xform(Some(if view.hidden == 0 {
            hidden()
        } else {
            Xform {
                translation: Vec3::new(0.0, 0.0, deepest),
                rotation:    Quat::IDENTITY,
                scale:       Vec3::ONE,
            }
        }))?;

        if view.hidden == 0 || self.written.get() == view.hidden {
            self.written.set(view.hidden);
            return Ok(());
        }
        self.written.set(view.hidden);
        self.hidden.set_text(Some(&Text {
            value:         format!("+{}", view.hidden),
            size:          Some(COUNT_SIZE),
            align:         Some(TextAlign::Center),
            anchor:        Some(TextAnchor::Middle),
            wrap:          None,
            line_height:   None,
            color:         None,
            outline:       None,
            outline_width: None,
            emissive:      Some(0.2),
            billboard:     None,
        }))?;
        Ok(())
    }
}

const fn material(style: Style) -> Material {
    Material {
        alpha_cutoff: None,
        alpha_mode:   Some(AlphaMode::Blend),
        base_color:   Some(Color {
            r: style.color.r,
            g: style.color.g,
            b: style.color.b,
            a: style.alpha,
        }),
        double_sided: Some(true),
        emissive:     Some(Color {
            r: style.color.r * style.emissive,
            g: style.color.g * style.emissive,
            b: style.color.b * style.emissive,
            a: 1.0,
        }),
        metallic:     None,
        roughness:    None,
    }
}

const fn hidden() -> Xform {
    Xform {
        translation: Vec3::ZERO,
        rotation:    Quat::IDENTITY,
        scale:       Vec3::ZERO,
    }
}
