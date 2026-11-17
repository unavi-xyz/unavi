//! Slates a script stands where it likes: a pane of rows, pressed and
//! scrubbed with any pointer, and picked up with a grip like anything else.
//!
//! A slate is a weightless body: one collider covers the pane, the host's
//! grab carries it, and once let go it floats where it stops (see
//! [`float`]). The row is read off where a pointer meets the pane, so a
//! slate costs one input listener however many rows it holds.

use wired_guest::math::{
    Color,
    Transform,
    Vec2,
    Vec3,
};

use self::float::Float;
use crate::{
    component::slate::{
        MAX_ROWS,
        MAX_TABS,
        Press,
        Row,
        {
            self,
        },
    },
    material::graphs,
    part::{
        draw,
        mesh,
        tether::prims::TetherPrims,
    },
    wired::{
        input::{
            device::pointers,
            targeted::listen,
            types::{
                InputSubscription,
                PointerKind,
            },
        },
        scene::{
            document::{
                Document,
                Layer,
            },
            properties::Property,
        },
    },
};

pub mod body;
pub mod float;
mod input;
mod rows;

/// A slate [`Slates`] is drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlateId(usize);

/// What a slate's rows did, in the terms a consumer thinks in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlateEvent {
    /// An action row was pressed.
    Pressed(u32),
    /// A toggle row was pressed, asking for this state.
    Toggled(u32, bool),
    /// A number row, or a part of a vector row, was dragged sideways, by
    /// this much of its value.
    Scrubbed(u32, u32, f32),
    /// A drag on a number row ended.
    Released(u32),
    /// A tab was pressed: the tabs row, and the tab's index.
    Picked(u32, u32),
}

/// Metres the type and the hover bar stand proud of the pane.
const LIFT: f32 = 0.0015;
/// Events a slate holds for a consumer that has not asked.
const EVENT_CAPACITY: usize = 64;

/// A drag on a number row.
struct Scrub {
    row:     usize,
    /// The row's id, which `released` reports even once the row is gone.
    id:      u32,
    /// Which number of a vector row; 0 for a number row.
    part:    u32,
    pointer: PointerKind,
    /// Where the pointer last met the pane, across it. `None` after a sample
    /// that missed it, so the next one sets a new baseline rather than
    /// reporting the whole gap as a drag.
    last_x:  Option<f32>,
}

struct Slate {
    /// The weightless body, at the pane's middle, carrying its collider.
    body:          (u64, u64),
    /// Beneath the body, at the pane's top centre: everything drawn hangs
    /// from it, so it pops and bobs without moving the collider.
    face:          (u64, u64),
    pane:          (u64, u64),
    hover:         (u64, u64),
    labels:        Vec<(u64, u64)>,
    values:        Vec<(u64, u64)>,
    /// Per row: the first two numbers of a vector row (the value prim draws
    /// the third), the dot, the mark and a section's rule.
    extras:        Vec<Extras>,
    /// One text prim per tab of the tabs row, and the bar under the selected.
    tab_labels:    Vec<(u64, u64)>,
    tab_mark:      (u64, u64),
    tether:        TetherPrims,
    /// Where the tether runs to, in the document frame.
    tether_to:     Option<Vec3>,
    input:         InputSubscription,
    rows:          Vec<Row>,
    /// Where the script last put the pane's top centre, until it is moved
    /// there.
    put:           Option<Vec3>,
    accent:        Option<Color>,
    shown:         bool,
    /// Whether the body is in the simulation, which it is only while shown.
    simulated:     bool,
    /// The row count the collider was last sized for.
    collider_rows: Option<usize>,
    /// How far the pop has run, 0 gone and 1 settled.
    progress:      f32,
    /// Seconds of bobbing, for its phase.
    bob:           f32,
    hovered:       Option<usize>,
    scrub:         Option<Scrub>,
    float:         Float,
    events:        Vec<SlateEvent>,
    /// The body's pose as last read.
    pose:          Option<Transform>,
    /// The face's pose under it as last drawn.
    face_pose:     Option<Transform>,
    /// Whether the rows or their look changed since they were last drawn.
    stale:         bool,
}

