//! A self-contained grid: a destination that accepts filings, with real
//! extents and pagination past its cells.

use wired_math::types::{
    Vec2,
    Vec3,
};
use wired_prelude::prelude::*;

use crate::{
    grasp::Outcome,
    layout::Layout,
    palette::Palette,
    pointer,
    scene::{
        Mounted,
        bodies::{
            Bodies,
            Hit,
            Signal,
        },
        cast::Site,
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
    },
    surface::Surface,
    tree::{
        Kind,
        Mote,
    },
    tuning::Tuning,
    view::Frame,
    wired::scene::types::Document,
};

/// A bounded grid of a mote's children, and a destination a carried mote can
/// be filed into. A row is `rows: 1`, a column `columns: 1`.
pub struct Grid {
    root:    Mote,
    surface: Surface,
    bodies:  Bodies,
    site:    Site,
    columns: usize,
    rows:    usize,
    pitch:   Vec2,
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

impl Grid {
    pub fn new(
        doc: &Document,
        root: Mote,
        columns: usize,
        rows: usize,
        pitch: Vec2,
        mount: Mount,
        tuning: &Tuning,
        palette: Palette,
    ) -> anyhow::Result<Self> {
        let capacity = columns.saturating_mul(rows);
        let surface = Surface::new(capacity, *tuning, palette);
        let extents = Layout::grid(columns, rows, pitch).extents(tuning);
        let bodies = Bodies::new(doc, capacity, tuning, Hit::Slab { extents })?;
        let site = Site::new(doc, bodies.root(), palette)?;

        Ok(Self {
            root,
            surface,
            bodies,
            site,
            columns,
            rows,
            pitch,
            casting: None,
            mount,
            depth: None,
            held: None,
            paged: None,
        })
    }
}

impl Mounted for Grid {
    fn mount(&self) -> Mount {
        self.mount
    }

    fn place(&mut self, anchor: &Transform) -> anyhow::Result<()> {
        self.bodies.place(anchor)
    }

    fn field_lift(&self) -> f32 {
        self.surface.tuning().field_lift
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

        let motes = self.contents();
        let specs = motes.iter().map(Mote::spec).collect::<Vec<_>>();
        let layout = Layout::grid(self.columns, self.rows, self.pitch);
        self.surface.update(&specs, layout, 0, &frame);

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
                (Signal::Grab(true), false) => self.press(eye, anchor),
                (Signal::Grab(false), _) => released = self.release(&mut events)?,
                (Signal::Turn(delta), false) => self.surface.turn_by(delta),
                (Signal::Grab(true) | Signal::Turn(_), true) => {}
            }
        }
        Ok(FixedUpdate { events, released })
    }

    /// A grid is a destination: a release over its housing files into it.
    fn accepts(&self, local: Vec2) -> bool {
        Layout::grid(self.columns, self.rows, self.pitch).accepts(local, self.surface.tuning())
    }

    fn stow(&mut self, mote: &Mote) -> bool {
        self.root.add_child(mote)
    }
}

impl Grid {
    fn contents(&self) -> Vec<Mote> {
        self.root.children()
    }

    fn press(&mut self, eye: &Transform, anchor: Transform) {
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
    }

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

        match outcome {
            Some(Outcome::Tap(slot)) => {
                if let Some(event) = self.select(slot) {
                    events.push(event);
                }
            }
            Some(Outcome::Place(slot)) => {
                if let Some((at, velocity)) = landed {
                    return Ok(self.build_released(slot, at, velocity));
                }
            }
            None => {}
        }
        Ok(None)
    }

    /// A tap on a cast mote opens a cast site, exactly as it does in an orbit;
    /// any other mote here is carried away instead.
    fn select(&mut self, slot: usize) -> Option<Event> {
        let index = self.surface.spec_index(slot)?;
        let mote = self.contents().get(index)?.clone();
        if mote.kind() != Kind::Cast {
            return None;
        }
        open_cast(&mut self.casting, slot, mote.clone(), &self.surface);
        Some(Event::Casting(mote))
    }

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
        self.held = Some(slot);
        self.bodies.make_dynamic(slot, radius)
    }

    fn build_released(&self, slot: usize, at: Vec3, velocity: Vec3) -> Option<Released> {
        let index = self.surface.spec_index(slot)?;
        Some(Released {
            mote:    self.contents().get(index)?.clone(),
            landing: Landing { at, velocity },
        })
    }
}
