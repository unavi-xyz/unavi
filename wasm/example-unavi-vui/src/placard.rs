//! The placard's prim pool. Like `bodies`, this decides nothing: every offset
//! and every opacity was computed by `unavi-vui`.

use std::cell::Cell;

use unavi_vui::{
    mesh,
    palette::Palette,
    placard::{
        Emphasis,
        MAX_LINES,
        PlacardView,
    },
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

/// How far in front of the panel the text sits, so the card reads as a
/// surface with writing on it rather than writing with a card behind it.
const TEXT_LIFT: f32 = 0.0015;
/// Opacity the backdrop reaches when the placard is fully in. Dark and mostly
/// opaque: a card you can read the room through is a card you cannot read.
const PANEL_ALPHA: f32 = 0.88;

pub struct Placard {
    root:    Prim,
    panel:   Prim,
    lines:   Vec<Prim>,
    /// Last opacity drawn. Text re-uploads nothing when it changes, but the
    /// attribute write still costs a sync, so an unchanged fade is skipped.
    opacity: Cell<Option<f32>>,
}

impl Placard {
    pub fn new(doc: &Document, parent: &Prim) -> anyhow::Result<Self> {
        let root = doc.create_prim()?;
        root.set_xform(Some(hidden()))?;
        parent.add_child(&root)?;

        let panel = doc.create_prim()?;
        let quad = mesh::panel();
        panel.set_mesh_stream("POSITION", Some(&quad.positions))?;
        panel.set_mesh_stream("NORMAL", Some(&quad.normals))?;
        panel.set_mesh_indices_u32(Some(&quad.indices))?;
        panel.set_xform(Some(hidden()))?;
        root.add_child(&panel)?;

        let mut lines = Vec::with_capacity(MAX_LINES);
        for _ in 0..MAX_LINES {
            let prim = doc.create_prim()?;
            prim.set_xform(Some(hidden()))?;
            root.add_child(&prim)?;
            lines.push(prim);
        }

        Ok(Self {
            root,
            panel,
            lines,
            opacity: Cell::new(None),
        })
    }

    pub fn hide(&self) -> anyhow::Result<()> {
        if self.opacity.get() != Some(0.0) {
            self.opacity.set(Some(0.0));
            self.root.set_xform(Some(hidden()))?;
        }
        Ok(())
    }

    pub fn apply(&self, view: &PlacardView, palette: &Palette) -> anyhow::Result<()> {
        self.root.set_xform(Some(Xform {
            translation: view.position,
            rotation:    Quat::IDENTITY,
            scale:       Vec3::ONE,
        }))?;

        self.panel.set_xform(Some(Xform {
            translation: Vec3::ZERO,
            rotation:    Quat::IDENTITY,
            scale:       Vec3::new(view.size.x, view.size.y, 1.0),
        }))?;
        self.panel.set_material(Some(Material {
            alpha_cutoff: None,
            alpha_mode:   Some(AlphaMode::Blend),
            base_color:   Some(with_alpha(palette.surface, PANEL_ALPHA * view.opacity)),
            double_sided: Some(true),
            emissive:     None,
            metallic:     None,
            roughness:    Some(1.0),
        }))?;

        for (prim, line) in self.lines.iter().zip(&view.lines) {
            prim.set_xform(Some(Xform {
                translation: Vec3::new(line.offset.x, line.offset.y, TEXT_LIFT),
                rotation:    Quat::IDENTITY,
                scale:       Vec3::ONE,
            }))?;
            prim.set_text(Some(&Text {
                value:         line.text.to_string(),
                size:          Some(line.size),
                align:         Some(TextAlign::Left),
                anchor:        Some(TextAnchor::Baseline),
                wrap:          None,
                line_height:   None,
                color:         Some(with_alpha(tint(palette, line.emphasis), view.opacity)),
                // No outline: the card behind it already supplies the
                // contrast, and an outline on top of a backdrop only thickens
                // the strokes.
                outline:       None,
                outline_width: None,
                emissive:      Some(match line.emphasis {
                    Emphasis::Title => 0.4,
                    Emphasis::Body => 0.12,
                    Emphasis::Dim => 0.0,
                }),
                billboard:     None,
            }))?;
        }
        for prim in self.lines.iter().skip(view.lines.len()) {
            prim.set_xform(Some(hidden()))?;
        }

        self.opacity.set(Some(view.opacity));
        Ok(())
    }
}

const fn tint(palette: &Palette, emphasis: Emphasis) -> Color {
    match emphasis {
        Emphasis::Title => palette.accent,
        Emphasis::Body => palette.base,
        Emphasis::Dim => palette.dim,
    }
}

const fn with_alpha(color: Color, a: f32) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a,
    }
}

const fn hidden() -> Xform {
    Xform {
        translation: Vec3::ZERO,
        rotation:    Quat::IDENTITY,
        scale:       Vec3::splat(0.0),
    }
}
