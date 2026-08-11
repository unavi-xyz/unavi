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
    mote::MoteKind,
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
    /// The slot the engine's grab is carrying. Its transform belongs to the
    /// solver until it comes back.
    held:        Cell<Option<usize>>,
    /// Whether the current press came from the host's own hit-test. Only
    /// those can be handed over: nothing is waiting on a mote the ray missed.
    hit_press:   Cell<bool>,
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
            Navigation::Collapsed => {
                println!("[{}] back to '{}'", self.tree.depth(), self.tree.here());
            }
            Navigation::Activated(label) => println!("activated '{label}'"),
            Navigation::Cast(label) => println!("'{label}' wants a cast circle"),
            Navigation::None => {}
        }
    }

    /// Once a hold has travelled far enough to be a take, the mote gains a
    /// body and the engine's grab — which is still watching — picks it up.
    fn hand_over(&self) {
        if self.held.get().is_some() || !self.hit_press.get() {
            return;
        }
        let Some(slot) = self.orbit.displaced() else {
            return;
        };
        match self.bodies.make_dynamic(slot) {
            Ok(()) => self.held.set(Some(slot)),
            Err(err) => eprintln!("could not hand mote to the engine: {err:?}"),
        }
    }

    /// The engine let go. Leave a planted duplicate where it ended up and
    /// give the mote back to its orbit.
    fn take_back(&self, slot: usize) {
        let pose = self.bodies.pose(slot);
        let velocity = self.bodies.velocity(slot);
        if let Err(err) = self.bodies.clear_dynamic(slot) {
            eprintln!("could not return mote to its orbit: {err:?}");
        }

        let level = self.tree.level();
        let (Some(pose), Some(spec)) = (pose, level.get(slot)) else {
            return;
        };
        match self
            .planted
            .plant(pose.translation, velocity, self.orbit.resting_style(spec))
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

    fn handle_input(&mut self) {
        for (slot, pressed) in self.bodies.poll_grabs() {
            if !pressed {
                continue;
            }
            let Some(at) = self.aim.get().map(|aim| aim.world) else {
                continue;
            };
            self.hit_press.set(true);
            self.orbit.press_slot(slot, at);
        }

        while let Some(event) = self.input.poll() {
            match event.action {
                InputAction::GrabUp => {
                    let outcome = self.orbit.release();
                    self.hit_press.set(false);
                    if let Some(slot) = self.held.take() {
                        self.take_back(slot);
                    } else if let Some(Outcome::Tap(slot)) = outcome {
                        let navigation = self.tree.select(slot);
                        self.report(&navigation);
                    }
                }
                // Attention is more forgiving than the ray, so a near miss
                // still taps what lights up.
                InputAction::GrabDown => {
                    if !self.orbit.is_seized()
                        && let (Some(slot), Some(aim)) = (self.orbit.attended(), self.aim.get())
                    {
                        self.orbit.press_slot(slot, aim.world);
                    }
                }
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
            orbit:       Orbit::new(CAPACITY, Tuning::DEFAULT, Palette::DEFAULT),
            bodies:      Bodies::new(&doc, CAPACITY)?,
            planted:     Planted::new(&doc, PLANTED_CAPACITY)?,
            input:       register_global_input_listener()?,
            camera:      RefCell::new(None),
            anchor:      Cell::new(None),
            aim:         Cell::new(None),
            held:        Cell::new(None),
            hit_press:   Cell::new(false),
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

        let hand = aim.map(|aim| aim.world);
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

        self.hand_over();
        self.bodies.apply(self.orbit.views(), self.held.get())
    }
}

/// Level rotation facing `forward`, so the ring stands upright regardless of
/// where the player was looking when it was planted.
fn yaw_only(forward: Vec3) -> Quat {
    let theta = (-forward.x).atan2(-forward.z);
    Quat::new(0.0, (theta * 0.5).sin(), 0.0, (theta * 0.5).cos())
}
