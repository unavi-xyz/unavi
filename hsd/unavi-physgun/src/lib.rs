use std::cell::{
    Cell,
    RefCell,
};

use wired_prelude::prelude::*;

use crate::{
    hold::Held,
    laser::Laser,
    outline::Outline,
    unavi::{
        shapes::api::Cuboid,
        tool::api::{
            Tool,
            ToolEvent,
        },
    },
    wired::{
        agent::api::local_camera,
        scene::{
            api::self_document,
            types::Prim,
        },
    },
};

mod hold;
mod laser;
mod outline;
mod palette;

wired_prelude::generate_script!(Script);

const ARTIFACT_OFFSET: Vec3 = Vec3::new(0.22, -0.18, -0.5);
const ICON_SIZE: f32 = 0.05;
/// Metres of hold-distance change per scroll notch.
const SCROLL_STEP: f32 = 0.4;

struct Script {
    tool:    Tool,
    laser:   Laser,
    camera:  RefCell<Option<Prim>>,
    active:  Cell<bool>,
    color:   Cell<Color>,
    pressed: Cell<bool>,
    held:    RefCell<Option<Held>>,
    outline: Outline,
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
        self.outline.clear();
    }
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let cuboid = Cuboid::new(Vec3::splat(ICON_SIZE));
        cuboid.set_doc(self_document()?);
        let icon = cuboid.mesh();
        icon.set_xform(Some(Transform {
            translation: Vec3::ZERO,
            rotation:    Quat::IDENTITY,
            scale:       Vec3::ZERO,
        }))?;

        Ok(Self {
            tool:    Tool::new(
                "Physgun",
                "Grabs a prop at a distance and drags it around.",
                &icon,
            ),
            laser:   Laser::new(),
            camera:  RefCell::new(None),
            active:  Cell::new(false),
            color:   Cell::new(palette::DEFAULT),
            pressed: Cell::new(false),
            held:    RefCell::new(None),
            outline: Outline::default(),
        })
    }

    fn fixed_update(&mut self) -> anyhow::Result<()> {
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

        if let Some(cam) = self.camera()
            && let Some(held) = &*self.held.borrow()
        {
            held.update(&cam);
        }
        Ok(())
    }

    fn update(&mut self) -> anyhow::Result<()> {
        let Some(cam) = self.camera() else {
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
