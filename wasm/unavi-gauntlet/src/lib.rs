use std::{cell::Cell, time::SystemTime};

use blake3::Hash;
use wired_prelude::{wired_math::types::Vec3, wired_scene::types::Color};

use crate::{
    gauntlet::{
        BG_ALPHA_BASE, BG_ALPHA_HOVER, CLOSE_ON_MOVE_THRESHOLD_SQ, Gauntlet, OPEN_SPEED_SECONDS,
        RAISE_DIST, RAISE_SPEED_SECONDS, Target,
    },
    unavi::vui_module::api::VuiModuleRegistry,
    wired::{
        agent::types::BoneName,
        input::{
            system_api::system_input_listener,
            types::{InputAction, InputDevice, InputListener},
        },
        scene::{context::get_document, types::Mesh},
    },
};

mod gauntlet;
mod sector;

wired_prelude::generate_script!(Script);

pub const MAX_MODULES: usize = 8;

fn palette(n: usize) -> Vec<Color> {
    (0..n)
        .map(|i| {
            let h = (0.6 + i as f32 / n as f32) % 1.0;
            Color::hsv(h, 0.75, 0.85)
        })
        .collect()
}

pub struct ModuleRef {
    pub doc_id: Vec<u8>,
    pub icon_mesh: Option<Mesh>,
    pub icon_mesh_id: String,
    pub name: String,
}

struct Script {
    gauntlets: [Gauntlet; 3],
    input: InputListener,
    render_time: Cell<SystemTime>,
    registry: VuiModuleRegistry,
    module_refs: std::cell::RefCell<Vec<ModuleRef>>,
}

impl GuestScript for Script {
    fn new() -> Self {
        let registry = VuiModuleRegistry::new();

        let gauntlets = [
            Target::Camera,
            Target::Bone(BoneName::LeftHand),
            Target::Bone(BoneName::RightHand),
        ]
        .map(Gauntlet::new);

        Self {
            gauntlets,
            input: system_input_listener(),
            render_time: Cell::new(SystemTime::now()),
            registry,
            module_refs: std::cell::RefCell::new(Vec::new()),
        }
    }

    #[expect(clippy::too_many_lines)]
    fn tick(&self) {
        let mut modules = self.module_refs.borrow_mut();
        let mut changed = false;

        for m in self.registry.poll() {
            if modules.len() < MAX_MODULES && !modules.iter().any(|d| d.doc_id == m.doc_id) {
                modules.push(ModuleRef {
                    doc_id: m.doc_id,
                    icon_mesh: None,
                    icon_mesh_id: m.icon_mesh_id,
                    name: m.name,
                });
            }
        }

        // Resolve icon meshes for modules that don't have one yet.
        for module in modules.iter_mut() {
            if module.icon_mesh.is_none() && !module.icon_mesh_id.is_empty() {
                if let Some(doc) = get_document(&module.doc_id) {
                    if let Some(mesh) = doc
                        .meshes()
                        .into_iter()
                        .find(|m| m.id() == module.icon_mesh_id)
                    {
                        module.icon_mesh = Some(mesh);
                        changed = true;
                    }
                } else if let Ok(doc_id) = Hash::from_slice(&module.doc_id) {
                    eprintln!("document {doc_id} not found");
                }
            }
        }

        if changed {
            modules.sort_by(|a, b| a.name.cmp(&b.name));
            let n = modules.len();
            for g in &self.gauntlets {
                g.rebuild_sectors(&modules, &palette(n));
                for s in g.sectors.borrow().as_slice() {
                    self.registry.set_color(&s.module_doc_id, s.bg_color);
                }
            }
        }
        drop(modules);

        if !self
            .gauntlets
            .iter()
            .all(gauntlet::Gauntlet::lazy_init_bone)
        {
            return;
        }

        let camera_pos: Option<Vec3> = {
            let bone_ref = self.gauntlets[0].bone.borrow();
            bone_ref.as_ref().map(|b| b.global_transform().translation)
        };

        if let Some(pos) = camera_pos {
            for g in &self.gauntlets {
                if let Some(open_pos) = g.open_pos.get() {
                    let delta = pos - open_pos;
                    if delta.dot(delta) > CLOSE_ON_MOVE_THRESHOLD_SQ {
                        g.open.set(false);
                        g.close_menu();
                    }
                }
            }
        }

        for g in &self.gauntlets {
            g.update_hovered_sector();
        }

        while let Some(event) = self.input.poll() {
            let menu_idx = match event.device {
                InputDevice::Keyboard => 0,
                InputDevice::LeftHand => 1,
                InputDevice::RightHand => 2,
            };

            match event.action {
                InputAction::MenuDown => {
                    let g = &self.gauntlets[menu_idx];
                    if !g.pressed.get() {
                        g.pressed.set(true);
                        if g.open.get() {
                            g.open.set(false);
                            g.close_menu();
                        } else {
                            g.open.set(true);
                            g.open_menu(camera_pos.unwrap_or(Vec3::ZERO));
                        }
                    }
                }
                InputAction::MenuUp => {
                    self.gauntlets[menu_idx].pressed.set(false);
                }
                InputAction::GrabDown => {
                    let modules = self.module_refs.borrow();
                    for g in &self.gauntlets {
                        let matches = matches!(
                            (&g.target, event.device),
                            (Target::Camera, _)
                                | (Target::Bone(BoneName::LeftHand), InputDevice::LeftHand)
                                | (Target::Bone(BoneName::RightHand), InputDevice::RightHand)
                        );
                        if matches
                            && g.open.get()
                            && let Some(sector) = g.hovered_sector.get()
                        {
                            g.select(sector, &modules, &self.registry);
                        }
                    }
                }
                InputAction::GrabUp => {}
            }
        }
    }

    fn render(&self) {
        let delta = self
            .render_time
            .get()
            .elapsed()
            .expect("elapsed")
            .as_secs_f32();
        self.render_time.set(SystemTime::now());

        for g in &self.gauntlets {
            let prev_t = g.scale_t.get();
            let inc = if g.open.get() {
                delta / OPEN_SPEED_SECONDS
            } else {
                -delta / OPEN_SPEED_SECONDS
            };
            let new_t = (prev_t + inc).clamp(0.0, 1.0);
            if new_t.to_bits() != prev_t.to_bits() {
                g.scale_t.set(new_t);
                g.core.set_scale(Vec3::splat(new_t));
            }

            if !g.open.get() && new_t > 0.0 {
                g.track_bone();
            }

            if !g.open.get() {
                continue;
            }

            let hovered = g.hovered_sector.get();
            let sectors = g.sectors.borrow();
            for (i, sector) in sectors.iter().enumerate() {
                let target_raise = if Some(i) == hovered { 1.0_f32 } else { 0.0_f32 };
                let prev_raise = sector.raise_t.get();
                let speed = delta / RAISE_SPEED_SECONDS;
                let new_raise = if target_raise > prev_raise {
                    (prev_raise + speed).min(target_raise)
                } else {
                    (prev_raise - speed).max(target_raise)
                };
                if new_raise.to_bits() != prev_raise.to_bits() {
                    sector.raise_t.set(new_raise);
                    sector
                        .root
                        .set_translation(Vec3::new(0.0, 0.0, new_raise * RAISE_DIST));
                    let c = sector.bg_color;
                    let bg_alpha = new_raise.mul_add(BG_ALPHA_HOVER - BG_ALPHA_BASE, BG_ALPHA_BASE);
                    sector
                        .bg_material
                        .set_base_color(Color::rgba(c.r, c.g, c.b, bg_alpha));
                }
            }
        }
    }

    fn drop(&self) {}
}
