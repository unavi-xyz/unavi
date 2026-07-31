use std::cell::{
    Cell,
    RefCell,
};

use wired_prelude::prelude::*;

use crate::{
    hold::Held,
    laser::Laser,
    unavi::{
        gauntlet_tool::api::{
            Tool,
            ToolEvent,
        },
        shapes::api::Cuboid,
    },
    wired::{
        agent::api::local_camera,
        scene::{
            api::self_document,
            types::{
                Prim,
                Xform,
            },
        },
    },
};

mod hold;
mod laser;
mod palette;

wired_prelude::generate_script!(Script);

const ARTIFACT_OFFSET: Vec3 = Vec3::new(0.22, -0.18, -0.5);
const ICON_SIZE: f32 = 0.05;
/// Metres of hold-distance change per scroll notch.
const SCROLL_STEP: f32 = 0.4;

struct Script {
    tool:     Tool,
    laser:    Laser,
    camera:   RefCell<Option<Prim>>,
    active:   Cell<bool>,
    color:    Cell<Color>,
    pressed:  Cell<bool>,
    held:     RefCell<Option<Held>>,
    hold_pos: Cell<Option<Vec3>>,
}

impl Script {
    fn camera(&self) -> Option<Transform> {
        let mut cam = self.camera.borrow_mut();
        if cam.is_none() {
            *cam = local_camera().ok();
        }
        cam.as_ref().map(Prim::global_xform)
    }

    fn release(&self) {
        if let Some(held) = self.held.borrow_mut().take() {
            held.release();
        }
        self.hold_pos.set(None);
    }
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let cuboid = Cuboid::new(Vec3::splat(ICON_SIZE));
        cuboid.set_doc(self_document()?);
        let icon = cuboid.mesh();
        icon.set_xform(Some(Xform {
            translation: Vec3::ZERO,
            rotation:    Quat::IDENTITY,
            scale:       Vec3::ZERO,
        }))?;

        Ok(Self {
            tool:     Tool::new("Physgun", &icon),
            laser:    Laser::new(),
            camera:   RefCell::new(None),
            active:   Cell::new(false),
            color:    Cell::new(palette::DEFAULT),
            pressed:  Cell::new(false),
            held:     RefCell::new(None),
            hold_pos: Cell::new(None),
        })
    }

    fn tick(&mut self) -> anyhow::Result<()> {
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
                    println!("physgun: trigger {pressed} (active={})", self.active.get());
                    if pressed && !self.pressed.get() && self.active.get() {
                        if let Some(cam) = self.camera() {
                            *self.held.borrow_mut() = Held::grab(&cam);
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

        if let Some(cam) = self.camera()
            && let Some(held) = &*self.held.borrow()
        {
            self.hold_pos.set(Some(held.update(&cam)));
        }
        Ok(())
    }

    fn render(&mut self) -> anyhow::Result<()> {
        let Some(cam) = self.camera() else {
            return Ok(());
        };
        match self.hold_pos.get() {
            Some(pos) => {
                let muzzle = cam.translation + cam.rotation * ARTIFACT_OFFSET;
                self.laser.show(muzzle, pos, self.color.get());
            }
            None => self.laser.hide(),
        }
        Ok(())
    }
}
