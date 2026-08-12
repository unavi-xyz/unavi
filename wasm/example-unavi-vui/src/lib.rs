//! The VUI gallery: a showcase consumer of `unavi-vui`.

use std::{
    cell::{
        Cell,
        RefCell,
    },
    time::SystemTime,
};

use smol_str::SmolStr;
use unavi_vui::{
    circle::{
        Cast,
        Circle,
    },
    grasp::Outcome,
    model::Model,
    mote::MoteSpec,
    orbit::{
        Centre,
        Orbit,
    },
    palette::Palette,
    pointer,
    rack::{
        Rack,
        Shelf,
    },
    sigil::{
        Cardinal,
        Sigil,
        Step,
    },
    trail,
    tree::{
        Navigation,
        Node,
        Tree,
    },
    tuning::Tuning,
    view::Frame,
};
use wired_prelude::prelude::*;

use crate::{
    beads::Trail,
    bodies::{
        Bodies,
        Hit,
        Signal,
    },
    cast::Circle as CastSite,
    planted::Planted,
    wired::{
        agent::api::local_camera,
        scene::{
            api::self_document,
            types::Prim,
        },
    },
};

mod beads;
mod bodies;
mod cast;
mod placard;
mod planted;

wired_prelude::generate_script!(Script);

const TUNING: Tuning = Tuning::DEFAULT;
const CAPACITY: usize = 16;
const PLANTED_CAPACITY: usize = 12;
const PLANT_DISTANCE: f32 = 1.1;
const PLANT_HEIGHT: f32 = -0.15;
/// Small and fixed on purpose: a pocket you reach into, not a filing cabinet.
const POCKET_COLUMNS: usize = 4;
const POCKET_ROWS: usize = 3;
/// Off to the user's left, so it never sits across the orbit.
const POCKET_OFFSET: f32 = -0.62;

/// A consequential action, mid-cast.
struct Casting {
    slot:   usize,
    label:  SmolStr,
    circle: Circle,
}

struct Script {
    tree:        Tree<Node>,
    orbit:       Orbit,
    bodies:      Bodies,
    trail:       Trail,
    site:        CastSite,
    casting:     Option<Casting>,
    pocket:      RefCell<Vec<MoteSpec>>,
    rack:        Rack,
    shelf:       Bodies,
    rack_anchor: Cell<Option<Transform>>,
    planted:     Planted,
    camera:      RefCell<Option<Prim>>,
    anchor:      Cell<Option<Transform>>,
    /// The slot the engine's grab is carrying. Its transform belongs to the
    /// solver until it comes back.
    held:        Cell<Option<usize>>,
    /// The pressed mote's depth along the view ray, standing in for a tracked
    /// hand on desktop.
    depth:       Cell<Option<f32>>,
    /// Last page reported, so a paged level says so once rather than every
    /// frame.
    paged:       Cell<Option<usize>>,
    update_time: SystemTime,
}

impl Script {
    fn camera(&self) -> Option<Transform> {
        let mut camera = self.camera.borrow_mut();
        if camera.is_none() {
            *camera = local_camera().ok();
        }
        camera.as_ref().map(Prim::global_xform)
    }

    fn report(&self, navigation: &Navigation) {
        match navigation {
            Navigation::Bloomed(label) => {
                println!("[{}] opened '{label}'{}", self.tree.depth(), self.sigil());
            }
            Navigation::Collapsed => {
                println!("[{}] back to '{}'", self.tree.depth(), self.tree.here().label);
            }
            Navigation::Activated(label) => println!("activated '{label}'{}", self.sigil()),
            Navigation::Cast(_) | Navigation::None => {}
        }
    }

