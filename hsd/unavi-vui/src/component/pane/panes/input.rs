//! What pointers and keys do to a pane: presses, drags, the wheel, hover and
//! the keyboard.

use wired_guest::math::{
    Vec2,
    Vec3,
};

use super::{
    Drag,
    Mount,
    Mounted,
    Panes,
};
use crate::{
    component::pane::{
        COLUMN,
        Event,
        control::Press,
    },
    tokens::tuning::Tuning,
    wired::{
        input::{
            device::pointers,
            types::{
                Action,
                Button,
                Key,
                Pointer,
                PointerKind,
            },
        },
        scene::document::Document,
    },
};

/// Rows a notch of the wheel scrolls.
const WHEEL_ROWS: f32 = 3.0;
/// Fraction a slider must move before another report.
const SLIDE_STEP: f32 = 1.0e-4;

impl Panes {
    /// Reads presses, drags, grabs, hover and keys, and keeps floating panes
    /// afloat. Call from `fixed_update`.
    pub fn fixed_update(&mut self, doc: &Document, tuning: &Tuning) {
        self.steer(doc);
        let pointers = pointers().unwrap_or_default();
        let mut pressed = None;
        for (index, slot) in self.slots.iter_mut().enumerate() {
            let Some(mounted) = slot else {
                continue;
            };
            if mounted.read(doc, &pointers, mounted.em(tuning)) {
                pressed = Some(index);
            }
        }
        if let Some(index) = pressed
            && self.focus != Some(index)
        {
            self.release_focus();
            self.focus = Some(index);
        }
        self.read_keys();
    }

    fn release_focus(&mut self) {
        if let Some(Some(mounted)) = self.focus.and_then(|index| self.slots.get_mut(index))
            && mounted.pane.set_focus(None)
        {
            mounted.runs = None;
        }
        self.focus = None;
    }

    fn read_keys(&mut self) {
        let Some(keys) = &self.keys else {
            return;
        };
        for event in keys.drain(64) {
            match event.action {
                Action::KeyPressed(Key::ShiftLeft | Key::ShiftRight) => self.shift = true,
                Action::KeyReleased(Key::ShiftLeft | Key::ShiftRight) => self.shift = false,
                Action::KeyPressed(Key::Escape) => self.release_focus(),
                Action::KeyPressed(key @ (Key::Tab | Key::Enter)) => {
                    let Some(Some(mounted)) =
                        self.focus.and_then(|index| self.slots.get_mut(index))
                    else {
                        continue;
                    };
                    if key == Key::Tab {
                        mounted.pane.focus_step(self.shift);
                        mounted.runs = None;
                    } else if let Some(event) = mounted.pane.activate() {
                        mounted.push(event);
                    }
                }
                _ => {}
            }
        }
    }
}

impl Mounted {
    /// A point in the script's frame, as ems from the pane's top left.
    fn local(&self, point: Vec3, em: f32) -> Option<Vec2> {
        let local = self.pose?.inverse().transform_point(point);
        Some(self.ems(local, em))
    }

    fn ems(&self, local: Vec3, em: f32) -> Vec2 {
        let size = self.pane.size();
        Vec2::new(
            local.x / em + size.x / 2.0,
            (size.y * em / 2.0 + self.bob_offset - local.y) / em,
        )
    }

    /// Where `origin` along `direction`, in the script's frame, meets the
    /// pane's plane.
    fn across(&self, origin: Vec3, direction: Vec3, em: f32) -> Option<Vec2> {
        let inverse = self.pose?.inverse();
        let origin = inverse.transform_point(origin);
        let direction = inverse.rotation * direction;
        if direction.z.abs() < 1.0e-4 {
            return None;
        }
        let travel = -origin.z / direction.z;
        (travel > 0.0).then(|| self.ems(origin + direction * travel, em))
    }

