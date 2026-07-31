use std::{
    cell::{
        Cell,
        RefCell,
    },
    time::SystemTime,
};

use unavi_gauntlet_menu::{
    Menu,
    Outcome,
    ToolChange,
};
use wired_prelude::prelude::*;

use crate::{
    artifact::Artifact,
    home::Home,
    unavi::gauntlet_tool::api::{
        ToolRegistry,
        ToolState,
    },
    wheel::{
        Wheel,
        hovered_sector,
    },
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
            types::{
                Prim,
                Xform,
            },
        },
    },
};

mod artifact;
mod geometry;
mod home;
mod palette;
mod wheel;

wired_prelude::generate_script!(Script);

const MENU_FORWARD: f32 = 0.7;
const ARTIFACT_OFFSET: Vec3 = Vec3::new(0.22, -0.18, -0.5);
const CLOSE_MOVE_SQ: f32 = 0.09;
const OPEN_SPEED: f32 = 7.0;
const ART_SPEED: f32 = 5.0;
const TOOL_PLACE_DIST: f32 = 1.2;

struct ToolRef {
    doc_id: Vec<u8>,
    name:   String,
}

struct Script {
    menu:          Menu,
    registry:      ToolRegistry,
    tools:         Vec<ToolRef>,
    wheel:         Wheel,
    artifact:      Artifact,
    artifact_root: Prim,
    home:          Home,
    input:         InputListener,
    camera:        RefCell<Option<Prim>>,
    placement:     Cell<Option<Transform>>,
    open_pos:      Cell<Option<Vec3>>,
    open_t:        Cell<f32>,
    art_t:         Cell<f32>,
    hovered:       Cell<Option<usize>>,
    pressed:       Cell<bool>,
    render_time:   SystemTime,
}

impl Script {
    fn camera(&self) -> Option<Transform> {
        let mut cam = self.camera.borrow_mut();
        if cam.is_none() {
            *cam = local_camera().ok();
        }
        cam.as_ref().map(Prim::global_xform)
    }

    fn open(&mut self, cam: &Transform) {
        let forward = cam.rotation * Vec3::new(0.0, 0.0, -1.0);
        let placement = Transform {
            translation: cam.translation + forward * MENU_FORWARD,
            rotation:    cam.rotation,
            scale:       Vec3::ONE,
        };
        self.open_pos.set(Some(cam.translation));
        self.placement.set(Some(placement));
        self.menu.open();
        self.wheel.rebuild(&self.menu.slots());
        self.wheel
            .root
            .set_xform(Some(Xform {
                translation: placement.translation,
                rotation:    placement.rotation,
                scale:       Vec3::ZERO,
            }))
            .ok();
    }

    fn close(&mut self) {
        self.menu.close();
        self.hovered.set(None);
        self.open_pos.set(None);
    }

    fn handle_outcome(&mut self, outcome: Outcome, cam: &Transform) {
        match outcome {
            Outcome::None => {
                if self.menu.is_open() {
                    self.wheel.rebuild(&self.menu.slots());
                    self.hovered.set(None);
                }
            }
            Outcome::Home => {
                println!("traveling home");
                self.home.request();
            }
            Outcome::Tool(change) => self.apply_tool_change(&change, cam),
        }
    }

    fn tool_name(&self, doc: &[u8]) -> String {
        self.tools
            .iter()
            .find(|t| t.doc_id == doc)
            .map_or_else(|| "?".to_string(), |t| t.name.clone())
    }

    fn tool_color(&self, doc: &[u8]) -> Color {
        let index = self.tools.iter().position(|t| t.doc_id == doc).unwrap_or(0);
        palette::tool_color(index)
    }

    fn apply_tool_change(&self, change: &ToolChange, cam: &Transform) {
        if let Some(doc) = &change.deactivated {
            println!("deactivated tool '{}'", self.tool_name(doc));
            self.registry.deactivate(doc);
            self.registry.set_state(
                doc,
                ToolState {
                    color:  palette::SECONDARY,
                    in_use: false,
                },
            );
        }
        if let Some(doc) = &change.activated {
            println!("activated tool '{}'", self.tool_name(doc));
            let color = self.tool_color(doc);
            self.artifact.set_color(color);
            let forward = cam.rotation * Vec3::new(0.0, 0.0, -1.0);
            self.registry.activate(
                doc,
                Transform {
                    translation: cam.translation + forward * TOOL_PLACE_DIST,
                    rotation:    cam.rotation,
                    scale:       Vec3::ONE,
                },
            );
            self.registry.set_state(
                doc,
                ToolState {
                    color,
                    in_use: false,
                },
            );
        }
    }