/// The prims a row draws besides its label and value.
#[derive(Clone, Copy)]
struct Extras {
    parts: [(u64, u64); 2],
    dot:   (u64, u64),
    mark:  (u64, u64),
    rule:  (u64, u64),
}

/// Every slate a script holds, as a free list like the panels.
#[derive(Default)]
pub struct Slates {
    slots: Vec<Option<Slate>>,
    free:  Vec<usize>,
    /// Where the viewer's eye was last drawn from, which free slates turn to.
    eye:   Option<Vec3>,
}

impl Slates {
    pub fn add(&mut self, doc: &Document) -> anyhow::Result<SlateId> {
        let body = doc.create_prim(Layer::Local, None)?;
        let face = doc.create_prim(Layer::Local, Some(body))?;
        let pane = doc.create_prim(Layer::Local, Some(face))?;
        let hover = doc.create_prim(Layer::Local, Some(face))?;

        let mut batch = doc.local().set(body, Property::Transform(draw::hidden()));
        batch = draw::mesh(&mesh::panel())
            .into_iter()
            .fold(batch, |batch, property| batch.set(pane, property));
        batch = draw::mesh(&mesh::bar())
            .into_iter()
            .fold(batch, |batch, property| batch.set(hover, property))
            .set(hover, Property::Transform(draw::hidden()));

        let mut labels = Vec::with_capacity(MAX_ROWS);
        let mut values = Vec::with_capacity(MAX_ROWS);
        let mut extras = Vec::with_capacity(MAX_ROWS);
        let dot_mesh = draw::mesh(&mesh::sphere(1.0, 6, 10));
        let bar_mesh = draw::mesh(&mesh::bar());
        for _ in 0..MAX_ROWS {
            let text = || doc.create_prim(Layer::Local, Some(face));
            let row = [text()?, text()?];
            let row_extras = Extras {
                parts: [text()?, text()?],
                dot:   text()?,
                mark:  text()?,
                rule:  text()?,
            };
            for prim in row.iter().chain(&row_extras.parts) {
                batch = batch.set(*prim, Property::Transform(draw::hidden()));
            }
            for (prim, shape) in [
                (row_extras.dot, &dot_mesh),
                (row_extras.mark, &dot_mesh),
                (row_extras.rule, &bar_mesh),
            ] {
                batch = shape
                    .iter()
                    .cloned()
                    .fold(batch, |batch, property| batch.set(prim, property))
                    .set(prim, Property::Transform(draw::hidden()));
            }
            labels.push(row[0]);
            values.push(row[1]);
            extras.push(row_extras);
        }
        let tab_mark = doc.create_prim(Layer::Local, Some(face))?;
        batch = draw::mesh(&mesh::bar())
            .into_iter()
            .fold(batch, |batch, property| batch.set(tab_mark, property))
            .set(tab_mark, Property::Transform(draw::hidden()));
        let mut tab_labels = Vec::with_capacity(MAX_TABS);
        for _ in 0..MAX_TABS {
            let prim = doc.create_prim(Layer::Local, Some(face))?;
            batch = batch.set(prim, Property::Transform(draw::hidden()));
            tab_labels.push(prim);
        }
        batch.flush()?;
        graphs::slab::set_slab(doc, pane)?;

        let slate = Slate {
            body,
            face,
            pane,
            hover,
            labels,
            values,
            extras,
            tab_labels,
            tab_mark,
            tether: TetherPrims::new(doc)?,
            tether_to: None,
            input: listen(doc, body)?,
            rows: Vec::new(),
            put: None,
            accent: None,
            shown: false,
            simulated: false,
            collider_rows: None,
            progress: 0.0,
            bob: 0.0,
            hovered: None,
            scrub: None,
            float: Float::default(),
            events: Vec::new(),
            pose: None,
            face_pose: None,
            stale: true,
        };
        if let Some(index) = self.free.pop() {
            self.slots[index] = Some(slate);
            return Ok(SlateId(index));
        }
        self.slots.push(Some(slate));
        Ok(SlateId(self.slots.len() - 1))
    }

    pub fn remove(&mut self, doc: &Document, id: SlateId) -> anyhow::Result<()> {
        let Some(slate) = self.slots.get_mut(id.0).and_then(Option::take) else {
            return Ok(());
        };
        self.free.push(id.0);
        doc.local().remove(slate.body).flush()?;
        slate.tether.remove(doc)?;
        Ok(())
    }

