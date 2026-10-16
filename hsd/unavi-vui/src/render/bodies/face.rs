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

/// How far in front of the shell's face the dots stand, as a fraction of the
/// body's radius. Enough to clear the curve of the glass across the whole
/// square, which a flat quad at the pole itself would not.
const FACE_LIFT: f32 = 1.04;

/// Steps a face's direction is snapped to along each axis of the unit
/// vector, about a degree apart near the pole.
const FACE_STEPS: f32 = 64.0;

/// A slot's glyph face: a quad that waits hidden until a glyph is first
/// wanted, and what it last showed.
pub(super) struct Face {
    prim:   (u64, u64),
    /// Whether the quad has its mesh and graph yet. Paid for on the first
    /// glyph rather than at every slot's creation, since most slots never
    /// wear one.
    ready:  Cell<bool>,
    pose:   Shadow<Option<Transform>>,
    flip:   RefCell<Flip>,
    inputs: Shadow<Inputs>,
    /// The mote this face last belonged to, held rather than remembered by
    /// address so a new mote can never be mistaken for the old one.
    owner:  RefCell<Option<Mote>>,
    /// Counts owner changes; it is what the flip tells owners apart by.
    epoch:  Cell<u64>,
    /// Whether the slot was drawn last frame. One that was not has been out of
    /// sight, and whatever glyph it comes back with simply appears.
    seen:   Cell<bool>,
}

impl Face {
    pub(super) const fn new(prim: (u64, u64)) -> Self {
        Self {
            prim,
            ready: Cell::new(false),
            pose: Shadow::new(),
            flip: RefCell::new(Flip::new()),
            inputs: Shadow::new(),
            owner: RefCell::new(None),
            epoch: Cell::new(0),
            seen: Cell::new(false),
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
    pub(super) fn of_slot(view: &SlotView, fill: f32, eye: Vec3) -> Self {
        Self {
            pose:  face_pose(view, fill, eye),
            color: view.style.color,
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
            batch = draw::mesh(&mesh::face())
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
/// The dots fill a square whose side is the icon fraction of the shell's
/// diameter, so a glyph takes the room an icon would. The face stands just in
/// front of the glass on the line from the shell's centre to `eye`, turned to
/// face it: a flat quad at the pole would slide toward the far edge when the
/// mote is seen from the side. It yaws and pitches but never rolls, so the
/// glyph stays upright.
///
/// The direction is snapped to steps of about a degree before it is used, so
/// a head held still, whose tracking still trembles, does not rewrite every
/// face every frame.
fn face_pose(view: &SlotView, fill: f32, eye: Vec3) -> Transform {
    let side = 2.0 * view.radius * fill;
    let toward = (eye - view.position).normalize_or(Vec3::Z);
    let toward = (toward * FACE_STEPS).round().normalize_or(Vec3::Z);
    let yaw = toward.x.atan2(toward.z);
    // Clamped: a unit vector can come back a hair over 1 long, and `asin`
    // of that is NaN.
    let pitch = (-toward.y).clamp(-1.0, 1.0).asin();
    let pitched = Quat::new((pitch * 0.5).sin(), 0.0, 0.0, (pitch * 0.5).cos());
    Transform {
        translation: toward * (view.radius * FACE_LIFT),
        rotation:    Quat::from_rotation_y(yaw) * pitched,
        scale:       Vec3::splat(side),
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
    fn the_face_takes_the_room_an_icon_would_and_clears_the_glass() {
        let pose = face_pose(&view(0.04), 0.5, Vec3::new(0.0, 0.0, 1.0));
        assert!((pose.scale.x - 0.04).abs() < 1.0e-6, "half the diameter");
        assert!(pose.translation.z > 0.04, "in front of the shell's pole");
        assert!(pose.translation.x.abs() < 1.0e-6);
        assert!(pose.translation.y.abs() < 1.0e-6);
    }

    /// The face is a flat quad, so the shell cannot reach it however far
    /// from the pole a dot is: the plane clears the pole, and the clip circle
    /// stays inside the silhouette it is seen against.
    #[test]
    fn the_largest_face_clears_the_glass_and_stays_inside_the_silhouette() {
        let fill = crate::tuning::Tuning::DEFAULT.icon_fill;
        const { assert!(FACE_LIFT >= 1.0, "the plane stands off the pole") };
        for size in [9.0_f32, 13.0] {
            let reach = (size - 1.0).mul_add(0.5, crate::glyph::CLIP_MARGIN);
            let radius = 2.0 * fill * reach / size;
            assert!(
                radius < 0.85,
                "{size}: the clip circle reaches {radius} of the shell"
            );
            assert!(
                radius.hypot(FACE_LIFT) >= 1.0,
                "{size}: a dot at the rim of the clip is inside the glass"
            );
        }
    }

    #[test]
    fn seen_from_the_side_the_face_stands_toward_the_eye_and_faces_it() {
        let mut slot = view(0.04);
        slot.position = Vec3::new(0.2, -0.1, 0.0);
        let eye = Vec3::new(-0.3, 0.5, 0.8);
        let toward = (eye - slot.position).normalize();

        let pose = face_pose(&slot, 0.5, eye);
        let lift = pose.translation.length();
        let expected = 0.04 * FACE_LIFT;
        assert!((lift - expected).abs() < 1.0e-6, "clear of the glass");
        // Within the snap: a step is 1/64 along each axis.
        let snap = 1.0 / FACE_STEPS;
        assert!(
            (pose.translation / lift - toward).length() < snap,
            "on the line from the shell's centre to the eye"
        );
        assert!(
            (pose.rotation * Vec3::Z - pose.translation / lift).length() < 1.0e-5,
            "turned to face the way it stands"
        );
        assert!(
            (pose.rotation * Vec3::X).y.abs() < 1.0e-5,
            "upright, with no roll"
        );
    }
}
