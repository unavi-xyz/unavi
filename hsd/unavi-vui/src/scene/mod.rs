//! Draws `unavi-vui` in a `wired:scene` document, and drives it.
//!
//! [`Vui`] owns every surface a script puts up: it finds the viewer, mounts
//! each surface in the room, and on each tick steps attention, grasp, casts,
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
    scene::{
        event::{
            Casting,
            Event,
            FixedUpdate,
            Released,
        },
        grid::Grid,
        mount::Mount,
        orbit::Orbit,
        site::Site,
    },
    surface::Surface,
    tree::Mote,
    tuning::Tuning,
    wired::scene::document::Document,
};

mod bodies;
pub(crate) mod draw;
pub mod event;
mod graphs;
mod grid;
pub mod mount;
mod orbit;
mod placard;
mod site;
mod viewer;

/// Events a surface holds for a consumer that has not asked. Past this the
/// oldest go: an unread queue is a script that stopped listening, not a log.
const EVENT_CAPACITY: usize = 64;

/// A mounted surface, self-contained: it owns its machinery, its prims, its
/// input and its cast site, and the host drives it through this.
pub(crate) trait Mounted {
    fn mount(&self) -> Mount;
    fn place(&mut self, doc: &Document, anchor: &Transform) -> anyhow::Result<()>;
    fn field_lift(&self) -> f32;

    /// The prim everything this surface drew hangs from. Removing it takes
    /// the whole surface — bodies, placard and cast site alike — out of the
    /// scene in one edit.
    fn root(&self) -> (u64, u64);

    /// Puts the surface up or takes it down, keeping its prims either way.
    fn show(&mut self, doc: &Document, shown: bool) -> anyhow::Result<()>;

    /// Whether anything of it is still drawn. A surface sent away keeps being
    /// stepped until it has finished leaving.
    fn is_visible(&self) -> bool;

    /// Steps and draws. Call from the script's `update`, where animation
    /// belongs — pinning it to the fixed rate makes motion step.
    fn update(
        &mut self,
        doc: &Document,
        gaze: &Gaze,
        anchor: Transform,
        delta: f32,
    ) -> anyhow::Result<Vec<Event>>;

    /// Reads input and resolves what it did. Call from the script's
    /// `fixed_update`, where state belongs.
    fn fixed_update(
        &mut self,
        doc: &Document,
        gaze: &Gaze,
        anchor: Transform,
    ) -> anyhow::Result<FixedUpdate>;

    /// Whether a release at `local` — in this surface's own plane — files into
    /// it. Only a grid is a destination; an orbit has no extents to land in.
    fn accepts(&self, _local: Vec2) -> bool {
        false
    }

    /// Takes a released mote in, reporting whether it did. Only a grid is a
    /// destination.
    fn stow(&mut self, _mote: &Mote) -> bool {
        false
    }

    /// Opens `mote` as this surface's level, reporting what that did. Only a
    /// surface holding a tree can navigate one.
    fn open(&mut self, _mote: &Mote) -> Option<Event> {
        None
    }
}

/// A surface [`Vui`] is drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceId(pub(crate) usize);

/// Every VUI surface a script is showing, and the machinery that runs them.
pub struct Vui {
    doc:     Document,
    tuning:  Tuning,
    palette: Palette,
    /// `None` once [`Vui::remove`] has taken the surface's prims out of the
    /// scene; the slot itself stays so every [`SurfaceId`] handed out keeps
    /// naming the same index.
    shapes:  Vec<Option<Box<dyn Mounted>>>,
    anchors: Vec<Option<Transform>>,
    /// Whether each surface is up. A summon clears the anchor beside it, so
    /// the surface re-measures from where the viewer is standing now.
    shown:   Vec<bool>,
    events:  Vec<Vec<Event>>,
}

