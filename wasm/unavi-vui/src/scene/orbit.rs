//! A self-contained navigable orbit: owns its motes, its machinery, its prims,
//! its input and its cast site.

use wired_math::types::{
    Vec2,
    Vec3,
};
use wired_prelude::prelude::*;

use crate::{
    grasp::Outcome,
    layout::{
        Centre,
        Layout,
    },
    mote::Arrange,
    palette::Palette,
    pointer,
    scene::{
        Mounted,
        bodies::{
            Bodies,
            Hit,
            Signal,
        },
        drive_cast,
        event::{
            Casting,
            Event,
            FixedUpdate,
            Landing,
            Released,
        },
        mount::Mount,
        open_cast,
        site::Site,
    },
    surface::Surface,
    tree::{
        Kind,
        Mote,
        Navigation,
        Tree,
    },
    tuning::Tuning,
    view::Frame,
    wired::scene::types::Document,
};

/// A level of motes arranged around an anchor, selected by direction.
pub struct Orbit {
    tree:    Tree,
    surface: Surface,
    bodies:  Bodies,
    site:    Site,
    casting: Option<Casting>,
    mount:   Mount,
    /// The pressed mote's depth along the view ray, standing in for a tracked
    /// hand on desktop.
    depth:   Option<f32>,
    /// The slot the engine's grab is carrying. Its transform belongs to the
    /// solver until it comes back.
    held:    Option<usize>,
    /// Last page reported, so a turn is announced once rather than every
    /// frame.
    paged:   Option<usize>,
}

impl Orbit {
    pub fn new(
        doc: &Document,
        root: Mote,
        mount: Mount,
        capacity: usize,
        tuning: &Tuning,
        palette: Palette,
    ) -> anyhow::Result<Self> {
        let surface = Surface::new(capacity, *tuning, palette);
        let reach = tuning.orbit_radius * tuning.reach_frac;
        let bodies = Bodies::new(doc, capacity, tuning, Hit::Disc { radius: reach })?;
        let site = Site::new(doc, bodies.root(), palette)?;

        Ok(Self {
            tree: Tree::new(root),
            surface,
            bodies,
            site,
            casting: None,
            mount,
            depth: None,
            held: None,
            paged: None,
        })
    }
}

impl Mounted for Orbit {
    fn mount(&self) -> Mount {
        self.mount
    }

    fn place(&mut self, anchor: &Transform) -> anyhow::Result<()> {
        self.bodies.place(anchor)
    }

    fn field_lift(&self) -> f32 {
        self.surface.tuning().field_lift
    }

    fn show(&mut self, shown: bool) -> anyhow::Result<()> {
        self.surface.set_open(shown);
        self.bodies.show(shown)
    }

    fn is_visible(&self) -> bool {
        self.surface.is_visible()
    }

    fn update(
        &mut self,
        eye: &Transform,
        anchor: Transform,
        delta: f32,
    ) -> anyhow::Result<Vec<Event>> {
        let hand = self.depth.map(|depth| pointer::hand(eye, depth));
        let frame = Frame {
            eye: eye.translation,
            anchor,
            aim: pointer::aim(eye, &anchor, self.surface.tuning().field_lift),
            hand,
            delta,
        };

        let motes = self.tree.level_motes();
        let specs = self.tree.level();
        let tuning = self.surface.tuning();
        let (layout, pinned) = match self.tree.arrange() {
            Arrange::Orbit => {
                let centre = if self.tree.is_nested() {
                    Centre::Held
                } else {
                    Centre::Open
                };
                Layout::orbit(
                    specs.len(),
                    centre,
                    self.surface.capacity(),
                    tuning.orbit_radius,
                )
            }
            Arrange::Grid => (
                Layout::grid(
                    tuning.grid_columns,
                    tuning.grid_rows,
                    Vec2::splat(tuning.grid_pitch),
                ),
                usize::from(self.tree.is_nested()),
            ),
        };
        self.surface.update(&specs, layout, pinned, &frame);

        self.bodies
            .icons(&motes, self.surface.views(), self.surface.drawn())?;
        self.bodies.apply(
            self.surface.views(),
            &specs,
            self.surface.drawn(),
            self.surface.placard(),
            self.surface.palette(),
            self.held,
        )?;
        let mut events = Vec::new();
        self.report_page(&mut events);
        drive_cast(
            &mut self.casting,
            &self.surface,
            &self.site,
            delta,
            &mut events,
        )?;
        self.hand_over()?;
        Ok(events)
    }

