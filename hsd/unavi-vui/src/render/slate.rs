//! Slates a script stands where it likes: a pane of rows, pressed and
//! scrubbed with any pointer.
//!
//! One collider covers the pane and the row is read off where the pointer
//! meets it, so a slate costs one input listener however many rows it holds.

use wired_guest::math::{
    Color,
    Quat,
    Transform,
    Vec2,
    Vec3,
};

use crate::{
    mesh,
    palette::Palette,
    render::{
        draw,
        graphs,
        pop,
    },
    slate::{
        self,
        HEADER_SIZE,
        Kind,
        MAX_ROWS,
        PADDING,
        Press,
        ROW_HEIGHT,
        Row,
        TEXT_SIZE,
        WIDTH,
    },
    wired::{
        input::{
            device::pointers,
            targeted::listen,
            types::{
                Action,
                Button,
                InputSubscription,
                PointerKind,
            },
        },
        scene::{
            document::{
                Document,
                Layer,
            },
            properties::{
                Collider,
                Property,
                Text,
                TextAlign,
                TextAnchor,
            },
        },
    },
};

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
    /// A number row was dragged sideways, by this much of its value.
    Scrubbed(u32, f32),
    /// A drag on a number row ended.
    Released(u32),
}

/// Metres of collider behind the pane.
const HITBOX_DEPTH: f32 = 0.01;
/// Metres the type and the hover bar stand proud of the pane.
const LIFT: f32 = 0.0015;
/// Events a slate holds for a consumer that has not asked.
const EVENT_CAPACITY: usize = 64;

/// A drag on a number row.
struct Scrub {
    row:     usize,
    pointer: PointerKind,
    /// Where the pointer last met the pane, across it.
    last_x:  f32,
}

struct Slate {
    root:     (u64, u64),
    pane:     (u64, u64),
    hover:    (u64, u64),
    hitbox:   (u64, u64),
    labels:   Vec<(u64, u64)>,
    values:   Vec<(u64, u64)>,
    input:    InputSubscription,
    rows:     Vec<Row>,
    at:       Vec3,
    accent:   Option<Color>,
    shown:    bool,
    /// How far the pop has run, 0 gone and 1 settled.
    progress: f32,
    hovered:  Option<usize>,
    scrub:    Option<Scrub>,
    events:   Vec<SlateEvent>,
    /// The root's pose last drawn, which pointer hits are read against.
    pose:     Option<Transform>,
    /// Whether the rows or their look changed since they were last drawn.
    stale:    bool,
}

/// Every slate a script holds, as a free list like the panels.
#[derive(Default)]
pub struct Slates {
    slots: Vec<Option<Slate>>,
    free:  Vec<usize>,
}

