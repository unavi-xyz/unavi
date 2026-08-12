//! The VUI gallery: a showcase consumer of `unavi-vui`.

use std::{
    cell::{
        Cell,
        RefCell,
    },
    time::SystemTime,
};

use unavi_vui::{
    grasp::Outcome,
    palette::Palette,
    pointer,
    tree::{
        Navigation,
        Node,
        Tree,
    },
    tuning::Tuning,
    view::{
        Frame,
        Orbit,
    },
};
use wired_prelude::prelude::*;

use crate::{
    bodies::Bodies,
    planted::Planted,
    wired::{
        agent::api::local_camera,
        scene::{
            api::self_document,
            types::Prim,
        },
    },
};

mod bodies;
mod placard;
mod planted;

wired_prelude::generate_script!(Script);

const TUNING: Tuning = Tuning::DEFAULT;
const CAPACITY: usize = 16;
const PLANTED_CAPACITY: usize = 12;
const PLANT_DISTANCE: f32 = 1.1;
const PLANT_HEIGHT: f32 = -0.15;

struct Script {
    tree:        Tree,
    orbit:       Orbit,
    bodies:      Bodies,
    planted:     Planted,
    camera:      RefCell<Option<Prim>>,
    anchor:      Cell<Option<Transform>>,
    /// The slot the engine's grab is carrying. Its transform belongs to the
    /// solver until it comes back.
    held:        Cell<Option<usize>>,
    /// The pressed mote's depth along the view ray, standing in for a tracked
    /// hand on desktop.
    depth:       Cell<Option<f32>>,
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
                println!("[{}] opened '{label}'", self.tree.depth());
            }
            Navigation::Collapsed => {
                println!("[{}] back to '{}'", self.tree.depth(), self.tree.here());
            }
            Navigation::Activated(label) => println!("activated '{label}'"),
            Navigation::Cast(label) => println!("'{label}' wants a cast circle"),
            Navigation::None => {}
        }
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

    /// Returns the carried mote to its orbit, planting a duplicate when the
    /// release was a place.
    fn release(&mut self) {
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
            Some(Outcome::Tap(slot)) => {
                let navigation = self.tree.select(slot);
                self.report(&navigation);
            }
            // A place with nothing carried is a mote the engine never took;
            // it springs home.
            Some(Outcome::Place(slot)) => {
                if let Some((at, velocity)) = landed {
                    self.plant(slot, at, velocity);
                }
            }
            None => {}
        }
    }

    fn plant(&self, slot: usize, at: Vec3, velocity: Vec3) {
        let level = self.tree.level();
        let Some(spec) = level.get(slot) else {
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

    fn handle_input(&mut self, camera: &Transform) {
        for pressed in self.bodies.poll_grabs() {
            match (pressed, self.orbit.is_seized()) {
                (true, false) => self.press(camera),
                (false, _) => self.release(),
                (true, true) => {}
            }
        }
    }
}

/// Deliberately uneven: group sizes differ, one group overflows the pip cap,
/// and the depth is unbounded, so the layout has to cope with all of it.
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
                        ],
                    )
                    .describe("More kinds than the ring can show at once."),
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
        println!("  scroll        while carrying, push it out or pull it in");
        println!("  click centre  go up a level");
        println!("reading a mote:");
        println!("  big + see-through   container");
        println!("  small + solid       leaf");
        println!("  small + dim, centre parent, the way back");
        println!("  pips inside         what it holds; see-through pips are containers");
        println!("  pips around         how deep you are");
        println!("items are under Pocket, and under Hand > Spawner");
        println!("planted items are ordinary bodies — grab them like anything else");

        Ok(Self {
            tree:        Tree::new(demo_tree()),
            orbit:       Orbit::new(CAPACITY, TUNING, Palette::DEFAULT),
            bodies:      Bodies::new(&doc, CAPACITY, &TUNING)?,
            planted:     Planted::new(&doc, PLANTED_CAPACITY)?,
            camera:      RefCell::new(None),
            anchor:      Cell::new(None),
            held:        Cell::new(None),
            depth:       Cell::new(None),
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
            let anchor = Transform {
                translation: camera.translation
                    + flat * PLANT_DISTANCE
                    + Vec3::new(0.0, PLANT_HEIGHT, 0.0),
                rotation:    yaw_only(flat),
                scale:       Vec3::ONE,
            };
            self.anchor.set(Some(anchor));
            self.bodies.place(&anchor)?;
            println!("[0] '{}' — walk up to it", self.tree.here());
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
        self.orbit.update(
            &specs,
            self.tree.is_nested(),
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
            self.orbit.placard(),
            &self.orbit.palette,
            self.held.get(),
        )?;
        self.hand_over();
        Ok(())
    }
}

/// Yaw-only rotation facing `forward`, so the ring stays upright regardless
/// of camera pitch or roll.
fn yaw_only(forward: Vec3) -> Quat {
    let theta = (-forward.x).atan2(-forward.z);
    Quat::new(0.0, (theta * 0.5).sin(), 0.0, (theta * 0.5).cos())
}
