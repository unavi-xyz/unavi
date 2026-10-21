//! A tool that grabs things at a distance and moves them, held on a beam from
//! the hand that holds it. It is both the play tool and the building one.
//!
//! | Grabbed | Moved by | Let go |
//! | --- | --- | --- |
//! | a dynamic body | physics: dragged, spun, thrown | simulates; nothing is kept |
//! | a dynamic body, frozen | — | kinematic where it hangs, kept as an undoable edit |
//! | anything else: a wall, a kept object's site | editing where it is, snapped to the grid | kept as an undoable edit |
//!
//! | Act | VR | Desktop |
//! | --- | --- | --- |
//! | grab | trigger | left mouse |
//! | push, pull | squeeze the grip, stick up, down | wheel |
//! | spin a body, turn an element 15° | squeeze the grip, stick left, right | R turns an element |
//! | turn a body with the wrist, or move an element off the grid | hold the other trigger | hold Shift |
//! | freeze | let go of the trigger while squeezing the grip | right mouse while holding |
//! | thaw | grip aimed at a frozen body, or grab it | right mouse aimed at it, or grab it |
//! | copy and move the copy | — | right mouse on an element, drag |
//! | copy in place | the other trigger, flick the stick up | Ctrl+D |
//! | delete | the other trigger, flick the stick down | Delete |
//! | undo, redo | the other trigger, flick the stick left, right | Ctrl+Z, Ctrl+Shift+Z |
//!
//! The stick is the tool's only while the grip or the other trigger is held,
//! so it still walks and turns the user the rest of the time.

use std::cell::{
    Cell,
    RefCell,
};

use wired_guest::math::{
    Color,
    Transform,
    Vec2,
    Vec3,
};

use crate::{
    drag::Drag,
    element::Element,
    hold::{
        Held,
        Target,
    },
    laser::Laser,
    outline::Outline,
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
        core::ids::PrimRef,
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
        portal::portals::current_space,
        scene::{
            document::{
                open_document,
                script_document,
            },
            properties::{
                Property,
                PropertyKey,
                RigidBody,
                RigidBodyKind,
            },
        },
    },
};

mod drag;
mod element;
mod freeze;
mod glyph;
mod hold;
mod laser;
mod outline;
mod palette;
mod snap;

wired_guest::generate_script!(Script);

/// Metres of hold-distance change per scroll notch.
const SCROLL_STEP: f32 = 0.4;
/// Metres per second of hold-distance change at full stick.
const PUSH_SPEED: f32 = 4.0;
/// Radians per second of spin at full stick.
const SPIN_SPEED: f32 = 2.5;
/// Stick travel below which it reads as centred.
const DEADZONE: f32 = 0.2;

/// What the beam holds.
enum Holding {
    Body(Held),
    /// An element being placed, and the button whose release keeps it.
    Element(Drag, ToolButton),
}

struct Script {
    tool:     Tool,
    laser:    Laser,
    active:   Cell<bool>,
    /// The pointer it aims with while in hand.
    hand:     Cell<PointerKind>,
    color:    Cell<Color>,
    /// Whether the other trigger is held.
    modifier: Cell<bool>,
    /// Whether the grip is squeezed, so a body grabbed with it down is
    /// steered from the start.
    squeezed: Cell<bool>,
    holding:  RefCell<Option<Holding>>,
    /// The tool hand's stick while the grip steers what is held.
    steer:    RefCell<Option<Claim>>,
    /// The tool hand's stick while the other trigger is held with nothing
    /// held, for undo, redo, copy and delete.
    command:  RefCell<Option<Claim>>,
    flicks:   Cell<Flicks>,
    /// Keys, for the desktop chords.
    keys:     InputSubscription,
    control:  Cell<[bool; 2]>,
    outline:  Outline,
}

impl Script {
    /// Where the tool aims from: the hand holding it, else the eye.
    fn aim(&self) -> Option<Transform> {
        pointer_pose(self.hand.get()).or_else(camera_pose)
    }

    fn is_vr(&self) -> bool {
        self.hand.get() != PointerKind::Screen
    }

    fn is_holding(&self) -> bool {
        self.holding.borrow().is_some()
    }

