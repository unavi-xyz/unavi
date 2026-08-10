//! The VUI gallery: a showcase consumer of `unavi-vui`.
//!
//! All state and every drawn value lives in the library — this file only
//! aims the pointer, forwards input, and transcribes the resulting views onto
//! prims.

use std::{
    cell::{
        Cell,
        RefCell,
    },
    time::SystemTime,
};

use unavi_vui::{
    grasp::Outcome,
    mote::{
        MoteKind,
        Role,
    },
    palette::Palette,
    tree::{
        Navigation,
        Node,
        Tree,
    },
    tuning::Tuning,
    view::{
        Aim,
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
        input::{
            context::register_global_input_listener,
            types::{
                InputAction,
                InputListener,
            },
        },
        scene::{
            api::self_document,
            types::Prim,
        },
    },
};

mod bodies;
mod planted;

wired_prelude::generate_script!(Script);

const CAPACITY: usize = 16;
const PLANTED_CAPACITY: usize = 12;
const PLANT_DISTANCE: f32 = 1.1;
const PLANT_HEIGHT: f32 = -0.15;

struct Script {
    tree:        Tree,
    orbit:       Orbit,
    bodies:      Bodies,
    planted:     Planted,
    input:       InputListener,
    camera:      RefCell<Option<Prim>>,
    anchor:      Cell<Option<Transform>>,
    aim:         Cell<Option<Aim>>,
    eye:         Cell<Option<Vec3>>,
    /// Distance from the eye at which a mote was taken, held constant while
    /// it is carried. Desktop's stand-in for a tracked hand: the body stays
    /// the depth you grabbed it at and sweeps freely as you look around.
    hold:        Cell<Option<f32>>,
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

    /// Where the view ray crosses the orbit plane, in the plane's own
    /// coordinates and in world space.
    fn look(camera: &Transform, anchor: &Transform) -> Option<Aim> {
        let normal = anchor.rotation * Vec3::Z;
        let forward = camera.rotation * Vec3::new(0.0, 0.0, -1.0);
        let denominator = forward.dot(normal);
        if denominator.abs() < 1.0e-6 {
            return None;
        }
        let distance = (anchor.translation - camera.translation).dot(normal) / denominator;
        if distance < 0.0 {
            return None;
        }
        let world = camera.translation + forward * distance;
        let relative = world - anchor.translation;
        Some(Aim {
            local: Vec2::new(
                relative.dot(anchor.rotation * Vec3::X),
                relative.dot(anchor.rotation * Vec3::Y),
            ),
            world,
        })
    }

    fn report(&self, navigation: &Navigation) {
        match navigation {
            Navigation::Bloomed(label) => {
                println!("[{}] opened '{label}'", self.tree.depth());
            }
            Navigation::Collapsed => println!("[{}] back to '{}'", self.tree.depth(), self.tree.here()),
            Navigation::Activated(label) => println!("activated '{label}'"),
            Navigation::Cast(label) => println!("'{label}' wants a cast circle"),
            Navigation::None => {}
        }
    }

    /// Pulling a mote out of the ring and letting go.
    ///
    /// This is what the grasp verb is for: the mote becomes a body in the
    /// room with a location of its own, which is the whole "orbits deliver
    /// objects, they are not the destination" premise made concrete. The
    /// parent mote is navigation rather than a thing, so it resets instead.
    fn place(&mut self, slot: usize, at: Vec3, velocity: Vec3) {
        let level = self.tree.level();
        let Some(spec) = level.get(slot) else {
            return;
        };

        if matches!(spec.role, Role::Parent { .. }) {
            let navigation = self.tree.reset();
            self.report(&navigation);
            return;
        }

        let Some(view) = self.orbit.views().get(slot) else {
            return;
        };

        match self.planted.plant(at, velocity, view.style) {
            Ok(recycled) => {
                println!("planted '{}' in the room", spec.label);
                if recycled {
                    println!("  (the oldest planted body was reused)");
                }
            }
            Err(err) => println!("could not plant '{}': {err:?}", spec.label),
        }
    }

    fn handle_input(&mut self) {
        while let Some(event) = self.input.poll() {
            match event.action {
                InputAction::GrabDown => {
                    if let (Some(aim), Some(eye)) = (self.aim.get(), self.eye.get()) {
                        self.hold.set(Some((aim.world - eye).length()));
                        self.orbit.press(aim.world);
                    }
                }
                InputAction::GrabUp => {
                    let outcome = self.orbit.release();
                    self.hold.set(None);
                    match outcome {
                        Some(Outcome::Tap(slot)) => {
                            let navigation = self.tree.select(slot);
                            self.report(&navigation);
                        }
                        Some(Outcome::Place { slot, at, velocity }) => {
                            self.place(slot, at, velocity);
                        }
                        None => {}
                    }
                }
                // Going up is the parent mote's job, not a keybinding: an
                // orbit is a general-purpose primitive and a menu button
                // belongs to whatever shell owns it.
                InputAction::MenuDown
                | InputAction::MenuUp
                | InputAction::ScrollUp
                | InputAction::ScrollDown => {}
            }
        }
    }
}