    fn get_mut(&mut self, id: SlateId) -> Option<&mut Slate> {
        self.slots.get_mut(id.0)?.as_mut()
    }

    pub fn set_rows(&mut self, id: SlateId, rows: Vec<Row>) {
        if let Some(slate) = self.get_mut(id)
            && slate.rows != rows
        {
            // A drag follows its row by id wherever the row moves to, and ends
            // when the row is gone or no longer a number.
            let moved = slate.scrub.as_ref().map(|scrub| {
                rows.iter()
                    .position(|row| row.id == scrub.id && matches!(row.press(), Press::Scrub(_)))
            });
            match moved {
                Some(Some(index)) => {
                    if let Some(scrub) = &mut slate.scrub {
                        scrub.row = index;
                    }
                }
                Some(None) => slate.end_scrub(),
                None => {}
            }
            slate.rows = rows;
            slate.stale = true;
        }
    }

    pub fn set_position(&mut self, id: SlateId, at: Vec3) {
        if let Some(slate) = self.get_mut(id) {
            slate.put = Some(at);
        }
    }

    /// Where the pane's top centre is: where the script put it, else where it
    /// floats.
    #[must_use]
    pub fn position(&self, id: SlateId) -> Option<Vec3> {
        let slate = self.slots.get(id.0)?.as_ref()?;
        slate
            .put
            .or_else(|| slate.face_pose.map(|pose| pose.translation))
    }

    pub fn set_tether(&mut self, id: SlateId, to: Option<Vec3>) {
        if let Some(slate) = self.get_mut(id) {
            slate.tether_to = to;
        }
    }

    pub fn set_accent(&mut self, id: SlateId, accent: Option<Color>) {
        if let Some(slate) = self.get_mut(id)
            && slate.accent != accent
        {
            slate.accent = accent;
            slate.stale = true;
        }
    }

    pub fn set_shown(&mut self, id: SlateId, shown: bool) {
        if let Some(slate) = self.get_mut(id) {
            slate.shown = shown;
            if !shown {
                slate.end_scrub();
            }
        }
    }

    #[must_use]
    pub fn size(&self, id: SlateId) -> Vec2 {
        self.slots
            .get(id.0)
            .and_then(Option::as_ref)
            .map_or(Vec2::ZERO, |slate| slate::size(slate.rows.len()))
    }

    pub fn drain(&mut self, id: SlateId) -> Vec<SlateEvent> {
        self.get_mut(id)
            .map(|slate| std::mem::take(&mut slate.events))
            .unwrap_or_default()
    }

    /// Reads presses, drags, grabs and hover, and keeps free slates afloat.
    /// Call from `fixed_update`.
    pub fn fixed_update(&mut self, doc: &Document) {
        let pointers = pointers().unwrap_or_default();
        for slate in self.slots.iter_mut().flatten() {
            slate.read(doc, &pointers);
            if let Some(eye) = self.eye
                && let Err(err) = slate.steer(doc, eye)
            {
                eprintln!("vui: slate float: {err:?}");
            }
        }
    }

    /// Moves slates the script put, eases each in or out, and draws its rows
    /// and tether. Call from `update`.
    pub fn update(
        &mut self,
        doc: &Document,
        eye: &Transform,
        palette: &crate::tokens::palette::Palette,
        tuning: &crate::tokens::tuning::Tuning,
        delta: f32,
    ) -> anyhow::Result<()> {
        self.eye = Some(eye.translation);
        for slate in self.slots.iter_mut().flatten() {
            slate.draw(doc, eye, palette, delta)?;
            slate.draw_tether(doc, palette, tuning)?;
        }
        Ok(())
    }
}

impl Slate {
    /// Ends a drag, telling the consumer, so a value it kept open is kept.
    fn end_scrub(&mut self) {
        if let Some(scrub) = self.scrub.take() {
            self.push(SlateEvent::Released(scrub.id));
        }
    }

    fn push(&mut self, event: SlateEvent) {
        self.events.push(event);
        let overrun = self.events.len().saturating_sub(EVENT_CAPACITY);
        self.events.drain(..overrun);
    }
}
