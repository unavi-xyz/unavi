//! Transcribes a [`PlacardView`] into prims.

use std::cell::{
    Cell,
    RefCell,
};

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
        graphs,
        pop,
        shadow::Shadow,
    },
    wired::scene::{
        document::{
            Document,
            Layer,
        },
        properties::{
            Property,
            Text,
            TextAlign,
            TextAnchor,
        },
    },
};

const TEXT_LIFT: f32 = 0.0015;

pub struct PlacardPrims {
    root:     (u64, u64),
    panel:    (u64, u64),
    lines:    Vec<(u64, u64)>,
    /// Whether anything is currently drawn, so [`PlacardPrims::hide`] writes at
    /// most once per time the placard actually goes away.
    shown:    Cell<bool>,
    /// How far the pop has run, 0 gone and 1 settled.
    progress: Cell<f32>,
    /// The view being shown, kept while it shrinks away after the placard
    /// itself has gone.
    view:     RefCell<Option<PlacardView>>,
    /// Last view and progress drawn; an identical pair costs no write at all.
    last:     Shadow<(PlacardView, f32)>,
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
        graphs::set_pane(doc, panel)?;

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
            progress: Cell::new(0.0),
            view: RefCell::new(None),
            last: Shadow::new(),
        })
    }

    /// Steps the pop toward `placard` shown or gone and draws where it has
    /// got to.
    pub fn update(
        &self,
        doc: &Document,
        placard: Option<&PlacardView>,
        palette: &Palette,
        delta: f32,
    ) -> anyhow::Result<()> {
        if let Some(placard) = placard {
            let mut view = self.view.borrow_mut();
            if view.as_ref() != Some(placard) {
                *view = Some(placard.clone());
            }
        }
        let progress = pop::step(self.progress.get(), placard.is_some(), delta);
        self.progress.set(progress);

        let view = self.view.borrow().clone();
        match view {
            Some(view) if progress > 0.0 => self.apply(doc, view, progress, palette),
            _ => {
                self.view.take();
                self.hide(doc)
            }
        }
    }

    fn hide(&self, doc: &Document) -> anyhow::Result<()> {
        self.last.clear();
        if self.shown.replace(false) {
            doc.local()
                .set(self.root, Property::Transform(draw::hidden()))
                .flush()?;
        }
        Ok(())
    }

    fn apply(
        &self,
        doc: &Document,
        view: PlacardView,
        progress: f32,
        palette: &Palette,
    ) -> anyhow::Result<()> {
        let Some((view, progress)) = self.last.diff((view, progress)) else {
            return Ok(());
        };
        self.shown.set(true);

        // The root sits at the pane's top centre and the pane hangs down from
        // it, so scaling the root alone would grow the placard from that edge.
        // Moving it by the pane's centre times what the scale took off keeps
        // the centre where it was.
        let scale = pop::scale(progress);
        let centre = Vec3::new(0.0, -view.size.y / 2.0, 0.0);
        let mut batch = doc.local().set(
            self.root,
            Property::Transform(Transform {
                translation: view.position + centre * (1.0 - scale),
                rotation:    Quat::IDENTITY,
                scale:       Vec3::splat(scale),
            }),
        );

        // The pane is opaque glass and cannot fade, so it stays whole while
        // shown; scale alone brings it in.
        batch = batch.set(
            self.panel,
            Property::Transform(Transform {
                translation: Vec3::ZERO,
                rotation:    Quat::IDENTITY,
                scale:       Vec3::new(view.size.x, view.size.y, 1.0),
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
                            progress,
                        )),
                        // No outline: dark type on a light pane needs none.
                        outline:       None,
                        outline_width: None,
                        // Ink does not glow.
                        emissive:      Some(0.0),
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

/// Body type is the same ink as the title: on a frosted pane a softened grey
/// sinks into whatever the glass shows through, and the title already
/// outweighs it by size.
const fn tint(palette: &Palette, emphasis: Emphasis) -> wired_guest::math::Color {
    match emphasis {
        Emphasis::Title | Emphasis::Body => palette.ink,
        Emphasis::Warning => palette.warning,
    }
}
