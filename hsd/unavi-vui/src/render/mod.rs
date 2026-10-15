//! Draws `unavi-vui` in a `wired:scene` document, and drives it.
//!
//! [`Vui`] owns every panel a script puts up: it finds the viewer, mounts
//! each panel in the room, and on each tick steps attention, grasp, casts,
//! paging and drawing. A consumer supplies motes and reads back [`Event`]s; no
//! prim and no pointer crosses that line.

use wired_guest::math::{
    Transform,
    Vec2,
    Vec3,
};

use crate::{
    cast::{
        Cast,
        State,
    },
    layout::Layout,
    palette::Palette,
    pointer::{
        self,
        Gaze,
    },
    render::{
        event::{
            Casting,
            Event,
            Released,
        },
        mount::Mount,
        panel::Panel,
        site::Site,
    },
    surface::Surface,
    tree::Mote,
    tuning::Tuning,
    wired::scene::document::Document,
};

mod bodies;
pub mod draw;
pub mod event;
mod graphs;
pub mod mount;
mod panel;
mod placard;
pub mod shadow;
mod site;
mod viewer;

/// Events a panel holds for a consumer that has not asked. Past this the
/// oldest go: an unread queue is a script that stopped listening, not a log.
const EVENT_CAPACITY: usize = 64;

/// A panel [`Vui`] is drawing, and the little it tracks on its behalf.
struct Slot {
    panel:  Panel,
    anchor: Option<Transform>,
    /// Whether it is up. A summon clears this, so the panel re-measures from
    /// where the viewer is standing now.
    shown:  bool,
    events: Vec<Event>,
}

/// A surface [`Vui`] is drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceId(usize);

/// Every VUI panel a script is showing, and the machinery that runs them.
///
/// `slots` is a free list: [`Vui::remove`] takes a panel's prims out of the
/// scene and frees its index for the next `orbit`/`grid`, rather than
/// growing forever across a script's whole life.
pub struct Vui {
    doc:     Document,
    tuning:  Tuning,
    palette: Palette,
    slots:   Vec<Option<Slot>>,
    free:    Vec<usize>,
}

