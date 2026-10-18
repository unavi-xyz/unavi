//! Draws a mote's glyph on its face, and flips it when the glyph changes.

use std::{
    cell::{
        Cell,
        RefCell,
    },
    rc::Rc,
};

use wired_guest::math::{
    Color,
    Quat,
    Transform,
    Vec3,
};

use super::Bodies;
use crate::{
    flip::Flip,
    glyph::Glyph,
    mesh,
    palette::Palette,
    render::{
        draw,
        graphs::matrix::{
            self,
            Inputs,
        },
        shadow::Shadow,
    },
    tree::Mote,
    view::SlotView,
    wired::{
        scene::{
            document::{
                Document,
                Layer,
            },
            properties::Property,
        },
        shading::graph::set_overrides,
    },
};

/// The radius a glyph's cap is drawn at, as a fraction of the body's. A hair
/// outside the glass, so the dots read as its surface and never z-fight it,
/// and close enough that they are not a pane standing in front.
const FACE_LIFT: f32 = 1.005;

/// Steps a face's direction is snapped to along each axis of the unit
/// vector, about a degree apart near the pole.
const FACE_STEPS: f32 = 64.0;

/// The surface a glyph's dots are drawn on.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Shape {
    /// A cap of the unit sphere, wrapping the glass the mote wears.
    Dome { fill: f32, reach: f32 },
    /// A flat quad, for a face with no glass behind it.
    Quad,
}

/// A slot's glyph face: a surface that waits hidden until a glyph is first
/// wanted, and what it last showed.
pub(super) struct Face {
    prim:  (u64, u64),
    /// Whether the face has its mesh and graph yet. Paid for on the first
    /// glyph rather than at every slot's creation, since most slots never
    /// wear one.
    ready: Cell<bool>,
    pose:  Shadow<Option<Transform>>,
    flip:  RefCell<Flip>,
    /// The surface the dots sit on.
    shape: Shape,
    /// The mote this face last belonged to, held rather than remembered by
    /// address so a new mote can never be mistaken for the old one.
    owner: RefCell<Option<Mote>>,
    /// Counts owner changes; it is what the flip tells owners apart by.
    epoch: Cell<u64>,
    /// Whether the slot was drawn last frame. One that was not has been out of
    /// sight, and whatever glyph it comes back with simply appears.
    seen:  Cell<bool>,
    inputs: Shadow<Inputs>,
}

impl Face {
    pub(super) const fn new(prim: (u64, u64), shape: Shape) -> Self {
        Self {
            prim,
            ready: Cell::new(false),
            pose: Shadow::new(),
            flip: RefCell::new(Flip::new()),
            shape,
            owner: RefCell::new(None),
            epoch: Cell::new(0),
            seen: Cell::new(false),
            inputs: Shadow::new(),
        }
    }

    /// Notes that the slot was not drawn this frame.
    pub(super) fn unseen(&self) {
        self.seen.set(false);
    }

    /// Forgets what was written, for a frame whose edits did not land.
    pub(super) fn forget(&self) {
        self.ready.set(false);
        self.pose.clear();
        self.inputs.clear();
    }

    /// The token telling `mote` apart from the face's previous owners.
    fn owner_token(&self, mote: Option<&Mote>) -> u64 {
        let mut held = self.owner.borrow_mut();
        let same = match (held.as_ref(), mote) {
            (Some(held), Some(mote)) => held.is(mote),
            (None, None) => true,
            _ => false,
        };
        if !same {
            self.epoch.set(self.epoch.get() + 1);
            *held = mote.cloned();
        }
        self.epoch.get()
    }
}

/// The glyph a slot's mote asks to wear, and whose it is.
#[derive(Clone, Copy)]
pub(super) struct Wear<'a> {
    /// The mote that wants it. A flip only runs for the mote that was
    /// already in the slot.
    pub owner: Option<&'a Mote>,
    pub glyph: Option<&'a Rc<Glyph>>,
}

/// How a face is drawn this frame: where it stands and what lights it.
#[derive(Clone, Copy)]
pub(super) struct Look {
    /// Where the face stands while it wears a glyph.
    pub pose:  Transform,
    /// The lit dots' colour.
    pub color: Color,
    pub heat:  f32,
    pub bloom: f32,
}

impl Look {
    /// A slot's face, on the shell of the mote it belongs to, turned toward
    /// `eye`, which is in the space the slot's position is.
    pub(super) fn of_slot(view: &SlotView, eye: Vec3, palette: &Palette) -> Self {
        Self {
            pose:  face_pose(view, eye),
            // Colour is spent on the shell, so every glyph is the same
            // off-white and reads against whatever hue the mote wears.
            color: palette.glyph,
            heat:  view.heat,
            bloom: view.bloom,
        }
    }
}