impl Vui {
    pub fn new(tuning: Tuning, palette: Palette) -> anyhow::Result<Self> {
        Ok(Self {
            doc: crate::wired::scene::document::script_document()?,
            tuning,
            palette,
            shapes: Vec::new(),
            anchors: Vec::new(),
            shown: Vec::new(),
            events: Vec::new(),
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
        let orbit = Orbit::new(&self.doc, root, mount, capacity, &self.tuning, self.palette)?;
        Ok(self.push(Box::new(orbit)))
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
        let grid = Grid::new(&self.doc, root, layout, mount, &self.tuning, self.palette)?;
        Ok(self.push(Box::new(grid)))
    }

    fn push(&mut self, shape: Box<dyn Mounted>) -> SurfaceId {
        self.shapes.push(Some(shape));
        self.anchors.push(None);
        self.shown.push(true);
        self.events.push(Vec::new());
        SurfaceId(self.shapes.len() - 1)
    }

    /// Puts a surface up where the viewer is standing now.
    ///
    /// Re-anchoring rather than taking the surface down and putting a new one
    /// up: every body it draws is already uploaded, and a mesh write costs a
    /// `Flow::BlobUpload` whatever its size.
    pub fn summon(&mut self, surface: SurfaceId) -> anyhow::Result<()> {
        let Some(Some(shape)) = self.shapes.get_mut(surface.0) else {
            return Ok(());
        };
        shape.show(&self.doc, true)?;
        self.anchors[surface.0] = None;
        self.shown[surface.0] = true;
        Ok(())
    }

    /// Takes a surface down, keeping its prims.
    pub fn dismiss(&mut self, surface: SurfaceId) -> anyhow::Result<()> {
        let Some(Some(shape)) = self.shapes.get_mut(surface.0) else {
            return Ok(());
        };
        shape.show(&self.doc, false)?;
        self.shown[surface.0] = false;
        Ok(())
    }

    /// Drops a surface, freeing every prim it drew in one edit. Called when
    /// the script drops its `orbit`/`grid` handle, so a surface nothing
    /// references any longer does not keep costing frames or draw calls.
    pub fn remove(&mut self, surface: SurfaceId) -> anyhow::Result<()> {
        let Some(slot) = self.shapes.get_mut(surface.0) else {
            return Ok(());
        };
        let Some(shape) = slot.take() else {
            return Ok(());
        };
        self.doc.local().remove(shape.root()).flush()?;
        self.anchors[surface.0] = None;
        self.shown[surface.0] = false;
        self.events[surface.0] = Vec::new();
        Ok(())
    }

    #[must_use]
    pub fn is_shown(&self, surface: SurfaceId) -> bool {
        self.shown.get(surface.0).copied().unwrap_or(false)
    }

    /// Everything a surface did since the last call.
    pub fn drain(&mut self, surface: SurfaceId) -> Vec<Event> {
        self.events
            .get_mut(surface.0)
            .map(std::mem::take)
            .unwrap_or_default()
    }

    fn report(&mut self, surface: usize, events: impl IntoIterator<Item = Event>) {
        let Some(queue) = self.events.get_mut(surface) else {
            return;
        };
        queue.extend(events);
        let overrun = queue.len().saturating_sub(EVENT_CAPACITY);
        queue.drain(..overrun);
    }

    /// Reads input and resolves what it did. Call from the script's
    /// `fixed_update`, where state belongs.
    pub fn fixed_update(&mut self) -> anyhow::Result<()> {
        let Some(eye) = crate::camera_pose() else {
            return Ok(());
        };
        let gaze = Gaze::read(&eye);

        for index in 0..self.shapes.len() {
            let Some(shape) = &self.shapes[index] else {
                continue;
            };
            // A surface sent away is stepped until it has finished leaving.
            if !self.shown[index] && !shape.is_visible() {
                continue;
            }
            let anchor = self.anchor(index, &gaze)?;
            let Some(shape) = &mut self.shapes[index] else {
                continue;
            };
            let result = shape.fixed_update(&self.doc, &gaze, anchor)?;
            self.report(index, result.events);
            if let Some(released) = result.released {
                self.place(index, released, result.opens_at, &gaze)?;
            }
        }
        Ok(())
    }

    /// Steps and draws every surface. Call from the script's `update`, where
    /// animation belongs — pinning it to the fixed rate makes motion step.
    pub fn update(&mut self, delta: f32) -> anyhow::Result<()> {
        let Some(eye) = crate::camera_pose() else {
            return Ok(());
        };
        let gaze = Gaze::read(&eye);

        for index in 0..self.shapes.len() {
            let Some(shape) = &self.shapes[index] else {
                continue;
            };
            // A surface sent away is stepped until it has finished leaving.
            if !self.shown[index] && !shape.is_visible() {
                continue;
            }
            let anchor = self.anchor(index, &gaze)?;
            let Some(shape) = &mut self.shapes[index] else {
                continue;
            };
            let events = shape.update(&self.doc, &gaze, anchor, delta)?;
            self.report(index, events);
        }
        Ok(())
    }

    /// Places a surface the first time it is drawn, and reports where it
    /// stands from then on.
    fn anchor(&mut self, index: usize, gaze: &Gaze) -> anyhow::Result<Transform> {
        if let Some(anchor) = self.anchors[index] {
            return Ok(anchor);
        }
        let Some(shape) = &mut self.shapes[index] else {
            return Ok(Transform::IDENTITY);
        };
        let anchor = shape.mount().anchor(&gaze.eye);
        shape.place(&self.doc, &anchor)?;
        self.anchors[index] = Some(anchor);
        Ok(anchor)
    }

    /// Stands a surface where the level it just opened was let go.
    ///
    /// A surface measures its place once and keeps it, so this is the same
    /// move a summon makes: the level opens around the drop rather than back
    /// where the surface happened to be standing.
    fn settle(&mut self, index: usize, at: Vec3, gaze: &Gaze) -> anyhow::Result<()> {
        let Some(shape) = &mut self.shapes[index] else {
            return Ok(());
        };
        let anchor = mount::landed(at, &gaze.eye, &self.tuning);
        shape.place(&self.doc, &anchor)?;
        self.anchors[index] = Some(anchor);
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
            && let Some(shape) = &mut self.shapes[target]
            && shape.stow(&released.mote)
        {
            self.report(index, [Event::Filed(released.mote)]);
            return Ok(());
        }

        if let Some(at) = opens_at
            && let Some(shape) = &mut self.shapes[index]
            && let Some(event) = shape.open(&released.mote)
        {
            self.settle(index, at, gaze)?;
            self.report(index, [event]);
            return Ok(());
        }

        self.report(index, [Event::Planted(released.mote, released.landing)]);
        Ok(())
    }

    /// The grid the pointer is over, which files rather than plants.
    fn filed_into(&self, gaze: &Gaze) -> Option<usize> {
        self.shapes.iter().enumerate().find_map(|(index, shape)| {
            let shape = shape.as_ref()?;
            let anchor = self.shown[index].then(|| self.anchors[index])??;
            pointer::aim(&gaze.ray, &anchor, shape.field_lift())
                .filter(|aim| shape.accepts(aim.local))
                .map(|_| index)
        })
    }
}

/// Opens a cast site on the mote that was tapped, whichever shape shows it.
pub(crate) fn open_cast(casting: &mut Option<Casting>, slot: usize, mote: Mote, surface: &Surface) {
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
pub(crate) fn drive_cast(
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