    /// The space's root in the tool's frame, which an element snaps to the
    /// grid in, unless the other trigger frees it.
    fn grid(&self) -> Option<Transform> {
        if self.modifier.get() {
            return None;
        }
        let space = open_document(current_space().ok()??).ok()??;
        script_document().ok()?.offset_to(&space)
    }

    fn grab(&self) {
        let Some(aim) = self.aim() else {
            println!("physgun: nothing to aim with");
            return;
        };
        let Some(target) = Target::under(&aim) else {
            return;
        };
        self.release();

        match body_kind(&target) {
            Some(RigidBodyKind::Dynamic) => self.hold_body(&aim, target),
            Some(RigidBodyKind::Kinematic) if target.doc.may_write() => {
                // A frozen body is kinematic, which a velocity cannot drag.
                if let Err(err) = freeze::thaw(&target.doc, target.prim) {
                    eprintln!("physgun: thaw before grabbing: {err}");
                    return;
                }
                self.hold_body(&aim, target);
            }
            _ => self.place(&aim, &target, ToolButton::Primary, false),
        }
    }

    fn hold_body(&self, aim: &Transform, target: Target) {
        let Some(mut held) = Held::grab(aim, target) else {
            return;
        };
        held.set_wrist(aim, self.modifier.get());
        if let Some(collider) = held.collider() {
            self.outline.attach(&collider, self.color.get());
        }
        *self.holding.borrow_mut() = Some(Holding::Body(held));
        self.command.borrow_mut().take();
        if self.squeezed.get() {
            self.take_stick(&self.steer);
        }
    }

    /// Starts placing the element `target` is part of, or a copy of it.
    fn place(&self, aim: &Transform, target: &Target, by: ToolButton, copy: bool) {
        let Some(element) = Element::of(PrimRef {
            document: target.doc.id(),
            prim:     target.prim,
        }) else {
            return;
        };
        if !element.doc.may_write() {
            println!("physgun: that is not yours to move");
            return;
        }
        let (element, label) = if copy {
            match element.duplicate() {
                Ok(copy) => (copy, "Copy"),
                Err(err) => {
                    eprintln!("physgun: copy: {err}");
                    return;
                }
            }
        } else {
            (element, "Move")
        };
        match Drag::begin(element, target.point, aim, label, copy) {
            Ok(drag) => {
                *self.holding.borrow_mut() = Some(Holding::Element(drag, by));
                self.command.borrow_mut().take();
                if self.squeezed.get() {
                    self.take_stick(&self.steer);
                }
            }
            Err((err, element)) => {
                eprintln!("physgun: {label}: {err}");
                if copy {
                    element.discard();
                }
            }
        }
    }

    /// Lets go of whatever is held: a body keeps its velocity, so a fast
    /// sweep throws it, and an element is kept where it is.
    fn release(&self) {
        self.steer.borrow_mut().take();
        self.outline.clear();
        let holding = self.holding.borrow_mut().take();
        match holding {
            Some(Holding::Body(held)) => held.release(),
            Some(Holding::Element(drag, _)) => {
                if let Err(err) = drag.end() {
                    eprintln!("physgun: keep: {err}");
                }
            }
            None => {}
        }
    }

    /// A button came up. Letting go of the trigger while the grip steers a
    /// body freezes it where it is.
    fn released(&self, button: ToolButton) {
        let steering = self.steer.borrow().is_some();
        let (freezes, ends) = match &*self.holding.borrow() {
            Some(Holding::Body(_)) => (
                button == ToolButton::Primary && steering,
                button == ToolButton::Primary,
            ),
            Some(Holding::Element(_, by)) => (false, *by == button),
            None => (false, false),
        };
        if freezes {
            self.freeze_held();
        } else if ends {
            self.release();
        }
    }

