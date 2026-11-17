//! The Inspector: points at anything and opens a pane on it, tethered to the
//! point inspected: the prim, the document it is in, a card for each of its
//! components (where it is, its mesh, its body) folded to a summary or open,
//! the prim hierarchy around it, and the layers its values come from.
//!
//! Each pane is a thing in the world, the same in a headset and on desktop:
//! it opens at reading distance beside what it describes, is picked up and
//! carried like anything else, and floats where it is let go. Several can be
//! open at once, one per thing inspected. Moving things is the physgun's;
//! the Inspector reads them, and nudges a value by scrubbing its control.
//!
//! | Act | VR | Desktop |
//! | --- | --- | --- |
//! | see the documents around | hold the Inspector: the Documents lens is on while it is in hand | the same |
//! | inspect | trigger on it | left mouse on it |
//! | inspect it again | trigger on it: its pane comes back to you | the same |
//! | move a pane | squeeze the grip on it and carry it; let go and it floats | right mouse on it, drag |
//! | open or fold a card or a prim | trigger on its fold mark | left mouse on it |
//! | nudge position, turn or scale | trigger on its number, slide across | left mouse on it, drag sideways |
//! | inspect its parent or a child | trigger on its row of the hierarchy | the same |
//! | lock, delete, make local | the controls of the pane | the same |
//! | close | the Close button; squeezing the grip on nothing closes the newest | the Close button, or right mouse on nothing |

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
            Pane,
        },
    },
    wired::{
        core::ids::{
            DocumentId,
            PrimRef,
        },
        event::messaging::{
            self,
            Scope,
        },
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

wired_guest::generate_script!(Script);

/// Furthest anything can be inspected from, in metres.
const REACH: f32 = 50.0;
/// Fixed ticks between redraws of the blocks, so values others change, or
/// physics moves, show without a press.
const REFRESH_TICKS: u32 = 15;
/// Most panes open at once; opening another closes the oldest.
const MAX_OPEN: usize = 6;
/// Fixed ticks a closed pane is kept while it shrinks away.
const ROLL_AWAY_TICKS: u32 = 30;
/// Columns a pane is laid out in.
const COLUMNS: u32 = 40;

/// A pane open on one inspected prim.
struct Open {
    inspection: Inspection,
    pane:       Pane,
}

struct Script {
    tool:    Tool,
    own:     Document,
    /// The pointer the tool aims with, while it is in hand.
    hand:    Option<PointerKind>,
    /// Oldest first.
    open:    Vec<Open>,
    /// Closed panes shrinking away, and the ticks until they are freed: one
    /// freed while shown would vanish mid-motion.
    closing: Vec<(Pane, u32)>,
    ticks:   u32,
}

impl Script {
    fn aim(&self) -> Option<Ray> {
        let aim = self.hand.and_then(pointer_pose).or_else(camera_pose)?;
        Some(Ray {
            origin:    aim.translation,
            direction: aim.forward(),
        })
    }

    /// Inspects what the aim points at. A hit on one of the tool's own panes
    /// is the pane's to answer.
    fn press(&mut self) {
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
            self.inspect(hit.target, hit.point);
        }
    }

    /// Opens a pane on `target`, tethered to `point` on it; a pane already
    /// open on it comes back to the viewer instead.
    fn inspect(&mut self, target: PrimRef, point: Vec3) {
        let own = self.own.id();
        if let Some(open) = self
            .open
            .iter()
            .find(|open| same(&open.inspection.target(), &target))
        {
            stand_beside(&open.pane, point, own);
            return;
        }
        let Some(mut inspection) = Inspection::of(target, Some(point)) else {
            return;
        };
        let pane = match Pane::new(COLUMNS, None) {
            Ok(pane) => pane,
            Err(err) => {
                eprintln!("inspector: pane: {err:?}");
                return;
            }
        };
        // Filled first, so the pane is measured at the height it will show.
        if let Err(err) = inspection.fill(&pane) {
            eprintln!("inspector: blocks: {err:?}");
        }
        stand_beside(&pane, point, own);
        pane.set_tether(inspection.anchor());
        pane.set_shown(true);
        if self.open.len() >= MAX_OPEN {
            self.close(0);
        }
        self.open.push(Open { inspection, pane });
    }

    fn close(&mut self, index: usize) {
        if index >= self.open.len() {
            return;
        }
        let mut closed = self.open.remove(index);
        closed.inspection.finish();
        closed.pane.set_tether(None);
        closed.pane.set_shown(false);
        self.closing.push((closed.pane, ROLL_AWAY_TICKS));
    }

    fn close_all(&mut self) {
        while let Some(newest) = self.open.len().checked_sub(1) {
            self.close(newest);
        }
    }

    /// Answers each pane, and redraws its blocks after any change and every
    /// few ticks.
    fn step_panes(&mut self) {
        self.ticks = (self.ticks + 1) % REFRESH_TICKS;
        let refresh = self.ticks == 0;
        let mut closing = Vec::new();
        for (index, open) in self.open.iter_mut().enumerate() {
            let events = open.pane.events();
            let mut next = None;
            for event in &events {
                match open.inspection.apply(event) {
                    Asked::Nothing => {}
                    Asked::Close => closing.push(index),
                    Asked::Inspect(target) => next = Some(target),
                }
            }
            // Moving on from what this pane shows keeps the pane where it
            // floats, and what it had unfolded.
            if let Some(target) = next
                && let Some(mut inspection) = Inspection::of(target, None)
            {
                inspection.set_folds(open.inspection.folds());
                open.inspection.finish();
                open.inspection = inspection;
            }
            if (refresh || !events.is_empty())
                && let Err(err) = open.inspection.fill(&open.pane)
            {
                eprintln!("inspector: blocks: {err:?}");
            }
        }
        closing.dedup();
        for index in closing.into_iter().rev() {
            self.close(index);
        }
        for (_, left) in &mut self.closing {
            *left = left.saturating_sub(1);
        }
        self.closing.retain(|(_, left)| *left > 0);
    }
}