impl Vui {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            doc:     crate::wired::scene::document::script_document()?,
            tuning:  Tuning::DEFAULT,
            palette: Palette::DEFAULT,
            slots:   Vec::new(),
            free:    Vec::new(),
        })
    }

    /// Puts up an orbit over `root`, drawing `capacity` motes at once and
    /// paging anything past that.
    pub fn orbit(
        &mut self,
        root: Mote,
        mount: Mount,
        capacity: usize,
    ) -> anyhow::Result<SurfaceId> {
        let panel = Panel::orbit(&self.doc, root, mount, capacity, &self.tuning, self.palette)?;
        Ok(self.push(panel))
    }

    /// Puts up a grid over `root`: somewhere carried motes can be filed, and
    /// a destination a release over it lands in.
    pub fn grid(
        &mut self,
        root: Mote,
        columns: usize,
        rows: usize,
        mount: Mount,
    ) -> anyhow::Result<SurfaceId> {
        let layout = Layout::grid(columns, rows, Vec2::splat(self.tuning.grid_pitch));
        let panel = Panel::grid(&self.doc, root, layout, mount, &self.tuning, self.palette)?;
        Ok(self.push(panel))
    }

    /// Takes the next free slot, or grows the free list by one.
    fn push(&mut self, panel: Panel) -> SurfaceId {
        let slot = Slot {
            panel,
            anchor: None,
            shown: true,
            events: Vec::new(),
        };
        if let Some(index) = self.free.pop() {
            self.slots[index] = Some(slot);
            return SurfaceId(index);
        }
        self.slots.push(Some(slot));
        SurfaceId(self.slots.len() - 1)
    }

    /// Puts a panel up where the viewer is standing now.
    ///
    /// Re-anchoring rather than taking the panel down and putting a new one
    /// up: every body it draws is already uploaded, and a mesh write costs a
    /// `Flow::BlobUpload` whatever its size.
    pub fn summon(&mut self, surface: SurfaceId) -> anyhow::Result<()> {
        let Some(Some(slot)) = self.slots.get_mut(surface.0) else {
            return Ok(());
        };
        slot.panel.show(&self.doc, true)?;
        slot.anchor = None;
        slot.shown = true;
        Ok(())
    }

    /// Takes a panel down, keeping its prims.
    pub fn dismiss(&mut self, surface: SurfaceId) -> anyhow::Result<()> {
        let Some(Some(slot)) = self.slots.get_mut(surface.0) else {
            return Ok(());
        };
        slot.panel.show(&self.doc, false)?;
        slot.shown = false;
        Ok(())
    }

    /// Drops a panel, freeing every prim it drew in one edit and returning
    /// its slot to the free list. Called when the script drops its
    /// `orbit`/`grid` handle, so a panel nothing references any longer does
    /// not keep costing frames, draw calls, or a slot in [`Vui::slots`].
    pub fn remove(&mut self, surface: SurfaceId) -> anyhow::Result<()> {
        let Some(entry) = self.slots.get_mut(surface.0) else {
            return Ok(());
        };
        let Some(slot) = entry.take() else {
            return Ok(());
        };
        self.doc.local().remove(slot.panel.root()).flush()?;
        self.free.push(surface.0);
        Ok(())
    }

    #[must_use]
    pub fn is_shown(&self, surface: SurfaceId) -> bool {
        self.slot(surface).is_some_and(|slot| slot.shown)
    }

    /// Everything a panel did since the last call.
    pub fn drain(&mut self, surface: SurfaceId) -> Vec<Event> {
        self.slot_mut(surface)
            .map(|slot| std::mem::take(&mut slot.events))
            .unwrap_or_default()
    }

    fn report(&mut self, index: usize, events: impl IntoIterator<Item = Event>) {
        let Some(Some(slot)) = self.slots.get_mut(index) else {
            return;
        };
        slot.events.extend(events);
        let overrun = slot.events.len().saturating_sub(EVENT_CAPACITY);
        slot.events.drain(..overrun);
    }

    fn slot(&self, surface: SurfaceId) -> Option<&Slot> {
        self.slots.get(surface.0)?.as_ref()
    }

    fn slot_mut(&mut self, surface: SurfaceId) -> Option<&mut Slot> {
        self.slots.get_mut(surface.0)?.as_mut()
    }

    /// Reads input and resolves what it did. Call from the script's
    /// `fixed_update`, where state belongs.
    pub fn fixed_update(&mut self) -> anyhow::Result<()> {
        let Some(eye) = crate::camera_pose() else {
            return Ok(());
        };
        let gaze = Gaze::read(&eye);

        for index in 0..self.slots.len() {
            let Some(slot) = &self.slots[index] else {
                continue;
            };
            // A panel sent away is stepped until it has finished leaving.
            if !slot.shown && !slot.panel.is_visible() {
                continue;
            }
            let anchor = self.anchor(index, &gaze)?;
            let Some(slot) = &mut self.slots[index] else {
                continue;
            };
            let result = slot.panel.fixed_update(&self.doc, &gaze, anchor)?;
            self.report(index, result.events);
            if let Some(released) = result.released {
                self.place(index, released, result.opens_at, &gaze)?;
            }
        }
        Ok(())
    }

    /// Steps and draws every panel. Call from the script's `update`, where
    /// animation belongs — pinning it to the fixed rate makes motion step.
    pub fn update(&mut self, delta: f32) -> anyhow::Result<()> {
        let Some(eye) = crate::camera_pose() else {
            return Ok(());
        };
        let gaze = Gaze::read(&eye);

        for index in 0..self.slots.len() {
            let Some(slot) = &self.slots[index] else {
                continue;
            };
            // A panel sent away is stepped until it has finished leaving.
            if !slot.shown && !slot.panel.is_visible() {
                continue;
            }
            let anchor = self.anchor(index, &gaze)?;
            let Some(slot) = &mut self.slots[index] else {
                continue;
            };
            let events = slot.panel.update(&self.doc, &gaze, anchor, delta)?;
            self.report(index, events);
        }
        Ok(())
    }

    /// Places a panel the first time it is drawn, and reports where it
    /// stands from then on.
    fn anchor(&mut self, index: usize, gaze: &Gaze) -> anyhow::Result<Transform> {
        let Some(Some(slot)) = self.slots.get(index) else {
            return Ok(Transform::IDENTITY);
        };
        if let Some(anchor) = slot.anchor {
            return Ok(anchor);
        }
        let anchor = slot.panel.mount().anchor(&gaze.eye);
        let Some(Some(slot)) = self.slots.get_mut(index) else {
            return Ok(anchor);
        };
        slot.panel.place(&self.doc, &anchor)?;
        slot.anchor = Some(anchor);
        Ok(anchor)
    }

    /// Stands a panel where the level it just opened was let go.
    ///
    /// A panel measures its place once and keeps it, so this is the same
    /// move a summon makes: the level opens around the drop rather than back
    /// where the panel happened to be standing.
    fn settle(&mut self, index: usize, at: Vec3, gaze: &Gaze) -> anyhow::Result<()> {
        let Some(Some(slot)) = self.slots.get_mut(index) else {
            return Ok(());
        };
        let anchor = mount::landed(at, &gaze.eye, &self.tuning);
        slot.panel.place(&self.doc, &anchor)?;
        slot.anchor = Some(anchor);
        Ok(())
    }

    /// Answers a landing, in the order a release means things: a grid under
    /// it takes it, else a level opens there, else the room has it.
    ///
    /// Nothing of the consumer's is drawn in that last case: the mote goes
    /// back where it came from, and a landing says where it was let go so the
    /// consumer can put its own thing there.
    fn place(
        &mut self,
        index: usize,
        released: Released,
        opens_at: Option<Vec3>,
        gaze: &Gaze,
    ) -> anyhow::Result<()> {
        if let Some(target) = self.filed_into(gaze)
            && let Some(Some(slot)) = self.slots.get_mut(target)
            && slot.panel.stow(&released.mote)
        {
            self.report(index, [Event::Filed(released.mote)]);
            return Ok(());
        }

        if let Some(at) = opens_at
            && let Some(Some(slot)) = self.slots.get_mut(index)
            && let Some(event) = slot.panel.open(&released.mote)
        {
            self.settle(index, at, gaze)?;
            self.report(index, [event]);
            return Ok(());
        }

        self.report(index, [Event::Planted(released.mote, released.landing)]);
        Ok(())
    }

    /// The panel the pointer is over, which files rather than plants.
    fn filed_into(&self, gaze: &Gaze) -> Option<usize> {
        self.slots.iter().enumerate().find_map(|(index, slot)| {
            let slot = slot.as_ref()?;
            let anchor = slot.shown.then_some(slot.anchor)??;
            pointer::aim(&gaze.ray, &anchor, slot.panel.field_lift())
                .filter(|aim| slot.panel.accepts(aim.local))
                .map(|_| index)
        })
    }
}

