//! A tool that puts a cube down on whatever it points at.

use std::cell::Cell;

use wired_guest::math::Color;

use crate::{
    preview::Preview,
    unavi::tool::api::{
        Tool,
        ToolEvent,
    },
};

mod palette;
mod preview;
mod spawn;

wired_guest::generate_script!(Script);

const ARTIFACT_OFFSET: wired_guest::math::Vec3 = wired_guest::math::Vec3::new(0.22, -0.18, -0.5);
const ART_SPEED: f32 = 5.0;

struct Script {
    tool:    Tool,
    preview: Preview,
    active:  Cell<bool>,
    color:   Cell<Color>,
    pressed: Cell<bool>,
    art_t:   Cell<f32>,
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        Ok(Self {
            tool:    Tool::new("Spawner", "Puts a cube down on whatever you point at.")?,
            preview: Preview::new(ARTIFACT_OFFSET, 0.0)?,
            active:  Cell::new(false),
            color:   Cell::new(palette::DEFAULT),
            pressed: Cell::new(false),
            art_t:   Cell::new(0.0),
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
                }
                ToolEvent::SetState(state) => self.color.set(state.color),
                ToolEvent::Scroll(_) => {}
                ToolEvent::Trigger(pressed) => {
                    if pressed
                        && !self.pressed.get()
                        && self.active.get()
                        && let Some(cam) = camera_pose()
                        && let Err(err) = spawn::spawn(self.color.get(), &cam)
                    {
                        eprintln!("spawner: spawn failed: {err:?}");
                    }
                    self.pressed.set(pressed);
                }
            }
        }
        Ok(())
    }

    fn update(&mut self, tick: exports::wired::script::lifecycle::Tick) -> anyhow::Result<()> {
        let art_t = approach(self.art_t.get(), self.active.get(), tick.dt * ART_SPEED);
        self.art_t.set(art_t);
        self.preview
            .update(ARTIFACT_OFFSET, art_t, self.color.get(), tick.dt);
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
