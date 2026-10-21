//! A tool that grabs a prop at a distance and drags it around, held taut on
//! a beam from the hand that holds it.
//!
//! | Act | VR | Desktop |
//! | --- | --- | --- |
//! | grab | trigger | left mouse |
//! | push, pull | squeeze the grip, stick up, down | wheel |
//! | spin | squeeze the grip, stick left, right | — |
//! | turn with the wrist | hold the other trigger | hold Shift |
//! | freeze | let go of the trigger while squeezing the grip | right mouse while holding |
//! | thaw | grip, aimed at a frozen body | right mouse, aimed at a frozen body |
//!
//! The stick is the tool's only while the grip is squeezed, so it still walks
//! and turns the user the rest of the time.

use std::cell::{
    Cell,
    RefCell,
};

use wired_guest::math::{
    Color,
    Transform,
    Vec2,
};

use crate::{
    freeze::Frozen,
    hold::{
        Held,
        Target,
    },
    laser::Laser,
    outline::Outline,
    unavi::tool::api::{
        Tool,
        ToolButton,
        ToolEvent,
    },
    wired::input::{
        device::{
            self,
            Claim,
            Control,
        },
        types::PointerKind,
    },
};

mod freeze;
mod glyph;
mod hold;
mod laser;
mod outline;
mod palette;

wired_guest::generate_script!(Script);

/// Metres of hold-distance change per scroll notch.
const SCROLL_STEP: f32 = 0.4;
/// Metres per second of hold-distance change at full stick.
const PUSH_SPEED: f32 = 4.0;
/// Radians per second of spin at full stick.
const SPIN_SPEED: f32 = 2.5;
/// Stick travel below which it reads as centred.
const DEADZONE: f32 = 0.2;

struct Script {
    tool:    Tool,
    laser:   Laser,
    active:  Cell<bool>,
    /// The pointer it aims with while in hand.
    hand:    Cell<PointerKind>,
    color:   Cell<Color>,
    /// Whether a held body turns with the whole aim.
    wrist:   Cell<bool>,
    /// Whether the grip is squeezed, so a body grabbed with it down is
    /// steered from the start.
    squeezed: Cell<bool>,
    held:    RefCell<Option<Held>>,
    /// The tool hand's stick, taken while the grip is squeezed on a held
    /// body so steering it does not also walk or turn the user.
    stick:   RefCell<Option<Claim>>,
    frozen:  RefCell<Frozen>,
    outline: Outline,
}

impl Script {
    /// Where the tool aims from: the hand holding it, else the eye.
    fn aim(&self) -> Option<Transform> {
        pointer_pose(self.hand.get()).or_else(camera_pose)
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
        // A frozen body is kinematic, which a velocity cannot drag.
        if let Err(err) = self.frozen.borrow_mut().thaw(&target.doc, target.prim) {
            eprintln!("physgun: thaw before grabbing: {err}");
        }

        let Some(mut held) = Held::grab(&aim, target) else {
            return;
        };
        held.set_wrist(&aim, self.wrist.get());
        if let Some(collider) = held.collider() {
            self.outline.attach(&collider, self.color.get());
        }
        *self.held.borrow_mut() = Some(held);
        if self.squeezed.get() {
            self.steer_with_stick();
        }
    }

    /// Takes the tool hand's stick for steering. Desktop has none.
    fn steer_with_stick(&self) {
        if self.hand.get() == PointerKind::Screen || self.is_steering() {
            return;
        }
        match device::take_control(self.hand.get(), Control::Stick) {
            Ok(claim) => *self.stick.borrow_mut() = Some(claim),
            Err(err) => eprintln!("physgun: claim the stick: {err:?}"),
        }
    }

    fn release(&self) {
        if let Some(held) = self.held.borrow_mut().take() {
            held.release();
        }
        self.stick.borrow_mut().take();
        self.outline.clear();
    }

    fn is_holding(&self) -> bool {
        self.held.borrow().is_some()
    }

    fn is_steering(&self) -> bool {
        self.stick.borrow().is_some()
    }

    /// The grip went down: steer what is held in VR, freeze it on desktop,
    /// or thaw the frozen body aimed at when nothing is held.
    fn squeeze(&self) {
        self.squeezed.set(true);
        if !self.is_holding() {
            self.thaw_aimed();
        } else if self.hand.get() == PointerKind::Screen {
            self.freeze_held();
        } else {
            self.steer_with_stick();
        }
    }

