//! The Inspector: points at anything and stands a slate beside it saying
//! what it is, with its transform to edit and its attributes to read.
//!
//! | Act | VR | Desktop |
//! | --- | --- | --- |
//! | inspect | trigger on it | left mouse on it |
//! | change a number | trigger on the row, drag sideways | left mouse on the row, drag |
//! | close | the Close row, or the grip | the Close row, or right mouse |

use std::cell::{
    Cell,
    RefCell,
};

use wired_guest::math::{
    Ray,
    Transform,
    Vec3,
};

use crate::{
    inspect::{
        Asked,
        Inspection,
    },
    unavi::{
        tool::api::{
            Tool,
            ToolButton,
            ToolEvent,
        },
        vui::api::{
            self,
            Slate,
        },
    },
    wired::{
        input::types::PointerKind,
        physics::simulation::{
            RayFilter,
            raycast,
        },
        scene::document::script_document,
    },
};

mod glyph;
mod inspect;

wired_guest::generate_script!(Script);

/// Furthest anything can be inspected from, in metres.
const REACH: f32 = 50.0;
/// Fixed ticks between redraws of the rows, so values others change, or
/// physics moves, show without a press.
const REFRESH_TICKS: u32 = 15;
/// Metres toward the viewer, and up, a slate stands from what it describes.
const TOWARD: f32 = 0.35;
const ABOVE: f32 = 0.4;

struct Script {
    tool:       Tool,
    active:     Cell<bool>,
    hand:       Cell<PointerKind>,
    slate:      Slate,
    inspection: RefCell<Option<Inspection>>,
    ticks:      Cell<u32>,
}

impl Script {
    fn aim(&self) -> Option<Transform> {
        pointer_pose(self.hand.get()).or_else(camera_pose)
    }

    /// Inspects what the aim points at. A hit on the tool's own slate is the
    /// slate's to answer.
    fn inspect(&self) {
        let Some(aim) = self.aim() else {
            return;
        };
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
            Ok(Some(hit)) => hit,
            Ok(None) => return,
            Err(err) => {
                eprintln!("inspector: raycast: {err:?}");
                return;
            }
        };
        if script_document().is_ok_and(|own| own.id() == hit.target.document) {
            return;
        }
        let Some(inspection) = Inspection::of(hit.target) else {
            return;
        };

        // Ends a drag still running on the slate, so it cannot carry on into
        // the element inspected next.
        self.slate.set_shown(false);
        self.step_slate();

        let toward = (aim.translation - hit.point) * Vec3::new(1.0, 0.0, 1.0);
        let at = hit.point + toward.normalize_or_zero() * TOWARD + Vec3::Y * ABOVE;
        self.slate.set_position(at);
        if let Err(err) = self.slate.set_rows(&inspection.rows()) {
            eprintln!("inspector: rows: {err:?}");
        }
        self.slate.set_shown(true);
        let previous = self.inspection.borrow_mut().replace(inspection);
        if let Some(mut previous) = previous {
            previous.finish();
        }
    }

    fn close(&self) {
        let closed = self.inspection.borrow_mut().take();
        if let Some(mut closed) = closed {
            closed.finish();
        }
        self.slate.set_shown(false);
    }

    /// Answers the slate, and redraws its rows after any change and every
    /// few ticks.
    fn step_slate(&self) {
        let events = self.slate.events();
        let mut closing = false;
        let mut changed = !events.is_empty();
        if let Some(inspection) = &mut *self.inspection.borrow_mut() {
            for event in &events {
                if matches!(inspection.apply(event), Asked::Close) {
                    closing = true;
                }
            }
            let ticks = self.ticks.get() + 1;
            self.ticks.set(ticks % REFRESH_TICKS);
            changed |= ticks >= REFRESH_TICKS;
            if changed
                && !closing
                && let Err(err) = self.slate.set_rows(&inspection.rows())
            {
                eprintln!("inspector: rows: {err:?}");
            }
        }
        if closing {
            self.close();
        }
    }
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        Ok(Self {
            tool:       Tool::new(
                "Inspector",
                "Shows what anything is, and edits where it stands.",
                Some(&glyph::mark()),
            )?,
            active:     Cell::new(false),
            hand:       Cell::new(PointerKind::Screen),
            slate:      Slate::new()?,
            inspection: RefCell::new(None),
            ticks:      Cell::new(0),
        })
    }

    fn fixed_update(&mut self, _tick: Tick) -> anyhow::Result<()> {
        api::fixed_update()?;
        while let Some(event) = self.tool.poll() {
            match event {
                ToolEvent::Activate(hand) => {
                    self.active.set(true);
                    self.hand.set(hand);
                }
                ToolEvent::Deactivate => {
                    self.active.set(false);
                    self.close();
                }
                ToolEvent::Pressed(ToolButton::Primary) if self.active.get() => self.inspect(),
                ToolEvent::Pressed(ToolButton::Secondary) if self.active.get() => self.close(),
                ToolEvent::SetState(_)
                | ToolEvent::Pressed(_)
                | ToolEvent::Released(_)
                | ToolEvent::Scroll(_) => {}
            }
        }
        self.step_slate();
        Ok(())
    }

    fn update(&mut self, tick: Tick) -> anyhow::Result<()> {
        api::update(tick.dt)?;
        Ok(())
    }
}
