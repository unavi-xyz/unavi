//! The Build tool: moves, turns, copies and removes the elements of a space,
//! keeping every change through an undoable edit.
//!
//! | Act | VR | Desktop |
//! | --- | --- | --- |
//! | move | trigger on an element, then drag | left mouse, drag |
//! | push, pull while moving | squeeze the grip, stick up, down | wheel |
//! | turn 15° while moving | squeeze the grip, flick the stick left, right | R |
//! | move off the grid | hold the other trigger while moving | hold Shift while moving |
//! | copy and move the copy | — | right mouse, drag |
//! | copy in place | hold the other trigger, flick the stick up | Ctrl+D |
//! | delete what is aimed at | hold the other trigger, flick the stick down | Delete |
//! | undo, redo | hold the other trigger, flick the stick left, right | Ctrl+Z, Ctrl+Shift+Z |
//!
//! The stick is the tool's only while the grip or the other trigger is held,
//! so it still walks and turns the user the rest of the time.

use std::cell::{
    Cell,
    RefCell,
};

use wired_guest::math::{
    Ray,
    Transform,
    Vec2,
    Vec3,
};

use crate::{
    drag::Drag,
    element::Element,
    snap::{
        Flick,
        Flicks,
        GRID,
    },
    unavi::{
        author::api::{
            Edit,
            request_redo,
            request_undo,
        },
        tool::api::{
            Tool,
            ToolButton,
            ToolEvent,
        },
    },
    wired::{
        input::{
            device::{
                self,
                Claim,
                Control,
            },
            types::{
                Action,
                InputSubscription,
                Key,
                PointerKind,
            },
        },
        physics::simulation::{
            RayFilter,
            raycast,
        },
        portal::portals::current_space,
        scene::{
            document::{
                open_document,
                script_document,
            },
            properties::Property,
        },
    },
};

mod drag;
mod element;
mod glyph;
mod snap;

wired_guest::generate_script!(Script);

/// Furthest an element can be picked from, in metres.
const REACH: f32 = 50.0;
/// Metres of distance per scroll notch.
const SCROLL_STEP: f32 = 0.25;
/// Metres per second of distance at full stick.
const PUSH_SPEED: f32 = 3.0;
/// Stick travel below which it reads as centred.
const DEADZONE: f32 = 0.2;

struct Script {
    tool:     Tool,
    active:   Cell<bool>,
    hand:     Cell<PointerKind>,
    drag:     RefCell<Option<Drag>>,
    /// The button whose release ends the drag.
    drag_by:  Cell<Option<ToolButton>>,
    modifier: Cell<bool>,
    /// The tool hand's stick while the grip steers a drag.
    steer:    RefCell<Option<Claim>>,
    /// The tool hand's stick while the other trigger is held with nothing
    /// moving, for undo, redo, copy and delete.
    command:  RefCell<Option<Claim>>,
    flicks:   Cell<Flicks>,
    /// Keys, for the desktop chords.
    keys:     InputSubscription,
    control:  Cell<[bool; 2]>,
}

impl Script {
    fn aim(&self) -> Option<Transform> {
        pointer_pose(self.hand.get()).or_else(camera_pose)
    }

    fn is_vr(&self) -> bool {
        self.hand.get() != PointerKind::Screen
    }

    /// The element the aim points at, and where the ray met it.
    fn aimed(&self) -> Option<(Element, Vec3, Transform)> {
        let aim = self.aim()?;
        let hit = match raycast(
            Ray {
                origin:    aim.translation,
                direction: aim.forward(),
            },
            REACH,
            &RayFilter {
                exclude_local_agent: true,
                exclude_documents:   Vec::new(),
            },
        ) {
            Ok(hit) => hit?,
            Err(err) => {
                eprintln!("build: raycast: {err:?}");
                return None;
            }
        };
        let element = Element::of(hit.target)?;
        if !element.doc.may_write() {
            println!("build: that is not yours to change");
            return None;
        }
        Some((element, hit.point, aim))
    }

    /// The space's root in the tool's frame, which the grid is laid in, unless
    /// the modifier frees the drag from it.
    fn grid(&self) -> Option<Transform> {
        if self.modifier.get() {
            return None;
        }
        let space = open_document(current_space().ok()??).ok()??;
        script_document().ok()?.offset_to(&space)
    }