/// Shows the visor's Documents lens while the Inspector is in hand,
/// leaving the user's own setting for it as it was.
fn hold_documents_lens(held: bool) {
    let request = unavi_lens::Hold {
        lens: unavi_lens::DOCUMENTS.to_owned(),
        held,
    };
    let sent = unavi_lens::encode_hold(&request)
        .map_err(|err| format!("{err}"))
        .and_then(|bytes| {
            messaging::emit(unavi_lens::HOLD, &bytes, None, Scope::Global)
                .map_err(|err| format!("{err:?}"))
        });
    if let Err(err) = sent {
        eprintln!("inspector: hold the documents lens: {err}");
    }
}

fn same(a: &PrimRef, b: &PrimRef) -> bool {
    a.document == b.document && a.prim == b.prim
}

/// Floats `pane` at reading distance beside `point`, as seen from the eye.
fn stand_beside(pane: &Pane, point: Vec3, own: DocumentId) {
    if let Some(eye) = camera_pose() {
        pane.set_floating(stand::stand(eye.translation, point, pane.size(), own));
    }
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        Ok(Self {
            tool:    Tool::new(
                "Inspector",
                "Shows what anything is, and nudges where it stands.",
                Some(&glyph::mark()),
            )?,
            own:     script_document()?,
            hand:    None,
            open:    Vec::new(),
            closing: Vec::new(),
            ticks:   0,
        })
    }

    fn fixed_update(&mut self, _tick: Tick) -> anyhow::Result<()> {
        api::fixed_update()?;
        while let Some(event) = self.tool.poll() {
            match event {
                ToolEvent::Activate(hand) => {
                    self.hand = Some(hand);
                    hold_documents_lens(true);
                }
                ToolEvent::Deactivate => {
                    self.hand = None;
                    hold_documents_lens(false);
                    self.close_all();
                }
                ToolEvent::Pressed(ToolButton::Primary) if self.hand.is_some() => self.press(),
                // The grip squeezed on nothing: put away the newest pane.
                ToolEvent::Pressed(ToolButton::Secondary) if self.hand.is_some() => {
                    if let Some(newest) = self.open.len().checked_sub(1) {
                        self.close(newest);
                    }
                }
                ToolEvent::SetState(_)
                | ToolEvent::Pressed(_)
                | ToolEvent::Released(_)
                | ToolEvent::Scroll(_) => {}
            }
        }
        self.step_panes();
        Ok(())
    }

    fn update(&mut self, tick: Tick) -> anyhow::Result<()> {
        // Each tether follows what it describes as physics or others move it.
        for open in &self.open {
            open.pane.set_tether(open.inspection.anchor());
        }
        api::update(tick.dt)?;
        Ok(())
    }
}