    /// The grip went down: steer what is held in VR, freeze a body on
    /// desktop, or with nothing held thaw the frozen body aimed at, or on
    /// desktop start placing a copy of the element aimed at.
    fn squeeze(&self) {
        self.squeezed.set(true);
        let holding_body = matches!(&*self.holding.borrow(), Some(Holding::Body(_)));
        if self.is_holding() {
            if self.is_vr() {
                self.take_stick(&self.steer);
            } else if holding_body {
                self.freeze_held();
            }
            return;
        }
        let Some(aim) = self.aim() else {
            return;
        };
        let Some(target) = Target::under(&aim) else {
            return;
        };
        if freeze::is_frozen(&target.doc, target.prim) {
            if let Err(err) = freeze::thaw(&target.doc, target.prim) {
                eprintln!("physgun: thaw: {err}");
            }
        } else if !self.is_vr() {
            self.place(&aim, &target, ToolButton::Secondary, true);
        }
    }

    /// Freezes the body held where it is, and lets go of it.
    fn freeze_held(&self) {
        let froze = match &*self.holding.borrow() {
            Some(Holding::Body(held)) => Some(freeze::freeze(held.doc(), held.prim())),
            _ => None,
        };
        if let Some(Err(err)) = froze {
            eprintln!("physgun: freeze: {err}");
        }
        self.release();
    }

    fn set_modifier(&self, held: bool) {
        self.modifier.set(held);
        if !held {
            self.command.borrow_mut().take();
        } else if self.active.get() && !self.is_holding() {
            self.take_stick(&self.command);
        }
        if let Some(aim) = self.aim()
            && let Some(Holding::Body(body)) = &mut *self.holding.borrow_mut()
        {
            body.set_wrist(&aim, held);
        }
    }

    fn take_stick(&self, slot: &RefCell<Option<Claim>>) {
        if !self.is_vr() || slot.borrow().is_some() {
            return;
        }
        match device::take_control(self.hand.get(), Control::Stick) {
            Ok(claim) => *slot.borrow_mut() = Some(claim),
            Err(err) => eprintln!("physgun: claim the stick: {err:?}"),
        }
    }

    /// The element the aim points at, when the user may change it.
    fn aimed_element(&self) -> Option<Element> {
        let target = Target::under(&self.aim()?)?;
        let element = Element::of(PrimRef {
            document: target.doc.id(),
            prim:     target.prim,
        })?;
        if !element.doc.may_write() {
            println!("physgun: that is not yours to change");
            return None;
        }
        Some(element)
    }

    fn delete_aimed(&self) {
        if let Some(element) = self.aimed_element()
            && let Err(err) = element.delete()
        {
            eprintln!("physgun: delete: {err}");
        }
    }