    /// The eyes-free motion that reaches where the tree now stands, or a plain
    /// statement that there is none.
    ///
    /// Derived from the rings actually drawn on the way down, so it cannot
    /// drift out of sync with the menu.
    fn sigil(&self) -> String {
        let mut steps = Vec::with_capacity(self.tree.depth());
        let mut prefix = Vec::with_capacity(self.tree.depth());
        for (depth, &index) in self.tree.path().iter().enumerate() {
            let nested = depth > 0;
            let centre = if nested { Centre::Held } else { Centre::Open };
            let count = self.tree.model().children(&prefix).len() + usize::from(nested);
            let ring = self.orbit.ring(count, centre);
            steps.push(Step {
                slot:    index + usize::from(ring.has_centre()),
                points:  ring.ring_len(),
                centred: ring.has_centre(),
            });
            prefix.push(index);
        }

        let Some(sigil) = Sigil::for_path(&steps) else {
            return "  (too deep for a sigil — reachable by navigation only)".to_string();
        };
        sigil.cardinals().map_or_else(
            || format!("  ({} deflections)", sigil.deflections()),
            |cardinals| {
                let named = cardinals
                    .iter()
                    .map(|cardinal| match cardinal {
                        Cardinal::Up => "up",
                        Cardinal::Right => "right",
                        Cardinal::Down => "down",
                        Cardinal::Left => "left",
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("  (sigil: {named})")
            },
        )
    }

    /// Says how much is off the page, because a ring that quietly holds three
    /// of eighteen apples is a ring that is lying.
    fn report_page(&self) {
        let page = self.orbit.page();
        if !page.is_paged() {
            self.paged.set(None);
            return;
        }
        if self.paged.get() == Some(page.index) {
            return;
        }
        self.paged.set(Some(page.index));
        println!(
            "  page {} of {} ({} motes here){}",
            page.index + 1,
            page.count,
            page.total,
            if page.has_next() {
                " — scroll for more"
            } else {
                " — scroll back"
            }
        );
    }

    fn hand_over(&self) {
        if self.held.get().is_some() {
            return;
        }
        let Some(slot) = self.orbit.displaced() else {
            return;
        };
        let Some(view) = self.orbit.views().get(slot) else {
            return;
        };
        if let Err(err) = self.bodies.make_dynamic(slot, view.radius) {
            eprintln!("could not hand mote to the engine: {err:?}");
        }
        // Tracked even when the promotion failed, so a half-promoted mote is
        // still stripped back on release.
        self.held.set(Some(slot));
    }

    /// Grabs the lit mote at its drawn depth, so it arrives under the pointer.
    fn press(&mut self, camera: &Transform) {
        let (Some(slot), Some(anchor)) = (self.orbit.attended(), self.anchor.get()) else {
            return;
        };
        let Some(view) = self.orbit.views().get(slot) else {
            return;
        };
        let world = anchor.translation + anchor.rotation * view.position;
        let depth = (world - camera.translation).length();
        self.depth.set(Some(depth));
        self.orbit.press(pointer::hand(camera, depth));
    }

    /// Returns the carried mote to its orbit, stowing or planting it depending
    /// on where the release landed.
    fn release(&mut self, camera: &Transform) {
        self.depth.set(None);
        let outcome = self.orbit.release();
        let carried = self.held.take();
        let landed = carried.and_then(|slot| {
            let at = self.bodies.pose(slot)?.translation;
            Some((at, self.bodies.velocity(slot)))
        });
        if let Some(slot) = carried
            && let Err(err) = self.bodies.clear_dynamic(slot)
        {
            eprintln!("could not return mote to its orbit: {err:?}");
        }

        match outcome {
            Some(Outcome::Tap(slot)) => self.select(slot),
            // A place with nothing carried is a mote the engine never took;
            // it springs home.
            Some(Outcome::Place(slot)) => {
                if let Some((at, velocity)) = landed {
                    if self.over_pocket(camera) {
                        self.stow(slot);
                    } else {
                        self.plant(slot, at, velocity);
                    }
                }
            }
            None => {}
        }
    }

    /// A cast opens a site rather than firing; everything else resolves now.
    fn select(&mut self, slot: usize) {
        let Some(index) = self.orbit.spec_index(slot) else {
            return;
        };
        let navigation = self.tree.select(index);
        if let Navigation::Cast(label) = &navigation {
            println!("'{label}' opens a cast — hold it to commit, look away to abort");
            self.casting = Some(Casting {
                slot,
                label: label.clone(),
                circle: Circle::standard(&TUNING),
            });
            return;
        }
        self.report(&navigation);
    }

    /// Whether the pointer is over the pocket, which files rather than plants.
    fn over_pocket(&self, camera: &Transform) -> bool {
        self.rack_anchor.get().is_some_and(|anchor| {
            pointer::aim(camera, &anchor, TUNING.field_lift)
                .is_some_and(|aim| self.rack.accepts(aim.local))
        })
    }

    fn stow(&self, slot: usize) {
        let Some(spec) = self
            .orbit
            .spec_index(slot)
            .and_then(|index| self.tree.level().get(index).cloned())
        else {
            return;
        };
        println!("stowed '{}' in the pocket", spec.label);
        self.pocket.borrow_mut().push(spec);
    }

    fn plant(&self, slot: usize, at: Vec3, velocity: Vec3) {
        let level = self.tree.level();
        let Some(spec) = self.orbit.spec_index(slot).and_then(|index| level.get(index)) else {
            return;
        };
        match self
            .planted
            .plant(at, velocity, self.orbit.resting_style(spec))
        {
            Ok(recycled) => {
                println!("planted '{}' in the room", spec.label);
                if recycled {
                    println!("  (the oldest planted body was reused)");
                }
            }
            Err(err) => println!("could not plant '{}': {err:?}", spec.label),
        }
    }

    /// Fills while attention stays on the mote that opened it, and aborts the
    /// moment it leaves.
    fn drive_cast(&mut self, delta: f32) -> anyhow::Result<()> {
        let Some(casting) = &mut self.casting else {
            return self.site.hide();
        };
        let held = self.orbit.attended() == Some(casting.slot);
        match casting.circle.update(held, delta) {
            Cast::Filling(_) => {
                let at = self
                    .orbit
                    .views()
                    .get(casting.slot)
                    .map_or(Vec3::ZERO, |view| view.position);
                self.site.apply(at, casting.circle.cast())?;
            }
            Cast::Committed => {
                println!("cast '{}' committed", casting.label);
                self.casting = None;
                self.site.hide()?;
            }
            Cast::Aborted => {
                println!("cast '{}' aborted — you pulled away", casting.label);
                self.casting = None;
                self.site.hide()?;
            }
        }
        Ok(())
    }

    fn handle_input(&mut self, camera: &Transform) {
        for signal in self.bodies.poll() {
            match (signal, self.orbit.is_seized()) {
                (Signal::Grab(true), false) => self.press(camera),
                (Signal::Grab(false), _) => self.release(camera),
                (Signal::Turn(delta), false) => self.orbit.turn_by(delta),
                (Signal::Grab(true) | Signal::Turn(_), true) => {}
            }
        }
        for signal in self.shelf.poll() {
            if let Signal::Turn(delta) = signal {
                self.rack.turn_by(delta);
            }
        }
    }
}

/// Deliberately uneven: group sizes differ, one group overflows the pip cap,
/// one outruns the ring and has to paginate, and the depth is unbounded.
fn demo_tree() -> Node {
    Node::group(
        "Produce",
        vec![
            Node::cast("Empty the crate").describe("Removes every item at once. There is no undo."),
            Node::group(
                "Citrus",
                vec![
                    Node::item("Lemon").describe("Sharp and thin-skinned."),
                    Node::item("Lime").describe("Smaller, and greener."),
                    Node::item("Orange").describe("The one everybody pictures."),
                    Node::item("Grapefruit").describe("Bitter enough to divide a room."),
                ],
            )
            .describe("Sharp fruit with a thick rind."),
            Node::group(
                "Berries",
                vec![
                    Node::item("Strawberry").describe("Not botanically a berry."),
                    Node::item("Blueberry").describe("Actually a berry."),
                    Node::item("Raspberry").describe("A cluster of tiny fruits."),
                ],
            )
            .describe("Small, soft, and quick to spoil."),
            Node::group(
                "Orchard",
                vec![
                    Node::group(
                        "Apples",
                        vec![
                            Node::item("Gala"),
                            Node::item("Fuji"),
                            Node::item("Bramley"),
                            Node::item("Pink Lady"),
                            Node::item("Granny Smith"),
                            Node::item("Braeburn"),
                            Node::item("Cox"),
                            Node::item("Discovery"),
                            Node::item("Egremont"),
                            Node::item("Worcester"),
                            Node::item("Katy"),
                            Node::item("Spartan"),
                            Node::item("Jazz"),
                            Node::item("Envy"),
                            Node::item("Russet"),
                            Node::item("Ambrosia"),
                            Node::item("Jonagold"),
                            Node::item("Empire"),
                        ],
                    )
                    .describe("More kinds than one ring holds; scroll to page."),
                    Node::item("Pear").describe("Ripe for about an hour."),
                    Node::item("Quince").describe("Inedible raw, excellent cooked."),
                ],
            )
            .describe("Tree fruit, and the deepest level here."),
            Node::action("Sort by name").describe("Reorders this level. Reversible."),
        ],
    )
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let doc = self_document()?;

        println!("vui gallery");
        println!("  look          aim; the lit mote leans toward you");
        println!("  click         open a container, or activate a leaf");
        println!("  drag out      only items can be taken; the engine carries them");
        println!("  drop on shelf stow it in the pocket, off to your left");
        println!("  scroll        turn the page of whatever you are pointing at");
        println!("  click centre  go up a level");
        println!("reading a mote:");
        println!("  big + see-through   container");
        println!("  small + solid       leaf");
        println!("  small + dim, centre parent, the way back");
        println!("  pips inside         what it holds; see-through pips are containers");
        println!("  pips around         how deep you are");
        println!("  beads behind        the levels above this one");
        println!("'Empty the crate' is a cast: hold it, or look away to abort");

        let orbit = Orbit::new(CAPACITY, TUNING, Palette::DEFAULT);
        let bodies = Bodies::new(
            &doc,
            CAPACITY,
            &TUNING,
            Hit::Disc {
                radius: orbit.reach(),
            },
        )?;
        let trail = Trail::new(&doc, bodies.root())?;
        let site = CastSite::new(&doc, bodies.root(), Palette::DEFAULT)?;

        let shelf = Shelf::grid(
            POCKET_COLUMNS,
            POCKET_ROWS,
            Vec2::splat(TUNING.rack_pitch),
        );
        let rack = Rack::new(shelf, TUNING, Palette::DEFAULT);
        let shelf_bodies = Bodies::new(
            &doc,
            shelf.cells(),
            &TUNING,
            Hit::Slab {
                extents: rack.extents(),
            },
        )?;

        Ok(Self {
            tree: Tree::new(demo_tree()),
            orbit,
            bodies,
            trail,
            site,
            casting: None,
            pocket: RefCell::new(Vec::new()),
            rack,
            shelf: shelf_bodies,
            rack_anchor: Cell::new(None),
            planted: Planted::new(&doc, PLANTED_CAPACITY)?,
            camera: RefCell::new(None),
            anchor: Cell::new(None),
            held: Cell::new(None),
            depth: Cell::new(None),
            paged: Cell::new(None),
            update_time: SystemTime::now(),
        })
    }

    fn fixed_update(&mut self) -> anyhow::Result<()> {
        let Some(camera) = self.camera() else {
            return Ok(());
        };

        if self.anchor.get().is_none() {
            let forward = camera.rotation * Vec3::new(0.0, 0.0, -1.0);
            let flat = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
            let rotation = yaw_only(flat);
            let anchor = Transform {
                translation: camera.translation
                    + flat * PLANT_DISTANCE
                    + Vec3::new(0.0, PLANT_HEIGHT, 0.0),
                rotation,
                scale: Vec3::ONE,
            };
            self.anchor.set(Some(anchor));
            self.bodies.place(&anchor)?;

            let pocket = Transform {
                translation: anchor.translation + rotation * Vec3::new(POCKET_OFFSET, 0.0, 0.0),
                rotation,
                scale: Vec3::ONE,
            };
            self.rack_anchor.set(Some(pocket));
            self.shelf.place(&pocket)?;

            println!("[0] '{}' — walk up to it", self.tree.here().label);
        }

        self.handle_input(&camera);
        Ok(())
    }

    fn update(&mut self) -> anyhow::Result<()> {
        let delta = self.update_time.elapsed()?.as_secs_f32();
        self.update_time = SystemTime::now();

        let (Some(camera), Some(anchor)) = (self.camera(), self.anchor.get()) else {
            return Ok(());
        };

        let aim = pointer::aim(&camera, &anchor, TUNING.field_lift);
        let hand = self.depth.get().map(|depth| pointer::hand(&camera, depth));
        let specs = self.tree.level();
        let centre = if self.tree.is_nested() {
            Centre::Held
        } else {
            Centre::Open
        };
        self.orbit.update(
            &specs,
            centre,
            &Frame {
                eye: camera.translation,
                anchor,
                aim,
                hand,
                delta,
            },
        );

        // Drawn before handing over: once promoted, the engine owns the
        // mote's transform, so drawing after would freeze it a frame behind
        // the pointer.
        self.bodies.apply(
            self.orbit.views(),
            &specs,
            self.orbit.drawn(),
            self.orbit.placard(),
            self.orbit.palette(),
            self.held.get(),
        )?;
        self.trail.apply(&trail::view(
            &self.tree.trail(),
            self.orbit.palette(),
            &TUNING,
        ))?;
        self.drive_cast(delta)?;
        self.report_page();
        self.hand_over();

        if let Some(pocket) = self.rack_anchor.get() {
            let stowed = self.pocket.borrow();
            self.rack.update(
                &stowed,
                &Frame {
                    eye: camera.translation,
                    anchor: pocket,
                    aim: pointer::aim(&camera, &pocket, TUNING.field_lift),
                    hand: None,
                    delta,
                },
            );
            self.shelf.apply(
                self.rack.views(),
                &stowed,
                self.rack.drawn(),
                self.rack.placard(),
                self.rack.palette(),
                None,
            )?;
        }
        Ok(())
    }
}

/// Yaw-only rotation facing `forward`, so the ring stays upright regardless
/// of camera pitch or roll.
fn yaw_only(forward: Vec3) -> Quat {
    let theta = (-forward.x).atan2(-forward.z);
    Quat::new(0.0, (theta * 0.5).sin(), 0.0, (theta * 0.5).cos())
}