/// Opens a cast site on the mote that was tapped, whichever panel shows it.
pub fn open_cast(casting: &mut Option<Casting>, slot: usize, mote: Mote, surface: &Surface) {
    *casting = Some(Casting {
        slot,
        mote,
        cast: Cast::standard(surface.tuning()),
    });
}

/// Fills the open cast site while the grasp stays down on the mote that opened
/// it, and aborts the moment it lets go.
///
/// The hold is the confirmation, so it is the grasp that fills the ring rather
/// than attention. Filling on attention made a cast fire from a single click:
/// the pointer is still on the mote it just pressed, so the ring filled with
/// no further input and the hold was decorative.
pub fn drive_cast(
    doc: &Document,
    casting: &mut Option<Casting>,
    surface: &Surface,
    site: &Site,
    delta: f32,
    events: &mut Vec<Event>,
) -> anyhow::Result<()> {
    // Before the guard: an abandoned ring is unwinding precisely when there is
    // no cast left to drive.
    site.step(doc, delta)?;

    let Some(active) = casting else {
        return Ok(());
    };

    let held = surface.seized_slot() == Some(active.slot);
    let state = active.cast.update(held, delta);
    let at = surface
        .views()
        .get(active.slot)
        .map_or(Vec3::ZERO, |view| view.position);
    site.apply(doc, at, state)?;

    if !state.is_settled() {
        return Ok(());
    }
    let mote = active.mote.clone();
    *casting = None;
    events.push(match state {
        State::Committed => Event::Cast(mote),
        State::Filling(_) | State::Aborted => Event::Aborted(mote),
    });
    Ok(())
}