/// Uneven on purpose: branch sizes differ so a layout has to cope with
/// whatever count a level happens to have, one branch overflows the pip cap,
/// and it nests deep enough to show that depth is unbounded.
///
/// Only the leaves that stand for *things* — items and spawnables — are
/// takeable. Commands, spaces and tools are fixed, because pulling a command
/// out of a menu means nothing.
fn demo_tree() -> Node {
    Node::branch(
        MoteKind::Folder,
        "Root",
        vec![
            Node::cast(MoteKind::Command, "Home"),
            Node::branch(
                MoteKind::Space,
                "Places",
                vec![
                    Node::leaf(MoteKind::Space, "Atrium"),
                    Node::leaf(MoteKind::Space, "Workshop"),
                    Node::leaf(MoteKind::Space, "Club"),
                    Node::leaf(MoteKind::Space, "Garden"),
                ],
            ),
            Node::branch(
                MoteKind::Item,
                "Pocket",
                vec![
                    Node::takeable(MoteKind::Item, "Lantern"),
                    Node::takeable(MoteKind::Item, "Crate"),
                    Node::takeable(MoteKind::Document, "Notes"),
                ],
            ),
            Node::branch(
                MoteKind::Tool,
                "Hand",
                vec![
                    Node::branch(
                        MoteKind::Tool,
                        "Spawner",
                        vec![
                            Node::takeable(MoteKind::Item, "Cube"),
                            Node::takeable(MoteKind::Item, "Sphere"),
                            Node::takeable(MoteKind::Item, "Ramp"),
                            Node::takeable(MoteKind::Item, "Plank"),
                            Node::takeable(MoteKind::Item, "Sign"),
                            Node::takeable(MoteKind::Item, "Lamp"),
                            Node::takeable(MoteKind::Item, "Door"),
                            Node::takeable(MoteKind::Item, "Window"),
                            Node::takeable(MoteKind::Item, "Pillar"),
                            Node::takeable(MoteKind::Item, "Arch"),
                        ],
                    ),
                    Node::leaf(MoteKind::Tool, "Physgun"),
                    Node::leaf(MoteKind::Tool, "Lens"),
                ],
            ),
            Node::leaf(MoteKind::Person, "Self"),
        ],
    )
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let doc = self_document()?;

        println!("vui gallery");
        println!("  look          aim; the lit mote leans toward you");
        println!("  click         open a container, or activate a leaf");
        println!("  drag out      only items can be taken; they drop where you let go");
        println!("  click centre  go up a level");
        println!("  drag centre   collapse all the way to the root");
        println!("reading a mote:");
        println!("  big + see-through   container");
        println!("  small + solid       leaf");
        println!("  small + dim, centre parent, the way back");
        println!("  pips inside         what it holds; see-through pips are containers");
        println!("  pips around         how deep you are");
        println!("items are under Pocket, and under Hand > Spawner");

        Ok(Self {
            tree:        Tree::new(demo_tree()),
            orbit:       Orbit::new(CAPACITY, Tuning::DEFAULT, Palette::DEFAULT),
            bodies:      Bodies::new(&doc, CAPACITY)?,
            planted:     Planted::new(&doc, PLANTED_CAPACITY)?,
            input:       register_global_input_listener()?,
            camera:      RefCell::new(None),
            anchor:      Cell::new(None),
            aim:         Cell::new(None),
            eye:         Cell::new(None),
            hold:        Cell::new(None),
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

        self.handle_input();
        Ok(())
    }

    fn update(&mut self) -> anyhow::Result<()> {
        let delta = self.update_time.elapsed()?.as_secs_f32();
        self.update_time = SystemTime::now();

        let (Some(camera), Some(anchor)) = (self.camera(), self.anchor.get()) else {
            return Ok(());
        };

        let aim = Self::look(&camera, &anchor);
        self.aim.set(aim);
        self.eye.set(Some(camera.translation));

        // A carried mote hangs off the view ray at the depth it was taken
        // from, rather than sliding around the orbit's plane.
        let hand = self.hold.get().map_or_else(
            || aim.map(|aim| aim.world),
            |distance| {
                Some(camera.translation + camera.rotation * Vec3::new(0.0, 0.0, -1.0) * distance)
            },
        );

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

        self.bodies.apply(self.orbit.views())
    }
}

/// Level rotation facing `forward`, so the ring stands upright regardless of
/// where the player was looking when it was planted.
fn yaw_only(forward: Vec3) -> Quat {
    let theta = (-forward.x).atan2(-forward.z);
    Quat::new(0.0, (theta * 0.5).sin(), 0.0, (theta * 0.5).cos())
}