    /// Returns whether a trigger went down on the pane.
    pub(super) fn read(&mut self, doc: &Document, pointers: &[Pointer], em: f32) -> bool {
        let mut pressed = false;
        for event in self.input.drain(64) {
            match event.action {
                Action::Pressed(Button::Grip) => {
                    if matches!(self.mount, Some(Mount::Hung { .. })) {
                        self.pulled = Some(event.pointer);
                    } else {
                        self.float.grab(event.pointer);
                    }
                }
                Action::Released(Button::Grip) => self.float.release(event.pointer),
                Action::Pressed(Button::Trigger) => {
                    if let Some(local) = event.hit.and_then(|hit| self.local(hit.position, em)) {
                        pressed |= self.press(local, event.pointer);
                    }
                }
                Action::Released(Button::Trigger)
                    if self
                        .drag
                        .is_some_and(|(pointer, _)| pointer == event.pointer) =>
                {
                    self.end_drag();
                }
                Action::Scroll(by) => {
                    let local = event.hit.and_then(|hit| self.local(hit.position, em));
                    if let Some(hit) = local.and_then(|local| self.pane.hit(local)) {
                        let rows = -(by.y * WHEEL_ROWS).round() as i32;
                        if self.pane.scroll_by(hit.block, rows) {
                            self.runs = None;
                        }
                    }
                }
                _ => {}
            }
        }

        self.step_drag(pointers, em);

        let target = (doc.id(), self.body);
        let over = pointers
            .iter()
            .filter(|pointer| pointer.active)
            .filter_map(|pointer| pointer.hit.as_ref())
            .filter(|hit| {
                hit.target
                    .as_ref()
                    .is_some_and(|hit| (hit.document, hit.prim) == target)
            })
            .find_map(|hit| {
                let local = self.local(hit.position, em)?;
                self.pane.hit(local)?.id
            });
        let held = self.drag.and_then(|(_, drag)| match drag {
            Drag::Scrub { id, .. } | Drag::Slide { id, .. } => Some(id),
            Drag::Rail { .. } => None,
        });
        if self.pane.set_hovered(held.or(over)) {
            self.runs = None;
        }
        pressed
    }

    /// A trigger went down on the pane at `local`. Returns whether it hit
    /// anything.
    fn press(&mut self, local: Vec2, pointer: PointerKind) -> bool {
        let Some(hit) = self.pane.hit(local) else {
            return false;
        };
        self.runs = None;
        self.pane.set_focus(hit.id);
        let drag = match (hit.press, hit.id) {
            (Press::Scrub { per_column }, Some(id)) => Some(Drag::Scrub {
                id,
                per_column,
                last: Some(local.x),
            }),
            (Press::Slide(value), Some(id)) => {
                self.push(Event::Slid(id, value));
                Some(Drag::Slide {
                    id,
                    left: hit.area.x,
                    wide: hit.area.w,
                    last: value,
                })
            }
            (Press::Rail(fraction), _) => {
                self.pane.scroll_to(hit.block, fraction);
                Some(Drag::Rail {
                    block:  hit.block,
                    top:    hit.area.y,
                    height: hit.area.h,
                })
            }
            (press, Some(id)) => {
                if let Some(event) = press.event(id) {
                    self.push(event);
                }
                None
            }
            (_, None) => None,
        };
        if let Some(drag) = drag {
            // One drag at a time: a second pointer taking a control ends the
            // first one's, reporting it kept.
            self.end_drag();
            self.drag = Some((pointer, drag));
            if let Some(id) = hit.id {
                self.pane.set_pressed(Some(id));
            }
        }
        true
    }

    /// Reports how far a drag moved since the last sample.
    fn step_drag(&mut self, pointers: &[Pointer], em: f32) {
        let Some((pointer, drag)) = self.drag else {
            return;
        };
        let at = pointers
            .iter()
            .find(|candidate| candidate.kind == pointer && candidate.active)
            .and_then(|pointer| self.across(pointer.ray.origin, pointer.ray.direction, em));
        let next = match (drag, at) {
            (
                Drag::Scrub {
                    id,
                    per_column,
                    last,
                },
                at,
            ) => {
                if let (Some(at), Some(last)) = (at, last)
                    && (at.x - last).abs() > f32::EPSILON
                {
                    self.push(Event::Scrubbed(id, (at.x - last) / COLUMN * per_column));
                }
                Drag::Scrub {
                    id,
                    per_column,
                    last: at.map(|at| at.x),
                }
            }
            (
                Drag::Slide {
                    id,
                    left,
                    wide,
                    last,
                },
                Some(at),
            ) => {
                let value = ((at.x - left) / wide).clamp(0.0, 1.0);
                if (value - last).abs() > SLIDE_STEP {
                    self.push(Event::Slid(id, value));
                }
                Drag::Slide {
                    id,
                    left,
                    wide,
                    last: value,
                }
            }
            (Drag::Rail { block, top, height }, Some(at)) => {
                if self.pane.scroll_to(block, (at.y - top) / height) {
                    self.runs = None;
                }
                drag
            }
            (drag, None) => drag,
        };
        self.drag = Some((pointer, next));
    }
}
