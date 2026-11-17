//! Draws `unavi-vui` in a `wired:scene` document, and drives it.
//!
//! [`Vui`] owns every panel a script puts up: it finds the viewer, mounts
//! each panel in the room, and on each tick steps attention, grasp, casts,
//! paging and drawing. A consumer supplies motes and reads back [`Event`]s; no
//! prim and no pointer crosses that line.

use wired_guest::{
    math::{
        Transform,
        Vec2,
    },
    teleport::Watch,
};

use crate::{
    component::{
        event::Event,
        pane::panes::Panes,
        panel::Panel,
        placard::Content,
        station::{
            dock,
            stations::Stations,
            tiling::Cells,
        },
        tree::Mote,
    },
    machinery::{
        layout::Layout,
        mount::Mount,
        pointer::Gaze,
    },
    tokens::{
        palette::Palette,
        tuning::Tuning,
    },
    wired::scene::document::Document,
};

mod pane;
mod place;
mod station;

/// The local screen, where a station may lay its panes against it.
fn screen() -> Option<dock::Screen> {
    crate::viewport().map(|screen| dock::Screen {
        pixels: Cells {
            w: screen.width,
            h: screen.height,
        },
        fov: screen.fov,
    })
}

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
    /// Framed pages of blocks, standing or floating where a script put them.
    panes:   Panes,
    /// Panes arranged together.
    stations: Stations,
    /// Seconds of updates so far, which notices are timed against.
    now:     f64,
    /// The previous fixed tick's eye, which a respawn is told apart by.
    jumps:   Watch,
}

impl Vui {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            doc:     crate::wired::scene::document::script_document()?,
            tuning:  Tuning::DEFAULT,
            palette: Palette::DEFAULT,
            slots:   Vec::new(),
            free:    Vec::new(),
            panes:   Panes::default(),
            stations: Stations::default(),
            now:     0.0,
            jumps:   Watch::default(),
        })
    }

    /// Seconds of updates so far.
    #[must_use]
    pub const fn now(&self) -> f64 {
        self.now
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
        let aborted = slot.panel.summon(&self.doc);
        slot.anchor = None;
        slot.shown = true;
        let aborted = aborted?;
        self.report(surface.0, aborted);
        Ok(())
    }

    /// Sets the readout an orbit shows at its centre, or takes it down.
    pub fn set_readout(&mut self, surface: SurfaceId, text: Option<&str>) {
        if let Some(slot) = self.slot_mut(surface) {
            slot.panel.set_readout(text);
        }
    }

    /// Sets what a bed's page says at the root while nothing is attended or
    /// kept, or takes it down.
    pub fn set_resting(&mut self, surface: SurfaceId, content: Option<Content>) {
        if let Some(slot) = self.slot_mut(surface) {
            slot.panel.set_resting(content);
        }
    }

    /// Takes a panel down, keeping its prims.
    pub fn dismiss(&mut self, surface: SurfaceId) -> anyhow::Result<()> {
        let Some(Some(slot)) = self.slots.get_mut(surface.0) else {
            return Ok(());
        };
        // Down whatever the writes did, or a failed one leaves the panel
        // stepped and shown with the summon already spent.
        let aborted = slot.panel.dismiss(&self.doc);
        slot.shown = false;
        let aborted = aborted?;
        self.report(surface.0, aborted);
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

    #[must_use]
    pub fn is_held(&self, surface: SurfaceId) -> bool {
        self.slot(surface).is_some_and(|slot| slot.panel.is_held())
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
        if let Some(by) = self.jumps.step(&eye) {
            self.carry(&by)?;
        }
        let gaze = Gaze::read(&eye);
        self.panes.fixed_update(&self.doc, &self.tuning);
        self.stations
            .step(&mut self.panes, &eye, &self.tuning, screen());

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
        self.now += f64::from(delta);
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
            let events = slot
                .panel
                .update(&self.doc, &gaze, anchor, delta, self.now)?;
            self.report(index, events);
        }
        self.panes
            .update(&self.doc, &gaze.eye, &self.palette, &self.tuning, delta)?;
        Ok(())
    }
}
