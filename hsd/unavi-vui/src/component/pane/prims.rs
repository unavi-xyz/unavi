//! Transcribes a pane's runs into prims: one slab, then one prim per text run
//! and one per rectangle, written only when their content changes.

use wired_guest::math::{
    Color,
    Quat,
    Transform,
    Vec2,
    Vec3,
};

use crate::{
    component::pane::{
        block::Tone,
        draw::{
            Fill,
            RectRun,
            Runs,
            TextRun,
        },
        pool::Pool,
    },
    material::graphs,
    part::{
        draw,
        mesh,
        pop,
        shadow::Shadow,
    },
    tokens::{
        palette::Palette,
        tuning::Tuning,
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

/// Most text runs and rectangles a pane draws. Past these, the rest of the
/// pane is left undrawn.
const MAX_TEXT: usize = 192;
const MAX_RECTS: usize = 96;
/// Steps type fades in over, so the fade rewrites each run a few times
/// rather than every frame.
const FADE_STEPS: f32 = 8.0;

/// How strongly each fill shows.
const TINT: f32 = 0.10;
const RULE: f32 = 0.15;
const TRACK: f32 = 0.10;
const LIT: f32 = 0.85;
const ACCENT: f32 = 0.28;
const THUMB: f32 = 0.40;
const UNDERLINE: f32 = 0.6;
const FOCUS: f32 = 0.55;

/// A run and how far its type has faded in.
type Faded<T> = (T, u8);

pub struct PanePrims {
    root:      (u64, u64),
    slab:      (u64, u64),
    texts:     Vec<(u64, u64)>,
    rects:     Vec<(u64, u64)>,
    text_pool: Pool<Faded<TextRun>>,
    rect_pool: Pool<Faded<RectRun>>,
    /// Where the root and slab were last put.
    pose:      Shadow<(Vec2, Vec3, f32)>,
    /// How far the pop has run, 0 gone and 1 settled.
    progress:  f32,
    /// What is being shown, kept while it shrinks away after the pane has
    /// gone.
    kept:      Option<(Runs, Vec2)>,
    shown:     bool,
}

impl PanePrims {
    /// Creates the slab under `parent`. Nothing shows until it is updated.
    pub fn new(doc: &Document, parent: (u64, u64)) -> anyhow::Result<Self> {
        let root = doc.create_prim(Layer::Local, Some(parent))?;
        let slab = doc.create_prim(Layer::Local, Some(root))?;
        let batch = draw::mesh(&mesh::panel())
            .into_iter()
            .fold(doc.local(), |batch, property| batch.set(slab, property))
            .set(slab, Property::Transform(draw::hidden()))
            .set(root, Property::Transform(draw::hidden()));
        batch.flush()?;
        graphs::slab::set_slab(doc, slab)?;
        Ok(Self {
            root,
            slab,
            texts: Vec::new(),
            rects: Vec::new(),
            text_pool: Pool::default(),
            rect_pool: Pool::default(),
            pose: Shadow::new(),
            progress: 0.0,
            kept: None,
            shown: false,
        })
    }

    /// How far the pop has run, 0 gone and 1 settled.
    #[must_use]
    pub const fn progress(&self) -> f32 {
        self.progress
    }

    /// Steps the pop toward `pane` shown or gone and draws where it has got
    /// to. `origin` is the pane's top centre in the parent's frame, and
    /// `size` its width and height in ems.
    pub fn update(
        &mut self,
        doc: &Document,
        pane: Option<(&Runs, Vec2)>,
        origin: Vec3,
        palette: &Palette,
        tuning: &Tuning,
        delta: f32,
    ) -> anyhow::Result<()> {
        if let Some((runs, size)) = pane
            && self
                .kept
                .as_ref()
                .is_none_or(|(kept, was)| kept != runs || *was != size)
        {
            self.kept = Some((runs.clone(), size));
        }
        self.progress = pop::step(self.progress, pane.is_some(), delta);

        let Some((runs, size)) = self.kept.clone().filter(|_| self.progress > 0.0) else {
            self.kept = None;
            return self.hide(doc);
        };
        self.shown = true;
        self.draw(doc, &runs, size, origin, palette, tuning)
    }

    fn hide(&mut self, doc: &Document) -> anyhow::Result<()> {
        self.pose.clear();
        if !self.shown {
            return Ok(());
        }
        self.shown = false;
        let mut batch = doc
            .local()
            .set(self.root, Property::Transform(draw::hidden()));
        for slot in self.text_pool.clear() {
            batch = batch.set(self.texts[slot], Property::Transform(draw::hidden()));
        }
        for slot in self.rect_pool.clear() {
            batch = batch.set(self.rects[slot], Property::Transform(draw::hidden()));
        }
        batch.flush()?;
        Ok(())
    }

    fn draw(
        &mut self,
        doc: &Document,
        runs: &Runs,
        size: Vec2,
        origin: Vec3,
        palette: &Palette,
        tuning: &Tuning,
    ) -> anyhow::Result<()> {
        let em = tuning.pane_em;
        let (width, height) = (size.x * em, size.y * em);
        let scale = pop::scale(self.progress);

        // The slab goes first and alone, so a tick that stops after it shows
        // blank glass rather than type with nothing under it. The root sits
        // at the pane's top centre, so scaling about the centre moves it by
        // what the scale took off.
        if self.pose.diff((size, origin, scale)).is_some() {
            let centre = Vec3::new(0.0, -height / 2.0, 0.0);
            doc.local()
                .set(
                    self.root,
                    Property::Transform(Transform {
                        translation: origin + centre * (1.0 - scale),
                        rotation:    Quat::IDENTITY,
                        scale:       Vec3::splat(scale),
                    }),
                )
                .set(
                    self.slab,
                    Property::Transform(Transform {
                        translation: Vec3::ZERO,
                        rotation:    Quat::IDENTITY,
                        scale:       Vec3::new(width, height, 1.0),
                    }),
                )
                .flush()?;
        }

        let fade = (self.progress * FADE_STEPS).round() as u8;
        let text = runs
            .text
            .iter()
            .take(MAX_TEXT)
            .map(|run| (run.clone(), fade))
            .collect::<Vec<_>>();
        let rects = runs
            .rects
            .iter()
            .take(MAX_RECTS)
            .map(|run| (run.clone(), fade))
            .collect::<Vec<_>>();
        let text_plan = self.text_pool.plan(text);
        let rect_plan = self.rect_pool.plan(rects);
        if text_plan.writes.is_empty()
            && text_plan.hidden.is_empty()
            && rect_plan.writes.is_empty()
            && rect_plan.hidden.is_empty()
        {
            return Ok(());
        }

        self.grow(doc)?;
        let mut batch = doc.local();
        let lift = tuning.pane_lift;
        for (slot, (run, fade)) in &rect_plan.writes {
            batch = self.write_rect(batch, *slot, run, *fade, (width, em, lift / 2.0), palette);
        }
        for (slot, (run, fade)) in &text_plan.writes {
            batch = self.write_text(batch, *slot, run, *fade, (width, em, lift), palette);
        }
        for slot in text_plan.hidden {
            batch = batch.set(self.texts[slot], Property::Transform(draw::hidden()));
        }
        for slot in rect_plan.hidden {
            batch = batch.set(self.rects[slot], Property::Transform(draw::hidden()));
        }
        batch.flush()?;
        Ok(())
    }

    /// Makes a prim for every slot the pools have touched.
    fn grow(&mut self, doc: &Document) -> anyhow::Result<()> {
        let mesh = draw::mesh(&mesh::panel());
        let mut batch = doc.local();
        while self.texts.len() < self.text_pool.slots() {
            let prim = doc.create_prim(Layer::Local, Some(self.root))?;
            batch = batch.set(prim, Property::Transform(draw::hidden()));
            self.texts.push(prim);
        }
        while self.rects.len() < self.rect_pool.slots() {
            let prim = doc.create_prim(Layer::Local, Some(self.root))?;
            batch = mesh
                .iter()
                .cloned()
                .fold(batch, |batch, property| batch.set(prim, property))
                .set(prim, Property::Transform(draw::hidden()));
            self.rects.push(prim);
        }
        batch.flush()?;
        Ok(())
    }

    fn write_rect<'a>(
        &self,
        batch: crate::Batch<'a>,
        slot: usize,
        run: &RectRun,
        fade: u8,
        (width, em, lift): (f32, f32, f32),
        palette: &Palette,
    ) -> crate::Batch<'a> {
        let prim = self.rects[slot];
        let (color, strength) = paint(run.fill, palette);
        let rect = run.rect;
        batch
            .set(
                prim,
                Property::Transform(Transform {
                    translation: Vec3::new(
                        (rect.x + rect.w / 2.0).mul_add(em, -width / 2.0),
                        -rect.y * em,
                        lift,
                    ),
                    rotation:    Quat::IDENTITY,
                    scale:       Vec3::new(rect.w * em, rect.h * em, 1.0),
                }),
            )
            .set(
                prim,
                Property::Material(draw::ink(color, strength * f32::from(fade) / FADE_STEPS)),
            )
    }

    fn write_text<'a>(
        &self,
        batch: crate::Batch<'a>,
        slot: usize,
        run: &TextRun,
        fade: u8,
        (width, em, lift): (f32, f32, f32),
        palette: &Palette,
    ) -> crate::Batch<'a> {
        let prim = self.texts[slot];
        batch
            .set(
                prim,
                Property::Transform(Transform {
                    translation: Vec3::new(
                        run.at.x.mul_add(em, -width / 2.0),
                        -run.at.y * em,
                        lift,
                    ),
                    rotation:    Quat::IDENTITY,
                    scale:       Vec3::ONE,
                }),
            )
            .set(
                prim,
                Property::Text(Text {
                    value:         run.text.to_string(),
                    size:          Some(run.scale * em),
                    align:         Some(TextAlign::Left),
                    anchor:        Some(TextAnchor::Baseline),
                    wrap:          None,
                    line_height:   None,
                    color:         Some(draw::with_alpha(
                        ink(palette, run.tone),
                        f32::from(fade) / FADE_STEPS,
                    )),
                    outline:       None,
                    outline_width: None,
                    emissive:      Some(0.0),
                    billboard:     None,
                }),
            )
    }
}

const fn ink(palette: &Palette, tone: Tone) -> Color {
    match tone {
        Tone::Body => palette.ink,
        Tone::Muted => palette.muted_ink,
        Tone::Warning => palette.warning,
    }
}

const fn paint(fill: Fill, palette: &Palette) -> (Color, f32) {
    match fill {
        Fill::Tint => (palette.muted_ink, TINT),
        Fill::Rule => (palette.ink, RULE),
        Fill::Track => (palette.ink, TRACK),
        Fill::Lit => (palette.ink, LIT),
        Fill::Accent => (palette.accent, ACCENT),
        Fill::Thumb => (palette.ink, THUMB),
        Fill::Underline => (palette.ink, UNDERLINE),
        Fill::Focus => (palette.accent, FOCUS),
    }
}
