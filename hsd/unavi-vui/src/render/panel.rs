//! A self-contained panel: owns its content, its machinery, its prims, its
//! input and its cast site. An orbit and a grid are the same panel with a
//! different [`Content`].

use wired_guest::math::{
    Transform,
    Vec2,
    Vec3,
};

use crate::{
    grasp::Outcome,
    layout::{
        Centre,
        Layout,
    },
    mote::{
        Arrange,
        MoteSpec,
    },
    palette::Palette,
    pointer::{
        self,
        Gaze,
    },
    render::{
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
    wired::scene::document::Document,
};

/// What a panel draws. An orbit navigates a level of a tree; a grid is a
/// fixed destination over one mote's children.
enum Content {
    /// A level open into a tree; layout follows the open mote's own
    /// arrangement.
    Level(Tree),
    /// A destination with a size of its own: a mote filed here is taken out
    /// of the level it came from and hung under `root`.
    Shelf { root: Mote, layout: Layout },
}

impl Content {
    /// The motes drawn, and what they draw as, in one pass.
    fn motes_and_specs(&mut self) -> (Vec<Mote>, Vec<MoteSpec>) {
        match self {
            Self::Level(tree) => tree.level_with_motes(),
            Self::Shelf { root, .. } => {
                let motes = root.children();
                let specs = motes.iter().map(Mote::spec).collect();
                (motes, specs)
            }
        }
    }

    /// The mote drawn at `index` of [`Content::motes_and_specs`].
    fn at(&mut self, index: usize) -> Option<Mote> {
        match self {
            Self::Level(tree) => tree.at_level(index),
            Self::Shelf { root, .. } => root.children().get(index).cloned(),
        }
    }

    /// The layout this content takes, and how many leading slots are pinned.
    fn layout(&mut self, capacity: usize, tuning: &Tuning) -> (Layout, usize) {
        match self {
            Self::Level(tree) => match tree.arrange() {
                Arrange::Orbit => {
                    let centre = if tree.is_nested() {
                        Centre::Held
                    } else {
                        Centre::Open
                    };
                    Layout::orbit(
                        tree.level_motes().len(),
                        centre,
                        capacity,
                        tuning.orbit_radius,
                    )
                }
                Arrange::Grid => (
                    Layout::grid(
                        tuning.grid_columns,
                        tuning.grid_rows,
                        Vec2::splat(tuning.grid_pitch),
                    ),
                    usize::from(tree.is_nested()),
                ),
            },
            Self::Shelf { layout, .. } => (*layout, 0),
        }
    }

    /// Navigates to `index`, only a level's business.
    fn select(&mut self, index: usize) -> Option<Event> {
        let Self::Level(tree) = self else {
            return None;
        };
        match tree.select(index) {
            Navigation::Bloomed(mote) => Some(Event::Opened(mote)),
            Navigation::Collapsed(mote) => Some(Event::Closed(mote)),
            Navigation::Activated(mote) => Some(Event::Activated(mote)),
            Navigation::Cast | Navigation::None => None,
        }
    }

    /// Opens `mote` as this content's level, only a level's business.
    fn open(&mut self, mote: &Mote) -> Option<Event> {
        let Self::Level(tree) = self else {
            return None;
        };
        let index = tree.level_motes().iter().position(|drawn| drawn.is(mote))?;
        self.select(index)
    }

    /// Whether a release at `local` lands inside this content, only a
    /// shelf's business.
    fn accepts(&self, local: Vec2, tuning: &Tuning) -> bool {
        match self {
            Self::Level(_) => false,
            Self::Shelf { layout, .. } => layout.accepts(local, tuning),
        }
    }

    /// Takes a released mote in, only a shelf's business.
    fn stow(&mut self, mote: &Mote) -> bool {
        match self {
            Self::Level(_) => false,
            Self::Shelf { root, .. } => root.add_child(mote),
        }
    }
}

/// A panel drawing one surface's worth of motes: an orbit navigable into a
/// tree, or a grid that files what is dropped on it.
pub struct Panel {
    content: Content,
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

impl Panel {
    /// A level of motes arranged around an anchor, selected by direction.
    pub fn orbit(
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
        let site = Site::new(doc, bodies.root(), tuning, palette)?;

        Ok(Self {
            content: Content::Level(Tree::new(root)),
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

    /// A bounded grid of `root`'s motes, and a destination a release over it
    /// lands in.
    pub fn grid(
        doc: &Document,
        root: Mote,
        layout: Layout,
        mount: Mount,
        tuning: &Tuning,
        palette: Palette,
    ) -> anyhow::Result<Self> {
        let capacity = layout.len();
        let surface = Surface::new(capacity, *tuning, palette);
        let extents = layout.extents(tuning);
        let bodies = Bodies::new(doc, capacity, tuning, Hit::Slab { extents })?;
        let site = Site::new(doc, bodies.root(), tuning, palette)?;

        Ok(Self {
            content: Content::Shelf { root, layout },
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

    pub const fn mount(&self) -> Mount {
        self.mount
    }

    pub fn place(&self, doc: &Document, anchor: &Transform) -> anyhow::Result<()> {
        self.bodies.place(doc, anchor)
    }

    pub const fn field_lift(&self) -> f32 {
        self.surface.tuning().field_lift
    }

    /// The prim everything this panel drew hangs from. Removing it takes the
    /// whole panel — bodies, placard and cast site alike — out of the scene
    /// in one edit.
    pub const fn root(&self) -> (u64, u64) {
        self.bodies.root()
    }

    /// Puts the panel up or takes it down, keeping its prims either way.
    pub fn show(&mut self, doc: &Document, shown: bool) -> anyhow::Result<()> {
        self.surface.set_open(shown);
        self.bodies.show(doc, shown)
    }

    /// Whether anything of it is still drawn. A panel sent away keeps being
    /// stepped until it has finished leaving.
    pub fn is_visible(&self) -> bool {
        self.surface.is_visible()
    }

    /// Whether a release at `local` — in this panel's own plane — files into
    /// it. Only a shelf is a destination; an orbit has no extents to land in.
    pub fn accepts(&self, local: Vec2) -> bool {
        self.content.accepts(local, self.surface.tuning())
    }

    /// Takes a released mote in, reporting whether it did.
    pub fn stow(&mut self, mote: &Mote) -> bool {
        self.content.stow(mote)
    }

    /// Opens `mote` as this panel's level, reporting what that did. Only a
    /// panel holding a tree can navigate one.
    ///
    /// A level let go in the room opens the same way a tapped one does, so
    /// the way back and everything else about it reads identically.
    pub fn open(&mut self, mote: &Mote) -> Option<Event> {
        self.content.open(mote)
    }

    /// Steps and draws. Call from the script's `update`, where animation
    /// belongs — pinning it to the fixed rate makes motion step.
    pub fn update(
        &mut self,
        doc: &Document,
        gaze: &Gaze,
        anchor: Transform,
        delta: f32,
    ) -> anyhow::Result<Vec<Event>> {
        let hand = self.depth.map(|depth| pointer::hand(&gaze.ray, depth));
        let frame = Frame {
            anchor,
            aim: pointer::aim(&gaze.ray, &anchor, self.surface.tuning().field_lift),
            hand,
            delta,
        };

        let (motes, specs) = self.content.motes_and_specs();
        let (layout, pinned) = self
            .content
            .layout(self.surface.capacity(), self.surface.tuning());
        self.surface.update(&specs, layout, pinned, &frame);

        self.bodies.icons(
            doc,
            &motes,
            &specs,
            self.surface.views(),
            self.surface.drawn(),
            delta,
        )?;
        self.bodies.apply(
            doc,
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
            doc,
            &mut self.casting,
            &self.surface,
            &self.site,
            delta,
            &mut events,
        )?;
        self.hand_over(doc)?;
        Ok(events)
    }

    /// Reads input and resolves what it did. Call from the script's
    /// `fixed_update`, where state belongs.
    pub fn fixed_update(
        &mut self,
        doc: &Document,
        gaze: &Gaze,
        anchor: Transform,
    ) -> anyhow::Result<FixedUpdate> {
        let mut done = FixedUpdate {
            events:   Vec::new(),
            released: None,
            opens_at: None,
        };
        for signal in self.bodies.poll() {
            match (signal, self.surface.is_seized()) {
                (Signal::Act(true), false) => self.press(gaze, anchor, false, &mut done.events),
                (Signal::Take(true), false) => self.press(gaze, anchor, true, &mut done.events),
                (Signal::Act(false) | Signal::Take(false), _) => self.release(doc, &mut done)?,
                (Signal::Turn(delta), false) => self.surface.turn_by(delta),
                (Signal::Act(true) | Signal::Take(true) | Signal::Turn(_), true) => {}
            }
        }
        Ok(done)
    }

    /// Grabs the lit mote at its drawn depth, so it arrives under the pointer.
    ///
    /// A consequential mote opens its cast site here rather than on release:
    /// the hold *is* the confirmation, so the ring starts filling the moment
    /// the mote is taken hold of and letting go early abandons it.
    fn press(&mut self, gaze: &Gaze, anchor: Transform, carrying: bool, events: &mut Vec<Event>) {
        let Some(slot) = self.surface.attended() else {
            return;
        };
        let Some(view) = self.surface.views().get(slot) else {
            return;
        };
        let world = anchor.translation + anchor.rotation * view.position;
        let depth = (world - gaze.ray.origin).length();
        self.depth = Some(depth);
        self.surface
            .press(pointer::hand(&gaze.ray, depth), carrying);

        if let Some(mote) = self.consequential(slot) {
            open_cast(&mut self.casting, slot, mote.clone(), &self.surface);
            events.push(Event::Casting(mote));
        }
    }

    /// The mote drawn in `slot`, if holding it is what fires it.
    fn consequential(&mut self, slot: usize) -> Option<Mote> {
        let index = self.surface.spec_index(slot)?;
        self.content
            .at(index)
            .filter(|mote| mote.kind() == Kind::Cast)
    }

    /// Returns the carried mote to its panel, reporting what a tap did and
    /// handing the host anything that was placed.
    ///
    /// What a landing means belongs to the mote that landed: a level opens
    /// where it was let go, and anything else is the consumer's to place.
    ///
    /// An item fired where it sits lands too, at the slot it was drawn in.
    /// Both buttons deliver the item; the grip only chooses where.
    fn release(&mut self, doc: &Document, done: &mut FixedUpdate) -> anyhow::Result<()> {
        self.depth = None;
        let outcome = self.surface.release();

        let carried = self.held.take();
        let landed = carried.and_then(|slot| {
            let bodies = &self.bodies;
            Some((
                bodies.pose(doc, slot)?.translation,
                bodies.velocity(doc, slot),
            ))
        });
        if let Some(slot) = carried {
            self.bodies.clear_dynamic(doc, slot)?;
        }

        match outcome {
            Some(Outcome::Tap(slot)) => {
                if let Some(released) = self.delivered(doc, slot) {
                    done.released = Some(released);
                } else if let Some(event) = self.select(slot) {
                    done.events.push(event);
                }
            }
            Some(Outcome::Place(slot)) => {
                if let Some((at, velocity)) = landed {
                    done.released = self.build_released(slot, at, velocity);
                    done.opens_at = self.opens(slot).then_some(at);
                }
            }
            None => {}
        }
        Ok(())
    }

    /// The item drawn in `slot`, landing where it stands.
    fn delivered(&mut self, doc: &Document, slot: usize) -> Option<Released> {
        let index = self.surface.spec_index(slot)?;
        let mote = self
            .content
            .at(index)
            .filter(|mote| mote.kind().delivers())?;
        Some(Released {
            mote,
            landing: Landing {
                at:       self.bodies.pose(doc, slot)?.translation,
                velocity: Vec3::ZERO,
            },
        })
    }

    /// Whether the mote drawn in `slot` is a level rather than a thing.
    fn opens(&mut self, slot: usize) -> bool {
        self.surface
            .spec_index(slot)
            .and_then(|index| self.content.at(index))
            .is_some_and(|mote| mote.kind().holds_children())
    }

    /// Selects whatever holds attention in `slot`, navigating the content.
    ///
    /// A consequential mote is not selected on release: its site opened when
    /// it was pressed, and by the time the grasp lets go the cast has either
    /// fired or been abandoned.
    fn select(&mut self, slot: usize) -> Option<Event> {
        let index = self.surface.spec_index(slot)?;
        self.content.select(index)
    }

    /// Says how much is off the page, because a panel that quietly holds
    /// three of eighteen apples is a panel that is lying.
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
    fn hand_over(&mut self, doc: &Document) -> anyhow::Result<()> {
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
        self.bodies.make_dynamic(doc, slot, radius)
    }

    fn build_released(&mut self, slot: usize, at: Vec3, velocity: Vec3) -> Option<Released> {
        let index = self.surface.spec_index(slot)?;
        Some(Released {
            mote:    self.content.at(index)?,
            landing: Landing { at, velocity },
        })
    }
}