impl Slates {
    pub fn add(&mut self, doc: &Document) -> anyhow::Result<SlateId> {
        let root = doc.create_prim(Layer::Local, None)?;
        let pane = doc.create_prim(Layer::Local, Some(root))?;
        let hover = doc.create_prim(Layer::Local, Some(root))?;
        let hitbox = doc.create_prim(Layer::Local, Some(root))?;

        let mut batch = doc.local().set(root, Property::Transform(draw::hidden()));
        batch = draw::mesh(&mesh::panel())
            .into_iter()
            .fold(batch, |batch, property| batch.set(pane, property));
        batch = draw::mesh(&mesh::bar())
            .into_iter()
            .fold(batch, |batch, property| batch.set(hover, property))
            .set(hover, Property::Transform(draw::hidden()));

        let mut labels = Vec::with_capacity(MAX_ROWS);
        let mut values = Vec::with_capacity(MAX_ROWS);
        for _ in 0..MAX_ROWS {
            let label = doc.create_prim(Layer::Local, Some(root))?;
            let value = doc.create_prim(Layer::Local, Some(root))?;
            batch = batch
                .set(label, Property::Transform(draw::hidden()))
                .set(value, Property::Transform(draw::hidden()));
            labels.push(label);
            values.push(value);
        }
        batch.flush()?;
        graphs::set_pane(doc, pane)?;

        let slate = Slate {
            root,
            pane,
            hover,
            hitbox,
            labels,
            values,
            input: listen(doc, hitbox)?,
            rows: Vec::new(),
            at: Vec3::ZERO,
            accent: None,
            shown: false,
            progress: 0.0,
            hovered: None,
            scrub: None,
            events: Vec::new(),
            pose: None,
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
        doc.local().remove(slate.root).flush()?;
        Ok(())
    }

    fn get_mut(&mut self, id: SlateId) -> Option<&mut Slate> {
        self.slots.get_mut(id.0)?.as_mut()
    }

    pub fn set_rows(&mut self, id: SlateId, rows: Vec<Row>) {
        if let Some(slate) = self.get_mut(id)
            && slate.rows != rows
        {
            // A drag on a row that is gone has nothing left to change.
            if slate
                .scrub
                .as_ref()
                .is_some_and(|scrub| scrub.row >= rows.len())
            {
                slate.scrub = None;
            }
            slate.rows = rows;
            slate.stale = true;
        }
    }

    pub fn set_position(&mut self, id: SlateId, at: Vec3) {
        if let Some(slate) = self.get_mut(id) {
            slate.at = at;
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
                slate.scrub = None;
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

    /// Reads presses, drags and hover. Call from `fixed_update`.
    pub fn fixed_update(&mut self, doc: &Document) {
        let pointers = pointers().unwrap_or_default();
        for slate in self.slots.iter_mut().flatten() {
            slate.read(doc, &pointers);
        }
    }

    /// Eases each slate in or out, faces it to `eye`, and draws its rows.
    /// Call from `update`.
    pub fn update(
        &mut self,
        doc: &Document,
        eye: &Transform,
        palette: &Palette,
        delta: f32,
    ) -> anyhow::Result<()> {
        for slate in self.slots.iter_mut().flatten() {
            slate.draw(doc, eye, palette, delta)?;
        }
        Ok(())
    }
}

impl Slate {
    /// `point`, in the script's frame, as a point on the pane.
    fn local(&self, point: Vec3) -> Option<Vec2> {
        let local = self.pose?.inverse().transform_point(point);
        Some(Vec2::new(local.x, local.y))
    }

    /// Where `origin` along `direction`, in the script's frame, meets the
    /// pane's plane, across the pane.
    fn across(&self, origin: Vec3, direction: Vec3) -> Option<f32> {
        let inverse = self.pose?.inverse();
        let origin = inverse.transform_point(origin);
        let direction = inverse.rotation * direction;
        if direction.z.abs() < 1.0e-4 {
            return None;
        }
        let travel = -origin.z / direction.z;
        (travel > 0.0).then(|| direction.x.mul_add(travel, origin.x))
    }

    fn push(&mut self, event: SlateEvent) {
        self.events.push(event);
        let overrun = self.events.len().saturating_sub(EVENT_CAPACITY);
        self.events.drain(..overrun);
    }

    fn read(&mut self, doc: &Document, pointers: &[crate::wired::input::types::Pointer]) {
        for event in self.input.drain(64) {
            match event.action {
                Action::Pressed(Button::Trigger) => {
                    let Some(local) = event.hit.and_then(|hit| self.local(hit.position)) else {
                        continue;
                    };
                    let Some(index) = slate::row_at(local, self.rows.len()) else {
                        continue;
                    };
                    let id = self.rows[index].id;
                    match self.rows[index].press() {
                        Press::Nothing => {}
                        Press::Act => self.push(SlateEvent::Pressed(id)),
                        Press::Toggle(on) => self.push(SlateEvent::Toggled(id, on)),
                        Press::Scrub(_) => {
                            self.scrub = Some(Scrub {
                                row:     index,
                                pointer: event.pointer,
                                last_x:  local.x,
                            });
                        }
                    }
                }
                Action::Released(Button::Trigger) => {
                    if let Some(scrub) = self.scrub.take()
                        && let Some(row) = self.rows.get(scrub.row)
                    {
                        self.push(SlateEvent::Released(row.id));
                    }
                }
                _ => {}
            }
        }

        let drag = self.scrub.as_ref().and_then(|scrub| {
            let pointer = pointers
                .iter()
                .find(|pointer| pointer.kind == scrub.pointer && pointer.active)?;
            let x = self.across(pointer.ray.origin, pointer.ray.direction)?;
            let row = self.rows.get(scrub.row)?;
            let Press::Scrub(per_metre) = row.press() else {
                return None;
            };
            Some((row.id, (x - scrub.last_x) * per_metre, x))
        });
        if let Some((id, change, x)) = drag {
            if let Some(scrub) = &mut self.scrub {
                scrub.last_x = x;
            }
            if change != 0.0 {
                self.push(SlateEvent::Scrubbed(id, change));
            }
        }

        let hitbox = (doc.id(), self.hitbox);
        let hovered = pointers
            .iter()
            .filter(|pointer| pointer.active)
            .filter_map(|pointer| pointer.hit.as_ref())
            .filter(|hit| {
                hit.target
                    .as_ref()
                    .is_some_and(|target| (target.document, target.prim) == hitbox)
            })
            .find_map(|hit| slate::row_at(self.local(hit.position)?, self.rows.len()))
            .filter(|index| self.rows[*index].press() != Press::Nothing);
        let held = self.scrub.as_ref().map(|scrub| scrub.row);
        let hovered = held.or(hovered);
        if hovered != self.hovered {
            self.hovered = hovered;
            self.stale = true;
        }
    }

    fn draw(
        &mut self,
        doc: &Document,
        eye: &Transform,
        palette: &Palette,
        delta: f32,
    ) -> anyhow::Result<()> {
        let progress = pop::step(self.progress, self.shown, delta);
        let moved = (progress - self.progress).abs() > f32::EPSILON;
        self.progress = progress;
        if progress <= 0.0 {
            if self.pose.take().is_some() {
                doc.local()
                    .set(self.root, Property::Transform(draw::hidden()))
                    .flush()?;
            }
            return Ok(());
        }

        let toward = eye.translation - self.at;
        let pose = Transform {
            translation: self.at,
            rotation:    Quat::from_rotation_y(toward.x.atan2(toward.z)),
            scale:       Vec3::splat(pop::scale(progress)),
        };
        let mut batch = doc.local();
        if self.pose != Some(pose) {
            batch = batch.set(self.root, Property::Transform(pose));
            self.pose = Some(pose);
        }
        if !self.stale && !moved {
            return Ok(batch.flush()?);
        }
        self.stale = false;

        let size = slate::size(self.rows.len());
        batch = batch
            .set(
                self.pane,
                Property::Transform(Transform {
                    translation: Vec3::ZERO,
                    rotation:    Quat::IDENTITY,
                    scale:       Vec3::new(size.x, size.y, 1.0),
                }),
            )
            .set(
                self.hitbox,
                Property::Transform(Transform::from_translation(Vec3::new(
                    0.0,
                    -size.y / 2.0,
                    -HITBOX_DEPTH / 2.0,
                ))),
            )
            .set(
                self.hitbox,
                Property::Collider(Collider::Cuboid(Vec3::new(size.x, size.y, HITBOX_DEPTH))),
            );

        batch = match self.hovered {
            Some(index) => batch
                .set(
                    self.hover,
                    Property::Transform(Transform {
                        translation: Vec3::new(0.0, slate::row_centre(index), LIFT / 2.0),
                        rotation:    Quat::IDENTITY,
                        scale:       Vec3::new(WIDTH - PADDING, ROW_HEIGHT * 0.9, LIFT / 2.0),
                    }),
                )
                .set(
                    self.hover,
                    Property::Material(draw::ink(
                        self.accent.unwrap_or(palette.accent),
                        0.18 * progress,
                    )),
                ),
            None => batch.set(self.hover, Property::Transform(draw::hidden())),
        };

        batch = self.draw_rows(batch, palette, progress);
        Ok(batch.flush()?)
    }

    /// Writes each row's label and value into `batch`, and hides the rest.
    fn draw_rows<'a>(
        &self,
        mut batch: crate::Batch<'a>,
        palette: &Palette,
        progress: f32,
    ) -> crate::Batch<'a> {
        for (index, (label, value)) in self.labels.iter().zip(&self.values).enumerate() {
            let Some(row) = self.rows.get(index) else {
                batch = batch
                    .set(*label, Property::Transform(draw::hidden()))
                    .set(*value, Property::Transform(draw::hidden()));
                continue;
            };
            let y = slate::row_centre(index);
            let header = row.kind == Kind::Header;
            let ink = if header {
                palette.muted_ink
            } else {
                palette.ink
            };
            batch = batch
                .set(
                    *label,
                    Property::Transform(Transform::from_translation(Vec3::new(
                        -WIDTH / 2.0 + PADDING,
                        y,
                        LIFT,
                    ))),
                )
                .set(
                    *label,
                    Property::Text(text(
                        &row.label,
                        if header { HEADER_SIZE } else { TEXT_SIZE },
                        TextAlign::Left,
                        draw::with_alpha(ink, progress),
                    )),
                )
                .set(
                    *value,
                    Property::Transform(Transform::from_translation(Vec3::new(
                        WIDTH / 2.0 - PADDING,
                        y,
                        LIFT,
                    ))),
                )
                .set(
                    *value,
                    Property::Text(text(
                        &row.value(),
                        TEXT_SIZE,
                        TextAlign::Right,
                        draw::with_alpha(palette.muted_ink, progress),
                    )),
                );
        }
        batch
    }
}

fn text(value: &str, size: f32, align: TextAlign, color: Color) -> Text {
    Text {
        value:         value.to_owned(),
        size:          Some(size),
        align:         Some(align),
        anchor:        Some(TextAnchor::Middle),
        wrap:          None,
        line_height:   None,
        color:         Some(color),
        // Dark type on a light pane needs no outline, and ink does not glow.
        outline:       None,
        outline_width: None,
        emissive:      Some(0.0),
        billboard:     None,
    }
}