    fn poll_tools(&mut self) {
        let mut changed = false;
        for tool in self.registry.poll() {
            if self.tools.iter().any(|t| t.doc_id == tool.doc_id) {
                continue;
            }
            self.registry.set_state(
                &tool.doc_id,
                ToolState {
                    color:  palette::SECONDARY,
                    in_use: false,
                },
            );
            self.tools.push(ToolRef {
                doc_id: tool.doc_id,
                name:   tool.name,
            });
            changed = true;
        }
        if changed {
            self.tools.sort_by(|a, b| a.name.cmp(&b.name));
            self.menu.set_tools(
                self.tools
                    .iter()
                    .map(|t| (t.doc_id.clone(), t.name.clone()))
                    .collect(),
            );
            if self.menu.is_open() {
                self.wheel.rebuild(&self.menu.slots());
            }
        }
    }
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let artifact_root = self_document()?.create_prim()?;
        artifact_root.set_xform(Some(Xform {
            translation: Vec3::ZERO,
            rotation:    Quat::IDENTITY,
            scale:       Vec3::ZERO,
        }))?;
        let artifact = Artifact::new(&artifact_root);

        let script = Self {
            menu: Menu::new(),
            registry: ToolRegistry::new(),
            tools: Vec::new(),
            wheel: Wheel::new(),
            artifact,
            artifact_root,
            home: Home::default(),
            input: register_global_input_listener()?,
            camera: RefCell::new(None),
            placement: Cell::new(None),
            open_pos: Cell::new(None),
            open_t: Cell::new(0.0),
            art_t: Cell::new(0.0),
            hovered: Cell::new(None),
            pressed: Cell::new(false),
            render_time: SystemTime::now(),
        };
        // Warm the root menu's meshes during load so the first open animates
        // instead of popping in once the async mesh upload lands.
        script.wheel.rebuild(&script.menu.slots());
        Ok(script)
    }

    fn tick(&mut self) -> anyhow::Result<()> {
        self.poll_tools();
        self.home.tick();

        let Some(cam) = self.camera() else {
            return Ok(());
        };

        if self.menu.is_open()
            && let Some(open_pos) = self.open_pos.get()
        {
            let delta = cam.translation - open_pos;
            if delta.dot(delta) > CLOSE_MOVE_SQ {
                self.close();
            }
        }

        if self.menu.is_open()
            && let Some(plane) = self.placement.get()
        {
            let forward = cam.rotation * Vec3::new(0.0, 0.0, -1.0);
            self.hovered.set(hovered_sector(
                cam.translation,
                forward,
                plane.translation,
                plane.rotation,
                self.wheel.len(),
            ));
        }

        while let Some(event) = self.input.poll() {
            match event.action {
                InputAction::MenuDown => {
                    if !self.pressed.get() {
                        self.pressed.set(true);
                        if self.menu.is_open() {
                            println!("menu closed");
                            self.close();
                        } else {
                            println!("menu opened");
                            self.open(&cam);
                        }
                    }
                }
                InputAction::MenuUp => self.pressed.set(false),
                InputAction::GrabDown => {
                    if self.menu.is_open() {
                        if let Some(idx) = self.hovered.get() {
                            if let Some(slot) = self.menu.slots().get(idx) {
                                println!("selected '{}'", slot.label);
                            }
                            let outcome = self.menu.select(idx);
                            self.handle_outcome(outcome, &cam);
                        }
                    } else if let Some(doc) = self.menu.active_tool().cloned() {
                        println!("gauntlet: forwarding trigger down to active tool");
                        self.registry.trigger(&doc, true);
                    }
                }
                InputAction::GrabUp => {
                    if !self.menu.is_open()
                        && let Some(doc) = self.menu.active_tool().cloned()
                    {
                        self.registry.trigger(&doc, false);
                    }
                }
                InputAction::ScrollUp | InputAction::ScrollDown => {
                    if !self.menu.is_open()
                        && let Some(doc) = self.menu.active_tool().cloned()
                    {
                        let delta = if matches!(event.action, InputAction::ScrollUp) {
                            1.0
                        } else {
                            -1.0
                        };
                        self.registry.scroll(&doc, delta);
                    }
                }
            }
        }
        Ok(())
    }

    fn render(&mut self) -> anyhow::Result<()> {
        let delta = self.render_time.elapsed().expect("elapsed").as_secs_f32();
        self.render_time = SystemTime::now();

        let Some(cam) = self.camera() else {
            return Ok(());
        };

        let open_t = approach(self.open_t.get(), self.menu.is_open(), delta * OPEN_SPEED);
        self.open_t.set(open_t);

        if let Some(plane) = self.placement.get() {
            self.wheel.root.set_xform(Some(Xform {
                translation: plane.translation,
                rotation:    plane.rotation,
                scale:       Vec3::splat(open_t),
            }))?;
            if open_t > 0.0 {
                self.wheel.animate(delta, self.hovered.get());
            } else if !self.menu.is_open() {
                self.placement.set(None);
            }
        }

        let active = self.menu.active_tool().is_some();
        let art_t = approach(self.art_t.get(), active, delta * ART_SPEED);
        self.art_t.set(art_t);
        self.artifact_root.set_xform(Some(Xform {
            translation: cam.translation + cam.rotation * ARTIFACT_OFFSET,
            rotation:    cam.rotation,
            scale:       Vec3::splat(art_t),
        }))?;
        if art_t > 0.0 {
            self.artifact.animate(delta, art_t);
        }
        Ok(())
    }
}

fn approach(current: f32, toward_one: bool, step: f32) -> f32 {
    if toward_one {
        (current + step).min(1.0)
    } else {
        (current - step).max(0.0)
    }
}