    fn fixed_update(&mut self, eye: &Transform, anchor: Transform) -> anyhow::Result<FixedUpdate> {
        let mut events = Vec::new();
        let mut released = None;
        for signal in self.bodies.poll() {
            match (signal, self.surface.is_seized()) {
                (Signal::Grab(true), false) => self.press(eye, anchor, &mut events),
                (Signal::Grab(false), _) => released = self.release(&mut events)?,
                (Signal::Turn(delta), false) => self.surface.turn_by(delta),
                (Signal::Grab(true) | Signal::Turn(_), true) => {}
            }
        }
        Ok(FixedUpdate { events, released })
    }
}

impl Orbit {
    /// Grabs the lit mote at its drawn depth, so it arrives under the pointer.
    ///
    /// A consequential mote opens its cast site here rather than on release:
    /// the hold *is* the confirmation, so the ring starts filling the moment
    /// the mote is taken hold of and letting go early abandons it.
    fn press(&mut self, eye: &Transform, anchor: Transform, events: &mut Vec<Event>) {
        let Some(slot) = self.surface.attended() else {
            return;
        };
        let Some(view) = self.surface.views().get(slot) else {
            return;
        };
        let world = anchor.translation + anchor.rotation * view.position;
        let depth = (world - eye.translation).length();
        self.depth = Some(depth);
        self.surface.press(pointer::hand(eye, depth));

        if let Some(mote) = self.consequential(slot) {
            open_cast(&mut self.casting, slot, mote.clone(), &self.surface);
            events.push(Event::Casting(mote));
        }
    }

    /// The mote drawn in `slot`, if holding it is what fires it.
    fn consequential(&mut self, slot: usize) -> Option<Mote> {
        let index = self.surface.spec_index(slot)?;
        self.tree
            .at_level(index)
            .filter(|mote| mote.kind() == Kind::Cast)
    }

    /// Returns the carried mote to its surface, reporting what a tap did and
    /// handing the host anything that was placed.
    fn release(&mut self, events: &mut Vec<Event>) -> anyhow::Result<Option<Released>> {
        self.depth = None;
        let outcome = self.surface.release();

        let carried = self.held.take();
        let landed = carried.and_then(|slot| {
            let bodies = &self.bodies;
            Some((bodies.pose(slot)?.translation, bodies.velocity(slot)))
        });
        if let Some(slot) = carried {
            self.bodies.clear_dynamic(slot)?;
        }

        let mut released = None;
        match outcome {
            Some(Outcome::Tap(slot)) => {
                if let Some(event) = self.select(slot) {
                    events.push(event);
                }
            }
            Some(Outcome::Place(slot)) => {
                if let Some((at, velocity)) = landed {
                    released = self.build_released(slot, at, velocity);
                }
            }
            None => {}
        }
        Ok(released)
    }

    /// Selects whatever holds attention in `slot`, navigating the tree.
    ///
    /// A consequential mote is not selected on release: its site opened when
    /// it was pressed, and by the time the grasp lets go the cast has either
    /// fired or been abandoned.
    fn select(&mut self, slot: usize) -> Option<Event> {
        let index = self.surface.spec_index(slot)?;
        match self.tree.select(index) {
            Navigation::Bloomed(mote) => Some(Event::Opened(mote)),
            Navigation::Collapsed(mote) => Some(Event::Closed(mote)),
            Navigation::Activated(mote) => Some(Event::Activated(mote)),
            Navigation::Cast(_) | Navigation::None => None,
        }
    }

    /// Says how much is off the page, because an orbit that quietly holds three
    /// of eighteen apples is an orbit that is lying.
    fn report_page(&mut self, events: &mut Vec<Event>) {
        let page = self.surface.page();
        if !page.is_paged() {
            self.paged = None;
            return;
        }
        if self.paged == Some(page.index) {
            return;
        }
        self.paged = Some(page.index);
        events.push(Event::Paged {
            index: page.index,
            count: page.count,
            total: page.total,
        });
    }

    /// Hands a mote that has left its slot to the engine's grab, which owns
    /// carrying from there.
    fn hand_over(&mut self) -> anyhow::Result<()> {
        if self.held.is_some() {
            return Ok(());
        }
        let Some(slot) = self.surface.displaced() else {
            return Ok(());
        };
        let Some(radius) = self.surface.views().get(slot).map(|view| view.radius) else {
            return Ok(());
        };
        // Tracked before the promotion, so a mote that only half made it is
        // still stripped back on release.
        self.held = Some(slot);
        self.bodies.make_dynamic(slot, radius)
    }

    fn build_released(&mut self, slot: usize, at: Vec3, velocity: Vec3) -> Option<Released> {
        let index = self.surface.spec_index(slot)?;
        Some(Released {
            mote:    self.tree.at_level(index)?,
            landing: Landing { at, velocity },
        })
    }
}
