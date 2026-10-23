//! The Inspector: points at anything and stands a slate saying what it is,
//! tethered to the point inspected, with a tab for each way to work on it.
//!
//! The slate is a thing in the world, the same in a headset and on desktop:
//! it opens at reading distance beside what it describes, is picked up and
//! carried like anything else, and floats where it is let go. Moving things is
//! the physgun's; the Inspector reads them, and nudges a value by scrubbing its
//! row.
//!
//! | Act | VR | Desktop |
//! | --- | --- | --- |
//! | inspect | trigger on it | left mouse on it |
//! | move the slate | squeeze the grip on it and carry it; let go and it floats | right mouse on it, drag |
//! | change tab | trigger on a tab | left mouse on a tab |
//! | nudge position, turn or scale | trigger on its row, slide across | left mouse on its row, drag sideways |
//! | inspect its parent or a child | the rows of the Tree tab | the same |
//! | lock, delete, make local | the rows of the Transform tab | the same |
//! | close | the Close row, or squeeze the grip on nothing | the Close row, or right mouse on nothing |

use std::cell::RefCell;

use wired_guest::math::{
    Ray,
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
        core::ids::PrimRef,
        input::types::PointerKind,
        physics::simulation::{
            RayFilter,
            raycast,
        },
        scene::document::{
            Document,
            script_document,
        },
    },
};

mod glyph;
mod inspect;
mod stand;
mod tab;

wired_guest::generate_script!(Script);

/// Furthest anything can be inspected from, in metres.
const REACH: f32 = 50.0;
/// Fixed ticks between redraws of the rows, so values others change, or
/// physics moves, show without a press.
const REFRESH_TICKS: u32 = 15;
/// Metres from the eye past which an open slate is left behind, so
/// inspecting something new stands it again beside that.
const LEFT_BEHIND: f32 = 3.0;

struct Script {
    tool:       Tool,
    own:        Document,
    /// The pointer the tool aims with, while it is in hand.
    hand:       Option<PointerKind>,
    slate:      Slate,
    inspection: RefCell<Option<Inspection>>,
    ticks:      u32,
}

impl Script {
    fn aim(&self) -> Option<Ray> {
        let aim = self.hand.and_then(pointer_pose).or_else(camera_pose)?;
        Some(Ray {
            origin:    aim.translation,
            direction: aim.forward(),
        })
    }

    /// Inspects what the aim points at. A hit on the tool's own slate is the
    /// slate's to answer.
    fn press(&self) {
        let Some(ray) = self.aim() else {
            return;
        };
        let hit = match raycast(
            ray,
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
        if self.own.id() != hit.target.document {
            self.open(hit.target, Some(hit.point));
        }
    }

    /// Inspects `target`, tethered to `point` on it, else to its origin.
    /// Inspecting from the world starts on the first tab; moving on from
    /// what is inspected stays on the tab in use.
    fn open(&self, target: PrimRef, point: Option<Vec3>) {
        let Some(mut inspection) = Inspection::of(target, point) else {
            return;
        };
        let previous = self.inspection.borrow_mut().take();
        let was_open = previous.is_some();
        if let Some(mut previous) = previous {
            if point.is_none() {
                inspection.set_tab(previous.tab());
            }
            previous.finish();
        }
        // Rows first, so the slate is measured at the height it will show.
        if let Err(err) = self.slate.set_rows(&inspection.rows()) {
            eprintln!("inspector: rows: {err:?}");
        }
        if let Some(point) = point
            && self.left_behind(was_open)
            && let Some(eye) = camera_pose()
        {
            let at = stand::stand(eye.translation, point, self.slate.size(), self.own.id());
            self.slate.set_position(at);
        }
        self.slate.set_tether(inspection.anchor());
        self.slate.set_shown(true);
        *self.inspection.borrow_mut() = Some(inspection);
    }

    /// Whether the slate should stand again beside what is inspected next:
    /// it was put away, or the viewer has walked off from it.
    fn left_behind(&self, was_open: bool) -> bool {
        if !was_open {
            return true;
        }
        camera_pose()
            .is_none_or(|eye| eye.translation.distance(self.slate.position()) > LEFT_BEHIND)
    }

    fn close(&self) {
        let closed = self.inspection.borrow_mut().take();
        if let Some(mut closed) = closed {
            closed.finish();
        }
        self.slate.set_tether(None);
        self.slate.set_shown(false);
    }

    /// Answers the slate, and redraws its rows after any change and every
    /// few ticks.
    fn step_slate(&mut self) {
        let events = self.slate.events();
        let mut closing = false;
        let mut next = None;
        let mut changed = !events.is_empty();
        self.ticks = (self.ticks + 1) % REFRESH_TICKS;
        changed |= self.ticks == 0;
        if let Some(inspection) = &mut *self.inspection.borrow_mut() {
            for event in &events {
                match inspection.apply(event) {
                    Asked::Nothing => {}
                    Asked::Close => closing = true,
                    Asked::Inspect(target) => next = Some(target),
                }
            }
            if changed
                && !closing
                && let Err(err) = self.slate.set_rows(&inspection.rows())
            {
                eprintln!("inspector: rows: {err:?}");
            }
        }
        if closing {
            self.close();
        } else if let Some(target) = next {
            self.open(target, None);
        }
    }
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        Ok(Self {
            tool:       Tool::new(
                "Inspector",
                "Shows what anything is, and nudges where it stands.",
                Some(&glyph::mark()),
            )?,
            own:        script_document()?,
            hand:       None,
            slate:      Slate::new()?,
            inspection: RefCell::new(None),
            ticks:      0,
        })
    }

    fn fixed_update(&mut self, _tick: Tick) -> anyhow::Result<()> {
        api::fixed_update()?;
        while let Some(event) = self.tool.poll() {
            match event {
                ToolEvent::Activate(hand) => self.hand = Some(hand),
                ToolEvent::Deactivate => {
                    self.hand = None;
                    self.close();
                }
                ToolEvent::Pressed(ToolButton::Primary) if self.hand.is_some() => self.press(),
                ToolEvent::Pressed(ToolButton::Secondary) if self.hand.is_some() => self.close(),
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
        // The tether follows what it describes as physics or others move it.
        if let Some(inspection) = &*self.inspection.borrow() {
            self.slate.set_tether(inspection.anchor());
        }
        api::update(tick.dt)?;
        Ok(())
    }
}
