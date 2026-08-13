//! Draws `unavi-vui` in a `wired:scene` document, and drives it.
//!
//! [`Vui`] owns every surface a script puts up: it finds the viewer, mounts
//! each surface in the room, and on each tick steps attention, grasp, casts,
//! paging and drawing. A consumer supplies motes and reads back [`Event`]s; no
//! prim and no pointer crosses that line.

use std::time::SystemTime;

use wired_prelude::prelude::*;

use crate::{
    circle::{
        Cast,
        Circle,
    },
    palette::Palette,
    pointer,
    scene::{
        cast::Site,
        event::{
            Casting,
            Event,
            FixedUpdate,
            Released,
        },
        grid::Grid,
        mount::Mount,
        orbit::Orbit,
    },
    surface::Surface,
    tree::Mote,
    tuning::Tuning,
    wired::scene::{
        api::self_document,
        types::Document,
    },
};

mod bodies;
mod cast;
pub(crate) mod draw;
pub mod event;
mod grid;
pub mod mount;
mod orbit;
mod placard;
mod viewer;

/// Events a surface holds for a consumer that has not asked. Past this the
/// oldest go: an unread queue is a script that stopped listening, not a log.
const EVENT_CAPACITY: usize = 64;

/// A mounted surface, self-contained: it owns its machinery, its prims, its
/// input and its cast site, and the host drives it through this.
pub(crate) trait Mounted {
    fn mount(&self) -> Mount;
    fn place(&mut self, anchor: &Transform) -> anyhow::Result<()>;
    fn field_lift(&self) -> f32;

    /// Puts the surface up or takes it down, keeping its prims either way.
    fn show(&mut self, shown: bool) -> anyhow::Result<()>;

    /// Whether anything of it is still drawn. A surface sent away keeps being
    /// stepped until it has finished leaving.
    fn is_visible(&self) -> bool;

    /// Steps and draws. Call from the script's `update`, where animation
    /// belongs — pinning it to the fixed rate makes motion step.
    fn update(
        &mut self,
        eye: &Transform,
        anchor: Transform,
        delta: f32,
    ) -> anyhow::Result<Vec<Event>>;

    /// Reads input and resolves what it did. Call from the script's
    /// `fixed_update`, where state belongs.
    fn fixed_update(&mut self, eye: &Transform, anchor: Transform) -> anyhow::Result<FixedUpdate>;

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
}

/// A surface [`Vui`] is drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceId(pub(crate) usize);

/// Every VUI surface a script is showing, and the machinery that runs them.
pub struct Vui {
    doc:      Document,
    viewer:   viewer::Viewer,
    tuning:   Tuning,
    palette:  Palette,
    shapes:   Vec<Box<dyn Mounted>>,
    anchors:  Vec<Option<Transform>>,
    /// Whether each surface is up. A summon clears the anchor beside it, so
    /// the surface re-measures from where the viewer is standing now.
    shown:    Vec<bool>,
    events:   Vec<Vec<Event>>,
    drawn_at: SystemTime,
}

