//! A tool that puts a cube down on whatever it points at.

use std::cell::Cell;

use wired_guest::{
    math::Color,
    xform::ARTIFACT_OFFSET,
};

use crate::{
    preview::Preview,
    unavi::tool::api::{
        Tool,
        ToolButton,
        ToolEvent,
    },
    wired::input::types::PointerKind,
};

mod glyph;
mod palette;
mod preview;
mod spawn;

wired_guest::generate_script!(Script);

const ART_SPEED: f32 = 5.0;

struct Script {
    tool:    Tool,
    preview: Preview,
    active:  Cell<bool>,
    /// The pointer it aims with while in hand.
    hand:    Cell<PointerKind>,
    color:   Cell<Color>,
    art_t:   Cell<f32>,
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        Ok(Self {
            tool:    Tool::new(
                "Spawner",
                "Puts a cube down on whatever you point at.",
                Some(&glyph::mark()),
            )?,
            preview: Preview::new(ARTIFACT_OFFSET)?,
            active:  Cell::new(false),
            hand:    Cell::new(PointerKind::Screen),
            color:   Cell::new(palette::DEFAULT),
            art_t:   Cell::new(0.0),
        })
    }

    fn fixed_update(&mut self, _tick: Tick) -> anyhow::Result<()> {
        while let Some(event) = self.tool.poll() {
            match event {
                ToolEvent::Activate(hand) => {
                    self.active.set(true);
                    self.hand.set(hand);
                }
                ToolEvent::Deactivate => self.active.set(false),
                ToolEvent::SetState(state) => self.color.set(state.color),
                ToolEvent::Pressed(ToolButton::Primary) => {
                    if self.active.get()
                        && let Some(aim) = pointer_pose(self.hand.get()).or_else(camera_pose)
                        && let Err(err) = spawn::spawn(self.color.get(), &aim)
                    {
                        eprintln!("spawner: spawn failed: {err:?}");
                    }
                }
                ToolEvent::Pressed(_) | ToolEvent::Released(_) | ToolEvent::Scroll(_) => {}
            }
        }
        Ok(())
    }

    fn update(&mut self, tick: Tick) -> anyhow::Result<()> {
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
