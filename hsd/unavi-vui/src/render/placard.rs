//! Transcribes a [`PlacardView`] into prims.

use std::cell::Cell;

use wired_guest::math::{
    Quat,
    Transform,
    Vec3,
};

use crate::{
    mesh,
    palette::Palette,
    placard::{
        Emphasis,
        MAX_LINES,
        PlacardView,
    },
    render::{
        draw,
        shadow::Shadow,
    },
    wired::scene::{
        document::{
            Document,
            Layer,
        },
        properties::{
            AlphaMode,
            Material,
            Property,
            Text,
            TextAlign,
            TextAnchor,
        },
    },
};

const TEXT_LIFT: f32 = 0.0015;
/// Backdrop opacity when the placard is fully in; dark enough that the text
/// stays readable.
const PANEL_ALPHA: f32 = 0.88;

pub struct PlacardPrims {
    root:  (u64, u64),
    panel: (u64, u64),
    lines: Vec<(u64, u64)>,
    /// Whether anything is currently drawn, so [`PlacardPrims::hide`] writes at
    /// most once per time the placard actually goes away.
    shown: Cell<bool>,
    /// Last view drawn; an identical one costs no write at all.
    last:  Shadow<PlacardView>,
}

impl PlacardPrims {
    pub fn new(doc: &Document, parent: (u64, u64)) -> anyhow::Result<Self> {
        let root = doc.create_prim(Layer::Local, Some(parent))?;
        doc.local()
            .set(root, Property::Transform(draw::hidden()))
            .flush()?;

        let panel = doc.create_prim(Layer::Local, Some(root))?;
        let mut batch = draw::mesh(&mesh::panel())
            .into_iter()
            .fold(doc.local(), |batch, property| batch.set(panel, property));
        batch = batch.set(panel, Property::Transform(draw::hidden()));

        let mut lines = Vec::with_capacity(MAX_LINES);
        for _ in 0..MAX_LINES {
            let prim = doc.create_prim(Layer::Local, Some(root))?;
            batch = batch.set(prim, Property::Transform(draw::hidden()));
            lines.push(prim);
        }
        batch.flush()?;

        Ok(Self {
            root,
            panel,
            lines,
            shown: Cell::new(false),
            last: Shadow::new(),
        })
    }

    pub fn hide(&self, doc: &Document) -> anyhow::Result<()> {
        self.last.clear();
        if self.shown.replace(false) {
            doc.local()
                .set(self.root, Property::Transform(draw::hidden()))
                .flush()?;
        }
        Ok(())
    }

    pub fn apply(
        &self,
        doc: &Document,
        view: &PlacardView,
        palette: &Palette,
    ) -> anyhow::Result<()> {
        let Some(view) = self.last.diff(view.clone()) else {
            return Ok(());
        };
        self.shown.set(true);

        let mut batch = doc.local().set(
            self.root,
            Property::Transform(Transform {
                translation: view.position,
                rotation:    Quat::IDENTITY,
                scale:       Vec3::ONE,
            }),
        );

        batch = batch
            .set(
                self.panel,
                Property::Transform(Transform {
                    translation: Vec3::ZERO,
                    rotation:    Quat::IDENTITY,
                    scale:       Vec3::new(view.size.x, view.size.y, 1.0),
                }),
            )
            .set(
                self.panel,
                Property::Material(Material {
                    alpha_cutoff: None,
                    alpha_mode:   Some(AlphaMode::Blend),
                    base_color:   Some(draw::with_alpha(
                        palette.surface,
                        PANEL_ALPHA * view.opacity,
                    )),
                    double_sided: Some(true),
                    emissive:     None,
                    metallic:     None,
                    roughness:    Some(1.0),
                }),
            );

        for (prim, line) in self.lines.iter().zip(&view.lines) {
            batch = batch
                .set(
                    *prim,
                    Property::Transform(Transform {
                        translation: Vec3::new(line.offset.x, line.offset.y, TEXT_LIFT),
                        rotation:    Quat::IDENTITY,
                        scale:       Vec3::ONE,
                    }),
                )
                .set(
                    *prim,
                    Property::Text(Text {
                        value:         line.text.to_string(),
                        size:          Some(line.size),
                        align:         Some(TextAlign::Left),
                        anchor:        Some(TextAnchor::Baseline),
                        wrap:          None,
                        line_height:   None,
                        color:         Some(draw::with_alpha(
                            tint(palette, line.emphasis),
                            view.opacity,
                        )),
                        // No outline: the card behind it already supplies the
                        // contrast.
                        outline:       None,
                        outline_width: None,
                        emissive:      Some(match line.emphasis {
                            Emphasis::Title => 0.4,
                            Emphasis::Body => 0.12,
                            Emphasis::Dim => 0.0,
                        }),
                        billboard:     None,
                    }),
                );
        }
        for prim in self.lines.iter().skip(view.lines.len()) {
            batch = batch.set(*prim, Property::Transform(draw::hidden()));
        }
        batch.flush()?;
        Ok(())
    }
}

const fn tint(palette: &Palette, emphasis: Emphasis) -> wired_guest::math::Color {
    match emphasis {
        Emphasis::Title => palette.accent,
        Emphasis::Body => palette.base,
        Emphasis::Dim => palette.dim,
    }
}
