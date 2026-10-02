//! A tool that grabs a prop at a distance and drags it around, held taut on
//! a beam from the camera.

use std::cell::{
    Cell,
    RefCell,
};

use wired_guest::math::{
    Color,
    Vec3,
};

use crate::{
    hold::Held,
    laser::Laser,
    outline::Outline,
    unavi::tool::api::{
        Tool,
        ToolEvent,
    },
};

mod hold;
mod laser;
mod outline;
mod palette;

wired_guest::generate_script!(Script);

const ARTIFACT_OFFSET: Vec3 = Vec3::new(0.22, -0.18, -0.5);
/// Metres of hold-distance change per scroll notch.
const SCROLL_STEP: f32 = 0.4;

struct Script {
    tool:    Tool,
    laser:   Laser,
    active:  Cell<bool>,
    color:   Cell<Color>,
    pressed: Cell<bool>,
    held:    RefCell<Option<Held>>,
    outline: Outline,
}

impl Script {
    fn release(&self) {
        if let Some(held) = self.held.borrow_mut().take() {
            held.release();
        }
        self.outline.clear();
    }
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        Ok(Self {
            tool:    Tool::new("Physgun", "Grabs a prop at a distance and drags it around.")?,
            laser:   Laser::new()?,
            active:  Cell::new(false),
            color:   Cell::new(palette::DEFAULT),
            pressed: Cell::new(false),
            held:    RefCell::new(None),
            outline: Outline::new()?,
        })
    }

    fn fixed_update(
        &mut self,
        _tick: exports::wired::script::lifecycle::Tick,
    ) -> anyhow::Result<()> {
        while let Some(event) = self.tool.poll() {
            match event {
                ToolEvent::Activate(_) => self.active.set(true),
                ToolEvent::Deactivate => {
                    self.active.set(false);
                    self.pressed.set(false);
                    self.release();
                }
                ToolEvent::SetState(state) => self.color.set(state.color),
                ToolEvent::Scroll(delta) => {
                    if let Some(held) = &mut *self.held.borrow_mut() {
                        held.nudge_distance(delta * SCROLL_STEP);
                    }
                }
                ToolEvent::Trigger(pressed) => {
                    if pressed && !self.pressed.get() && self.active.get() {
                        if let Some(cam) = camera_pose() {
                            let held = Held::grab(&cam);
                            if let Some(held) = &held
                                && let Some(collider) = held.collider()
                            {
                                self.outline.attach(&collider, self.color.get());
                            }
                            *self.held.borrow_mut() = held;
                        } else {
                            println!("physgun: no camera");
                        }
                    } else if !pressed {
                        self.release();
                    }
                    self.pressed.set(pressed);
                }
            }
        }

        if let Some(cam) = camera_pose()
            && let Some(held) = &*self.held.borrow()
        {
            held.update(&cam);
        }
        Ok(())
    }

    fn update(&mut self, _tick: exports::wired::script::lifecycle::Tick) -> anyhow::Result<()> {
        let Some(cam) = camera_pose() else {
            return Ok(());
        };
        // Re-read at render rate: reusing the fixed-rate grab point made the
        // beam step while the muzzle end swept smoothly.
        match self.held.borrow().as_ref() {
            Some(held) => {
                let muzzle = cam.translation + cam.rotation * ARTIFACT_OFFSET;
                self.laser.show(muzzle, held.grab_point(), self.color.get());
                self.outline.track(&held.body());
            }
            None => self.laser.hide(),
        }
        Ok(())
    }
}
