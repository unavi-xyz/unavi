//! What pointers do to a slate: presses on its rows, drags on its numbers,
//! hover, and the grip that picks it up.

use wired_guest::math::{
    Vec2,
    Vec3,
};

use super::{
    Scrub,
    Slate,
    SlateEvent,
};
use crate::{
    slate::{
        self,
        Kind,
        Press,
        Row,
        WIDTH,
    },
    wired::{
        input::types::{
            Action,
            Button,
            Pointer,
        },
        scene::document::Document,
    },
};

/// Metres off the pane's centre, across it, past which a sample is a ray
/// grazing the pane rather than a drag along it.
const SCRUB_REACH: f32 = WIDTH * 2.0;

impl Slate {
    /// `point`, in the script's frame, as a point on the pane.
    fn local(&self, point: Vec3) -> Option<Vec2> {
        let local = self.face_pose?.inverse().transform_point(point);
        Some(Vec2::new(local.x, local.y))
    }

    /// Where `origin` along `direction`, in the script's frame, meets the
    /// pane's plane, across the pane.
    fn across(&self, origin: Vec3, direction: Vec3) -> Option<f32> {
        let inverse = self.face_pose?.inverse();
        let origin = inverse.transform_point(origin);
        let direction = inverse.rotation * direction;
        if direction.z.abs() < 1.0e-4 {
            return None;
        }
        let travel = -origin.z / direction.z;
        let x = direction.x.mul_add(travel, origin.x);
        (travel > 0.0 && x.abs() < SCRUB_REACH).then_some(x)
    }

    pub(super) fn read(&mut self, doc: &Document, pointers: &[Pointer]) {
        for event in self.input.drain(64) {
            match event.action {
                // The host's grab carries the body; the slate only stops
                // floating while it does.
                Action::Pressed(Button::Grip) => self.float.grab(event.pointer),
                Action::Released(Button::Grip) => self.float.release(event.pointer),
                Action::Pressed(Button::Trigger) => {
                    if let Some(local) = event.hit.and_then(|hit| self.local(hit.position)) {
                        self.press(local, event.pointer);
                    }
                }
                Action::Released(Button::Trigger)
                    if self
                        .scrub
                        .as_ref()
                        .is_some_and(|scrub| scrub.pointer == event.pointer) =>
                {
                    self.end_scrub();
                }
                _ => {}
            }
        }

        self.step_scrub(pointers);

        let target = (doc.id(), self.body);
        let hovered = pointers
            .iter()
            .filter(|pointer| pointer.active)
            .filter_map(|pointer| pointer.hit.as_ref())
            .filter(|hit| {
                hit.target
                    .as_ref()
                    .is_some_and(|hit| (hit.document, hit.prim) == target)
            })
            .find_map(|hit| slate::row_at(self.local(hit.position)?, self.rows.len()))
            .filter(|index| !matches!(self.rows[*index].press(), Press::Nothing | Press::Pick));
        let held = self.scrub.as_ref().map(|scrub| scrub.row);
        let hovered = held.or(hovered);
        if hovered != self.hovered {
            self.hovered = hovered;
            self.stale = true;
        }
    }

    /// A trigger went down on the pane at `local`.
    fn press(&mut self, local: Vec2, pointer: crate::wired::input::types::PointerKind) {
        let Some(index) = slate::row_at(local, self.rows.len()) else {
            return;
        };
        let id = self.rows[index].id;
        match self.rows[index].press() {
            Press::Nothing => {}
            Press::Act => self.push(SlateEvent::Pressed(id)),
            Press::Toggle(on) => self.push(SlateEvent::Toggled(id, on)),
            Press::Pick => {
                let tab = match &self.rows[index].kind {
                    Kind::Tabs { labels, .. } => slate::tab_at(local.x, labels.len()),
                    _ => None,
                };
                if let Some(tab) = tab {
                    self.push(SlateEvent::Picked(id, tab as u32));
                }
            }
            Press::Scrub(_) => {
                // One drag at a time: a second pointer taking a row ends the
                // first one's, reporting it kept.
                self.end_scrub();
                self.scrub = Some(Scrub {
                    row: index,
                    id,
                    pointer,
                    last_x: Some(local.x),
                });
            }
        }
    }

    /// Reports how far a drag moved since the last sample.
    fn step_scrub(&mut self, pointers: &[Pointer]) {
        let Some(scrub) = &self.scrub else {
            return;
        };
        let x = pointers
            .iter()
            .find(|pointer| pointer.kind == scrub.pointer && pointer.active)
            .and_then(|pointer| self.across(pointer.ray.origin, pointer.ray.direction));
        let Some(Press::Scrub(per_metre)) = self.rows.get(scrub.row).map(Row::press) else {
            return;
        };
        let (id, last) = (scrub.id, scrub.last_x);
        if let Some(scrub) = &mut self.scrub {
            scrub.last_x = x;
        }
        if let (Some(x), Some(last)) = (x, last)
            && (x - last).abs() > f32::EPSILON
        {
            self.push(SlateEvent::Scrubbed(id, (x - last) * per_metre));
        }
    }
}