    fn begin(&self, by: ToolButton, copy: bool) {
        if self.drag.borrow().is_some() {
            return;
        }
        let Some((element, point, aim)) = self.aimed() else {
            return;
        };
        let (element, label) = if copy {
            match element.duplicate() {
                Ok(copy) => (copy, "Copy"),
                Err(err) => {
                    eprintln!("build: copy: {err}");
                    return;
                }
            }
        } else {
            (element, "Move")
        };
        match Drag::begin(element, point, &aim, label, copy) {
            Ok(drag) => {
                *self.drag.borrow_mut() = Some(drag);
                self.drag_by.set(Some(by));
                // The other trigger frees a drag from the grid; it is not a
                // command while something moves.
                self.command.borrow_mut().take();
            }
            Err((err, element)) => {
                eprintln!("build: {label}: {err}");
                if copy {
                    element.discard();
                }
            }
        }
    }

    fn end(&self) {
        self.steer.borrow_mut().take();
        self.drag_by.set(None);
        if let Some(drag) = self.drag.borrow_mut().take()
            && let Err(err) = drag.end()
        {
            eprintln!("build: keep: {err}");
        }
    }

    fn delete_aimed(&self) {
        if let Some((element, ..)) = self.aimed()
            && let Err(err) = element.delete()
        {
            eprintln!("build: delete: {err}");
        }
    }

    /// Copies the aimed element one grid step along the space's X axis, and
    /// keeps the copy. A copy that cannot be kept is taken back out.
    fn copy_aimed(&self) {
        let Some((element, ..)) = self.aimed() else {
            return;
        };
        let copy = match element.duplicate() {
            Ok(copy) => copy,
            Err(err) => {
                eprintln!("build: copy: {err}");
                return;
            }
        };
        let step = self
            .grid()
            .map_or(Vec3::X, |grid| grid.rotation * Vec3::X)
            * GRID;
        let kept = element.pose().and_then(|pose| {
            let at = Transform {
                translation: pose.translation + step,
                ..pose
            };
            let local = pose_in_parent(&copy.doc, copy.prim, at)
                .ok_or_else(|| anyhow::anyhow!("the copy's parent is not placed"))?;
            copy.doc
                .local()
                .set(copy.prim, Property::Transform(local))
                .flush()?;
            let edit = Edit::new("Copy")?;
            edit.track_creation(&copy.doc, copy.prim)?;
            edit.commit()?;
            Ok(())
        });
        if let Err(err) = kept {
            eprintln!("build: copy: {err}");
            copy.discard();
        }
    }

    fn take_stick(&self, slot: &RefCell<Option<Claim>>) {
        if !self.is_vr() || slot.borrow().is_some() {
            return;
        }
        match device::take_control(self.hand.get(), Control::Stick) {
            Ok(claim) => *slot.borrow_mut() = Some(claim),
            Err(err) => eprintln!("build: claim the stick: {err:?}"),
        }
    }

    fn on_event(&self, event: ToolEvent) {
        match event {
            ToolEvent::Activate(hand) => {
                self.active.set(true);
                self.hand.set(hand);
            }
            ToolEvent::Deactivate => {
                self.end();
                self.command.borrow_mut().take();
                self.modifier.set(false);
                self.active.set(false);
            }
            ToolEvent::Scroll(delta) => {
                if let Some(drag) = &mut *self.drag.borrow_mut() {
                    drag.push(delta * SCROLL_STEP);
                }
            }
            ToolEvent::Pressed(ToolButton::Primary) if self.active.get() => {
                self.begin(ToolButton::Primary, false);
            }
            ToolEvent::Pressed(ToolButton::Secondary) if self.active.get() => {
                if self.drag.borrow().is_some() {
                    self.take_stick(&self.steer);
                } else if !self.is_vr() {
                    self.begin(ToolButton::Secondary, true);
                }
            }
            ToolEvent::Pressed(ToolButton::Modifier) => {
                self.modifier.set(true);
                if self.active.get() && self.drag.borrow().is_none() {
                    self.take_stick(&self.command);
                }
            }
            ToolEvent::Released(ToolButton::Modifier) => {
                self.modifier.set(false);
                self.command.borrow_mut().take();
            }
            ToolEvent::Released(button) => {
                if button == ToolButton::Secondary {
                    self.steer.borrow_mut().take();
                }
                if self.drag_by.get() == Some(button) {
                    self.end();
                }
            }
            ToolEvent::SetState(_) | ToolEvent::Pressed(_) => {}
        }
    }