impl Bodies {
    /// Brings `face` up to date with what it is asked to wear, a frame's
    /// `delta` later.
    ///
    /// Writes only what changed: the dots are shaded entirely from the
    /// overrides, so a settled face costs nothing per frame, and a flip
    /// writes while it runs.
    pub(super) fn apply_face(
        &self,
        doc: &Document,
        face: &Face,
        look: Look,
        wear: Wear<'_>,
        delta: f32,
    ) -> anyhow::Result<()> {
        let owner = face.owner_token(wear.owner);
        {
            let mut flip = face.flip.borrow_mut();
            if !face.seen.replace(true) {
                // Out of sight last frame: nothing was shown to flip from.
                *flip = Flip::new();
            }
            // Stepped before the new glyph is taken, so a flip is drawn at
            // its start for a frame rather than skipping its first one.
            flip.step(delta, self.tuning.flip_duration);
            flip.show(owner, wear.glyph);
        }
        let flip = face.flip.borrow();
        let shown = flip.shown();

        let pose = shown.map(|_| look.pose);
        let mut batch = doc.local();
        let carve = pose.is_some() && !face.ready.get();
        if carve {
            matrix::set_matrix(doc, face.prim)?;
            let shape = match face.shape {
                Shape::Dome { fill, reach } => mesh::dome(fill, reach),
                Shape::Quad => mesh::face(),
            };
            batch = draw::mesh(&shape)
                .into_iter()
                .fold(batch, |batch, property| batch.set(face.prim, property));
        }
        if let Some(pose) = face.pose.diff(pose) {
            batch = batch.set(
                face.prim,
                Property::Transform(pose.unwrap_or_else(draw::hidden)),
            );
        }
        // The shadows were committed as they were diffed, so a flush that
        // fails must take them back or the edit would never be retried.
        if let Err(err) = batch.flush() {
            face.forget();
            return Err(err.into());
        }
        if carve {
            face.ready.set(true);
        }

        let Some(shown) = shown else {
            return Ok(());
        };
        let inputs = Inputs {
            lit:      look.color,
            size:     shown.size(),
            flip:     flip.progress(),
            heat:     look.heat,
            bloom:    look.bloom,
            current:  *shown.lanes(),
            previous: *flip.leaving().unwrap_or(shown).lanes(),
        };
        if let Some(inputs) = face.inputs.diff(inputs)
            && let Err(err) = set_overrides(doc, face.prim, Layer::Local, &inputs.overrides())
        {
            face.forget();
            return Err(err.into());
        }
        Ok(())
    }
}

/// Where a slot's face stands and how big it is.
///
/// The cap is centred on the body, at [`FACE_LIFT`] of its radius, so the dots
/// sit on the glass rather than on a pane in front of it. It is turned toward
/// `eye`: it yaws and pitches but never rolls, so the glyph stays upright.
///
/// The direction is snapped to steps of about a degree before it is used, so
/// a head held still, whose tracking still trembles, does not rewrite every
/// face every frame.
fn face_pose(view: &SlotView, eye: Vec3) -> Transform {
    let toward = (eye - view.position).normalize_or(Vec3::Z);
    let toward = (toward * FACE_STEPS).round().normalize_or(Vec3::Z);
    let yaw = toward.x.atan2(toward.z);
    // Clamped: a unit vector can come back a hair over 1 long, and `asin`
    // of that is NaN.
    let pitch = (-toward.y).clamp(-1.0, 1.0).asin();
    let pitched = Quat::new((pitch * 0.5).sin(), 0.0, 0.0, (pitch * 0.5).cos());
    Transform {
        translation: Vec3::ZERO,
        rotation:    Quat::from_rotation_y(yaw) * pitched,
        scale:       Vec3::splat(view.radius * FACE_LIFT),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        attention::Attention,
        mote::{
            Pips,
            Role,
        },
        view::Style,
    };

    fn view(radius: f32) -> SlotView {
        SlotView {
            position: Vec3::ZERO,
            radius,
            style: Style {
                color:    crate::palette::shade(1.0),
                alpha:    1.0,
                emissive: 0.0,
            },
            role: Role::Action,
            attention: Attention::Idle,
            heat: 0.0,
            pips: Pips::NONE,
            seized: false,
            label_offset: Vec3::ZERO,
            label_size: 0.01,
            bloom: 1.0,
        }
    }

    #[test]
    fn the_face_wraps_the_glass_it_is_drawn_on() {
        let pose = face_pose(&view(0.04), Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(pose.translation, Vec3::ZERO, "centred on the body");
        assert!(
            (pose.scale.x - 0.04 * FACE_LIFT).abs() < 1.0e-6,
            "the cap's radius"
        );
    }

    /// The cap stands just outside the glass, so the dots read as its surface
    /// without z-fighting it, and never as a pane floating in front.
    #[test]
    fn the_cap_stands_just_off_the_glass() {
        const { assert!(FACE_LIFT >= 1.0, "outside the glass") };
        const { assert!(FACE_LIFT < 1.02, "close enough to read as its surface") };
    }

    #[test]
    fn seen_from_the_side_the_face_turns_toward_the_eye() {
        let mut slot = view(0.04);
        slot.position = Vec3::new(0.2, -0.1, 0.0);
        let eye = Vec3::new(-0.3, 0.5, 0.8);
        let toward = (eye - slot.position).normalize();

        let pose = face_pose(&slot, eye);
        // Within the snap: a step is 1/64 along each axis.
        let snap = 1.0 / FACE_STEPS;
        assert!(
            (pose.rotation * Vec3::Z - toward).length() < snap,
            "turned to face the eye"
        );
        assert!(
            (pose.rotation * Vec3::X).y.abs() < 1.0e-5,
            "upright, with no roll"
        );
    }
}