    /// Copies the aimed element one grid step along the space's X axis, and
    /// keeps the copy. A copy that cannot be kept is taken back out.
    fn copy_aimed(&self) {
        let Some(element) = self.aimed_element() else {
            return;
        };
        let copy = match element.duplicate() {
            Ok(copy) => copy,
            Err(err) => {
                eprintln!("physgun: copy: {err}");
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
            eprintln!("physgun: copy: {err}");
            copy.discard();
        }
    }

    fn on_event(&self, event: ToolEvent) {
        match event {
            ToolEvent::Activate(hand) => {
                self.active.set(true);
                self.hand.set(hand);
            }
            ToolEvent::Deactivate => {
                self.release();
                self.command.borrow_mut().take();
                self.modifier.set(false);
                self.squeezed.set(false);
                self.active.set(false);
            }
            ToolEvent::SetState(state) => self.color.set(state.color),
            ToolEvent::Scroll(delta) => match &mut *self.holding.borrow_mut() {
                Some(Holding::Body(held)) => held.nudge_distance(delta * SCROLL_STEP),
                Some(Holding::Element(drag, _)) => drag.push(delta * SCROLL_STEP),
                None => {}
            },
            ToolEvent::Pressed(ToolButton::Primary) if self.active.get() => self.grab(),
            ToolEvent::Pressed(ToolButton::Secondary) if self.active.get() => self.squeeze(),
            ToolEvent::Pressed(ToolButton::Modifier) => self.set_modifier(true),
            ToolEvent::Released(ToolButton::Modifier) => self.set_modifier(false),
            ToolEvent::Released(button) => {
                if button == ToolButton::Secondary {
                    self.squeezed.set(false);
                    self.steer.borrow_mut().take();
                }
                self.released(button);
            }
            ToolEvent::Pressed(_) => {}
        }
    }

    /// The stick, while the grip steers what is held or the other trigger
    /// makes it a command.
    fn read_stick(&self, aim: &Transform, dt: f32) {
        let steering = self.steer.borrow().is_some();
        let commanding = self.command.borrow().is_some() && !self.is_holding();
        if !steering && !commanding {
            return;
        }
        let stick = stick(self.hand.get());
        let mut flicks = self.flicks.get();

        if steering {
            match &mut *self.holding.borrow_mut() {
                Some(Holding::Body(held)) => {
                    if stick.y.abs() > DEADZONE {
                        held.nudge_distance(stick.y * PUSH_SPEED * dt);
                    }
                    if stick.x.abs() > DEADZONE {
                        held.spin(aim, -stick.x * SPIN_SPEED * dt);
                    }
                }
                Some(Holding::Element(drag, _)) => {
                    if stick.y.abs() > DEADZONE {
                        drag.push(stick.y * PUSH_SPEED * dt);
                    }
                    match flicks.read(stick.x, 0.0) {
                        Some(Flick::Left) => drag.turn(1),
                        Some(Flick::Right) => drag.turn(-1),
                        _ => {}
                    }
                }
                None => {}
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
                Key::Delete | Key::Backspace if !self.is_holding() => self.delete_aimed(),
                Key::D if control.contains(&true) && !self.is_holding() => self.copy_aimed(),
                Key::R => {
                    if let Some(Holding::Element(drag, _)) = &mut *self.holding.borrow_mut() {
                        drag.turn(1);
                    }
                }
                _ => {}
            }
        }
    }
}

/// The kind of body `target` is, `None` for a prim with no body.
fn body_kind(target: &Target) -> Option<RigidBodyKind> {
    match target.doc.get(target.prim, &PropertyKey::RigidBody) {
        Some(Property::RigidBody(RigidBody { kind, .. })) => Some(kind),
        _ => None,
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
                "Physgun",
                "Grabs things at a distance: throw them, freeze them, or place them.",
                Some(&glyph::mark()),
            )?,
            laser:    Laser::new()?,
            active:   Cell::new(false),
            hand:     Cell::new(PointerKind::Screen),
            color:    Cell::new(palette::DEFAULT),
            modifier: Cell::new(false),
            squeezed: Cell::new(false),
            holding:  RefCell::new(None),
            steer:    RefCell::new(None),
            command:  RefCell::new(None),
            flicks:   Cell::default(),
            keys:     device::listen()?,
            control:  Cell::new([false; 2]),
            outline:  Outline::new()?,
        })
    }

    fn fixed_update(&mut self, tick: Tick) -> anyhow::Result<()> {
        while let Some(event) = self.tool.poll() {
            self.on_event(event);
        }
        self.read_keys();
        let Some(aim) = self.aim() else {
            return Ok(());
        };
        self.read_stick(&aim, tick.dt);

        let grid = self.grid();
        let moved = match &*self.holding.borrow() {
            Some(Holding::Body(held)) => {
                held.update(&aim);
                Ok(())
            }
            Some(Holding::Element(drag, _)) => drag.update(&aim, grid.as_ref()),
            None => Ok(()),
        };
        // A move that cannot be written keeps where it got to, rather than
        // failing again every tick.
        if let Err(err) = moved {
            eprintln!("physgun: move: {err}");
            self.release();
        }
        Ok(())
    }

    fn update(&mut self, _tick: Tick) -> anyhow::Result<()> {
        // Re-read at render rate: reusing the fixed-rate grab point made the
        // beam step while the muzzle end swept smoothly.
        let muzzle = held_tool_pose(self.hand.get());
        let aim = self.aim();
        match (&*self.holding.borrow(), muzzle, aim) {
            (Some(Holding::Body(held)), Some(muzzle), _) => {
                self.laser
                    .show(muzzle.translation, held.grab_point(), self.color.get());
                self.outline.track(&held.body());
            }
            (Some(Holding::Element(drag, _)), Some(muzzle), Some(aim)) => {
                self.laser
                    .show(muzzle.translation, drag.point(&aim), self.color.get());
            }
            _ => self.laser.hide(),
        }
        Ok(())
    }
}