impl Vui {
    pub fn new(tuning: Tuning, palette: Palette) -> anyhow::Result<Self> {
        Ok(Self {
            doc: self_document()?,
            viewer: viewer::Viewer::new(),
            tuning,
            palette,
            shapes: Vec::new(),
            anchors: Vec::new(),
            shown: Vec::new(),
            events: Vec::new(),
            drawn_at: SystemTime::now(),
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
        let grid = Grid::new(
            &self.doc,
            root,
            columns,
            rows,
            Vec2::splat(self.tuning.grid_pitch),
            mount,
            &self.tuning,
            self.palette,
        )?;
        Ok(self.push(Box::new(grid)))
    }

    fn push(&mut self, shape: Box<dyn Mounted>) -> SurfaceId {
        self.shapes.push(shape);
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
        let Some(shape) = self.shapes.get_mut(surface.0) else {
            return Ok(());
        };
        shape.show(true)?;
        self.anchors[surface.0] = None;
        self.shown[surface.0] = true;
        Ok(())
    }

    /// Takes a surface down, keeping its prims.
    pub fn dismiss(&mut self, surface: SurfaceId) -> anyhow::Result<()> {
        let Some(shape) = self.shapes.get_mut(surface.0) else {
            return Ok(());
        };
        shape.show(false)?;
        self.shown[surface.0] = false;
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
        let Some(eye) = self.viewer.pose() else {
            return Ok(());
        };

        for index in 0..self.shapes.len() {
            // A surface sent away is stepped until it has finished leaving.
            if !self.shown[index] && !self.shapes[index].is_visible() {
                continue;
            }
            let anchor = self.anchor(index, &eye)?;
            let result = self.shapes[index].fixed_update(&eye, anchor)?;
            self.report(index, result.events);
            if let Some(released) = result.released {
                self.place(index, released, &eye);
            }
        }
        Ok(())
    }

    /// Steps and draws every surface. Call from the script's `update`, where
    /// animation belongs — pinning it to the fixed rate makes motion step.
    pub fn update(&mut self) -> anyhow::Result<()> {
        let delta = self.drawn_at.elapsed().unwrap_or_default().as_secs_f32();
        self.drawn_at = SystemTime::now();

        let Some(eye) = self.viewer.pose() else {
            return Ok(());
        };

        for index in 0..self.shapes.len() {
            // A surface sent away is stepped until it has finished leaving.
            if !self.shown[index] && !self.shapes[index].is_visible() {
                continue;
            }
            let anchor = self.anchor(index, &eye)?;
            let events = self.shapes[index].update(&eye, anchor, delta)?;
            self.report(index, events);
        }
        Ok(())
    }

    /// Places a surface the first time it is drawn, and reports where it
    /// stands from then on.
    fn anchor(&mut self, index: usize, eye: &Transform) -> anyhow::Result<Transform> {
        if let Some(anchor) = self.anchors[index] {
            return Ok(anchor);
        }
        let anchor = self.shapes[index].mount().anchor(eye);
        self.shapes[index].place(&anchor)?;
        self.anchors[index] = Some(anchor);
        Ok(anchor)
    }

    /// A carried mote lands in the grid it was released over, or in the room.
    ///
    /// Nothing of the consumer's is drawn either way: the mote goes back where
    /// it came from, and a landing says where it was let go so the consumer
    /// can put its own thing there.
    fn place(&mut self, index: usize, released: Released, eye: &Transform) {
        let event = match self.filed_into(eye) {
            Some(target) if self.shapes[target].stow(&released.mote) => Event::Filed(released.mote),
            _ => Event::Planted(released.mote, released.landing),
        };
        self.report(index, [event]);
    }

    /// The grid the pointer is over, which files rather than plants.
    fn filed_into(&self, eye: &Transform) -> Option<usize> {
        self.shapes.iter().enumerate().find_map(|(index, shape)| {
            let anchor = self.shown[index].then(|| self.anchors[index])??;
            pointer::aim(eye, &anchor, shape.field_lift())
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
        circle: Circle::standard(surface.tuning()),
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
    casting: &mut Option<Casting>,
    surface: &Surface,
    site: &Site,
    delta: f32,
    events: &mut Vec<Event>,
) -> anyhow::Result<()> {
    let Some(active) = casting else {
        return Ok(());
    };

    let held = surface.seized_slot() == Some(active.slot);
    let cast = active.circle.update(held, delta);
    let at = surface
        .views()
        .get(active.slot)
        .map_or(Vec3::ZERO, |view| view.position);
    site.apply(at, cast)?;

    if !cast.is_settled() {
        return Ok(());
    }
    let mote = active.mote.clone();
    *casting = None;
    events.push(match cast {
        Cast::Committed => Event::Cast(mote),
        Cast::Filling(_) | Cast::Aborted => Event::Aborted(mote),
    });
    Ok(())
}