    /// The stick, while the grip steers a drag or the other trigger makes it
    /// a command.
    fn read_stick(&self, dt: f32) {
        if !self.is_vr() {
            return;
        }
        let steering = self.steer.borrow().is_some();
        let commanding = self.command.borrow().is_some() && self.drag.borrow().is_none();
        if !steering && !commanding {
            return;
        }
        let stick = stick(self.hand.get());
        let mut flicks = self.flicks.get();

        if steering {
            if let Some(drag) = &mut *self.drag.borrow_mut() {
                if stick.y.abs() > DEADZONE {
                    drag.push(stick.y * PUSH_SPEED * dt);
                }
                match flicks.read(stick.x, 0.0) {
                    Some(Flick::Left) => drag.turn(1),
                    Some(Flick::Right) => drag.turn(-1),
                    _ => {}
                }
            }
        } else {
            match flicks.read(stick.x, stick.y) {
                Some(Flick::Left) => request_undo(),
                Some(Flick::Right) => request_redo(),
                Some(Flick::Up) => self.copy_aimed(),
                Some(Flick::Down) => self.delete_aimed(),
                None => {}
            }
        }
        self.flicks.set(flicks);
    }

    /// The desktop chords. Shift is the modifier, which the shell forwards.
    fn read_keys(&self) {
        for event in self.keys.drain(16) {
            let (key, pressed) = match event.action {
                Action::KeyPressed(key) => (key, true),
                Action::KeyReleased(key) => (key, false),
                _ => continue,
            };
            let mut control = self.control.get();
            match key {
                Key::ControlLeft => control[0] = pressed,
                Key::ControlRight => control[1] = pressed,
                _ => {}
            }
            self.control.set(control);
            if !pressed || !self.active.get() {
                continue;
            }
            match key {
                Key::Delete | Key::Backspace => self.delete_aimed(),
                Key::D if control.contains(&true) => self.copy_aimed(),
                Key::R => {
                    if let Some(drag) = &mut *self.drag.borrow_mut() {
                        drag.turn(1);
                    }
                }
                _ => {}
            }
        }
    }
}

/// The stick of `hand`, centred when it is not tracked.
fn stick(hand: PointerKind) -> Vec2 {
    device::pointers()
        .ok()
        .and_then(|pointers| pointers.into_iter().find(|pointer| pointer.kind == hand))
        .map_or(Vec2::ZERO, |pointer| pointer.axis)
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        Ok(Self {
            tool:     Tool::new(
                "Build",
                "Moves, turns, copies and removes what is in your space.",
                Some(&glyph::mark()),
            )?,
            active:   Cell::new(false),
            hand:     Cell::new(PointerKind::Screen),
            drag:     RefCell::new(None),
            drag_by:  Cell::new(None),
            modifier: Cell::new(false),
            steer:    RefCell::new(None),
            command:  RefCell::new(None),
            flicks:   Cell::default(),
            keys:     device::listen()?,
            control:  Cell::new([false; 2]),
        })
    }

    fn fixed_update(&mut self, tick: Tick) -> anyhow::Result<()> {
        while let Some(event) = self.tool.poll() {
            self.on_event(event);
        }
        self.read_keys();
        self.read_stick(tick.dt);

        let grid = self.grid();
        let moved = match (self.aim(), &*self.drag.borrow()) {
            (Some(aim), Some(drag)) => drag.update(&aim, grid.as_ref()),
            _ => Ok(()),
        };
        // A move that cannot be written keeps where it got to, rather than
        // failing again every tick.
        if let Err(err) = moved {
            eprintln!("build: move: {err}");
            self.end();
        }
        Ok(())
    }
}
