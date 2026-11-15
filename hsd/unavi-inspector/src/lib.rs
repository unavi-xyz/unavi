//! The Inspector: points at anything and opens a slate on it, tethered to
//! the point inspected: the prim, the document it is in, and a card for each
//! of its components (where it is, its mesh, its body), folded to a summary
//! or open to edit.
//!
//! Each slate is a thing in the world, the same in a headset and on desktop:
//! it opens at reading distance beside what it describes, is picked up and
//! carried like anything else, and floats where it is let go. Several can be
//! open at once, one per thing inspected. Moving things is the physgun's;
//! the Inspector reads them, and nudges a value by scrubbing its row.
//!
//! | Act | VR | Desktop |
//! | --- | --- | --- |
//! | see the documents around | hold the Inspector: the Documents lens is on while it is in hand | the same |
//! | inspect | trigger on it | left mouse on it |
//! | inspect it again | trigger on it: its slate comes back to you | the same |
//! | move a slate | squeeze the grip on it and carry it; let go and it floats | right mouse on it, drag |
//! | open or fold a card | trigger on its heading | left mouse on it |
//! | nudge position, turn or scale | trigger on its number, slide across | left mouse on it, drag sideways |
//! | inspect its parent or a child | the rows of the Tree card | the same |
//! | lock, delete, make local | the rows of the slate | the same |
//! | close | the Close row; squeezing the grip on nothing closes the newest | the Close row, or right mouse on nothing |

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
/// Fixed ticks between redraws of the rows, so values others change, or
/// physics moves, show without a press.
const REFRESH_TICKS: u32 = 15;
/// Most slates open at once; opening another closes the oldest.
const MAX_OPEN: usize = 6;
/// Fixed ticks a closed slate is kept while it rolls away.
const ROLL_AWAY_TICKS: u32 = 30;

/// A slate open on one inspected prim.
struct Open {
    inspection: Inspection,
    slate:      Slate,
}

struct Script {
    tool:    Tool,
    own:     Document,
    /// The pointer the tool aims with, while it is in hand.
    hand:    Option<PointerKind>,
    /// Oldest first.
    open:    Vec<Open>,
    /// Closed slates rolling away, and the ticks until they are freed: one
    /// freed while shown would vanish mid-motion.
    closing: Vec<(Slate, u32)>,
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

    /// Inspects what the aim points at. A hit on one of the tool's own slates
    /// is the slate's to answer.
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

    /// Opens a slate on `target`, tethered to `point` on it; a slate already
    /// open on it comes back to the viewer instead.
    fn inspect(&mut self, target: PrimRef, point: Vec3) {
        let own = self.own.id();
        if let Some(open) = self
            .open
            .iter()
            .find(|open| same(&open.inspection.target(), &target))
        {
            stand_beside(&open.slate, point, own);
            return;
        }
        let Some(inspection) = Inspection::of(target, Some(point)) else {
            return;
        };
        let slate = match Slate::new() {
            Ok(slate) => slate,
            Err(err) => {
                eprintln!("inspector: slate: {err:?}");
                return;
            }
        };
        // Rows first, so the slate is measured at the height it will show.
        if let Err(err) = slate.set_rows(&inspection.rows()) {
            eprintln!("inspector: rows: {err:?}");
        }
        stand_beside(&slate, point, own);
        slate.set_tether(inspection.anchor());
        slate.set_shown(true);
        if self.open.len() >= MAX_OPEN {
            self.close(0);
        }
        self.open.push(Open { inspection, slate });
    }

    fn close(&mut self, index: usize) {
        if index >= self.open.len() {
            return;
        }
        let mut closed = self.open.remove(index);
        closed.inspection.finish();
        closed.slate.set_tether(None);
        closed.slate.set_shown(false);
        self.closing.push((closed.slate, ROLL_AWAY_TICKS));
    }

    fn close_all(&mut self) {
        while let Some(newest) = self.open.len().checked_sub(1) {
            self.close(newest);
        }
    }

    /// Answers each slate, and redraws its rows after any change and every
    /// few ticks.
    fn step_slates(&mut self) {
        self.ticks = (self.ticks + 1) % REFRESH_TICKS;
        let refresh = self.ticks == 0;
        let mut closing = Vec::new();
        for (index, open) in self.open.iter_mut().enumerate() {
            let events = open.slate.events();
            let mut next = None;
            for event in &events {
                match open.inspection.apply(event) {
                    Asked::Nothing => {}
                    Asked::Close => closing.push(index),
                    Asked::Inspect(target) => next = Some(target),
                }
            }
            // Moving on from what this slate shows keeps the slate where it
            // floats, and the cards it had open.
            if let Some(target) = next
                && let Some(mut inspection) = Inspection::of(target, None)
            {
                inspection.set_open_cards(open.inspection.open_cards());
                open.inspection.finish();
                open.inspection = inspection;
            }
            if (refresh || !events.is_empty())
                && let Err(err) = open.slate.set_rows(&open.inspection.rows())
            {
                eprintln!("inspector: rows: {err:?}");
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

/// Stands `slate` at reading distance beside `point`, as seen from the eye.
fn stand_beside(slate: &Slate, point: Vec3, own: DocumentId) {
    if let Some(eye) = camera_pose() {
        slate.set_position(stand::stand(eye.translation, point, slate.size(), own));
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
                // The grip squeezed on nothing: put away the newest slate.
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
        self.step_slates();
        Ok(())
    }

    fn update(&mut self, tick: Tick) -> anyhow::Result<()> {
        // Each tether follows what it describes as physics or others move it.
        for open in &self.open {
            open.slate.set_tether(open.inspection.anchor());
        }
        api::update(tick.dt)?;
        Ok(())
    }
}