    /// The trigger came up. Let go while steering, a body stays where it is.
    fn let_go(&self) {
        if self.is_steering() && self.is_holding() {
            self.freeze_held();
        } else {
            self.release();
        }
    }

    /// Freezes what is held where it is, and lets go of it.
    fn freeze_held(&self) {
        let froze = self
            .held
            .borrow()
            .as_ref()
            .map(|held| self.frozen.borrow_mut().freeze(held.doc(), held.prim()));
        if let Some(Err(err)) = froze {
            eprintln!("physgun: freeze: {err}");
        }
        self.release();
    }

    fn thaw_aimed(&self) {
        if let Some(aim) = self.aim()
            && let Some(target) = Target::under(&aim)
            && let Err(err) = self.frozen.borrow_mut().thaw(&target.doc, target.prim)
        {
            eprintln!("physgun: thaw: {err}");
        }
    }

    fn set_wrist(&self, wrist: bool) {
        self.wrist.set(wrist);
        if let Some(aim) = self.aim()
            && let Some(held) = &mut *self.held.borrow_mut()
        {
            held.set_wrist(&aim, wrist);
        }
    }

    /// Pushes, pulls and spins what is held by the tool hand's stick,
    /// while the grip is squeezed.
    fn steer(&self, held: &mut Held, aim: &Transform, dt: f32) {
        if !self.is_steering() {
            return;
        }
        let stick = stick(self.hand.get());
        if stick.y.abs() > DEADZONE {
            held.nudge_distance(stick.y * PUSH_SPEED * dt);
        }
        if stick.x.abs() > DEADZONE {
            held.spin(aim, -stick.x * SPIN_SPEED * dt);
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
            tool:    Tool::new(
                "Physgun",
                "Grabs a prop at a distance and drags it around.",
                Some(&glyph::mark()),
            )?,
            laser:   Laser::new()?,
            active:  Cell::new(false),
            hand:    Cell::new(PointerKind::Screen),
            color:   Cell::new(palette::DEFAULT),
            wrist:   Cell::new(false),
            squeezed: Cell::new(false),
            held:    RefCell::new(None),
            stick:   RefCell::new(None),
            frozen:  RefCell::default(),
            outline: Outline::new()?,
        })
    }

    fn fixed_update(&mut self, tick: Tick) -> anyhow::Result<()> {
        while let Some(event) = self.tool.poll() {
            match event {
                ToolEvent::Activate(hand) => {
                    self.active.set(true);
                    self.hand.set(hand);
                }
                ToolEvent::Deactivate => {
                    self.active.set(false);
                    self.wrist.set(false);
                    self.squeezed.set(false);
                    self.release();
                }
                ToolEvent::SetState(state) => self.color.set(state.color),
                ToolEvent::Scroll(delta) => {
                    if let Some(held) = &mut *self.held.borrow_mut() {
                        held.nudge_distance(delta * SCROLL_STEP);
                    }
                }
                ToolEvent::Pressed(ToolButton::Primary) if self.active.get() => self.grab(),
                ToolEvent::Released(ToolButton::Primary) => self.let_go(),
                ToolEvent::Pressed(ToolButton::Secondary) if self.active.get() => self.squeeze(),
                ToolEvent::Released(ToolButton::Secondary) => {
                    self.squeezed.set(false);
                    self.stick.borrow_mut().take();
                }
                ToolEvent::Pressed(ToolButton::Modifier) => self.set_wrist(true),
                ToolEvent::Released(ToolButton::Modifier) => self.set_wrist(false),
                ToolEvent::Pressed(_) => {}
            }
        }

        if let Some(aim) = self.aim()
            && let Some(held) = &mut *self.held.borrow_mut()
        {
            self.steer(held, &aim, tick.dt);
            held.update(&aim);
        }
        Ok(())
    }

    fn update(&mut self, _tick: Tick) -> anyhow::Result<()> {
        // Re-read at render rate: reusing the fixed-rate grab point made the
        // beam step while the muzzle end swept smoothly.
        match (self.held.borrow().as_ref(), held_tool_pose(self.hand.get())) {
            (Some(held), Some(muzzle)) => {
                self.laser
                    .show(muzzle.translation, held.grab_point(), self.color.get());
                self.outline.track(&held.body());
            }
            _ => self.laser.hide(),
        }
        Ok(())
    }
}
