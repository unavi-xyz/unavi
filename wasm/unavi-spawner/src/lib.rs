use std::{
    cell::{
        Cell,
        RefCell,
    },
    time::SystemTime,
};

use wired_prelude::prelude::*;

use crate::{
    preview::Preview,
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
            types::{
                Prim,
                Xform,
            },
        },
    },
};

mod palette;
mod preview;
mod spawn;

wired_prelude::generate_script!(Script);

const ARTIFACT_OFFSET: Vec3 = Vec3::new(0.22, -0.18, -0.5);
const ICON_SIZE: f32 = 0.05;
const ART_SPEED: f32 = 5.0;

struct Script {
    tool:    Tool,
    preview: Preview,
    camera:  RefCell<Option<Prim>>,
    active:  Cell<bool>,
    color:   Cell<Color>,
    pressed: Cell<bool>,
    art_t:   Cell<f32>,
    time:    SystemTime,
}

impl Script {
    fn camera(&self) -> Option<Transform> {
        let mut cam = self.camera.borrow_mut();
        if cam.is_none() {
            *cam = local_camera().ok();
        }
        cam.as_ref().map(Prim::global_xform)
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
            tool:    Tool::new(
                "Spawner",
                "Puts a cube down on whatever you point at.",
                &icon,
            ),
            preview: Preview::new(),
            camera:  RefCell::new(None),
            active:  Cell::new(false),
            color:   Cell::new(palette::DEFAULT),
            pressed: Cell::new(false),
            art_t:   Cell::new(0.0),
            time:    SystemTime::now(),
        })
    }

    fn fixed_update(&mut self) -> anyhow::Result<()> {
        while let Some(event) = self.tool.poll() {
            match event {
                ToolEvent::Activate(_) => self.active.set(true),
                ToolEvent::Deactivate => {
                    self.active.set(false);
                    self.pressed.set(false);
                }
                ToolEvent::SetState(state) => self.color.set(state.color),
                ToolEvent::Scroll(_) => {}
                ToolEvent::Trigger(pressed) => {
                    if pressed
                        && !self.pressed.get()
                        && self.active.get()
                        && let Some(cam) = self.camera()
                        && let Err(err) = spawn::spawn(self.color.get(), &cam)
                    {
                        eprintln!("spawn failed: {err:?}");
                    }
                    self.pressed.set(pressed);
                }
            }
        }
        Ok(())
    }

    fn update(&mut self) -> anyhow::Result<()> {
        let delta = self.time.elapsed().expect("elapsed").as_secs_f32();
        self.time = SystemTime::now();

        let Some(cam) = self.camera() else {
            return Ok(());
        };

        let art_t = approach(self.art_t.get(), self.active.get(), delta * ART_SPEED);
        self.art_t.set(art_t);
        self.preview
            .update(&cam, ARTIFACT_OFFSET, art_t, self.color.get(), delta);
        Ok(())
    }
}

fn approach(current: f32, toward_one: bool, step: f32) -> f32 {
    if toward_one {
        (current + step).min(1.0)
    } else {
        (current - step).max(0.0)
    }
}
