mod gauntlet;
mod module;

use std::{cell::Cell, f32::consts::PI, time::SystemTime};

use blake3::Hash;
use wired_prelude::{wired_math::types::Vec3, wired_scene::types::Color};

use crate::{
    gauntlet::{
        BG_ALPHA_BASE, BG_ALPHA_HOVER, CLOSE_ON_MOVE_THRESHOLD_SQ, Gauntlet, OPEN_SPEED_SECONDS,
        RAISE_DIST, RAISE_SPEED_SECONDS, RING_RADIUS, SECTOR_INNER_R, Target,
    },
    unavi::vui_module::discovery::ModuleDiscovery,
    wired::{
        agent::types::BoneName,
        input::{
            system_api::system_input_listener,
            types::{InputAction, InputDevice, InputListener},
        },
        scene::{context::get_document, types::Node},
    },
};

wired_prelude::generate_script!(Script);

pub const MAX_MODULES: usize = 8;

const ICON_R: f32 = f32::midpoint(SECTOR_INNER_R, RING_RADIUS);
const ICON_SCALE: f32 = 1.0;
const ICON_Z_OFFSET: f32 = 0.004;

pub const MODULE_PALETTE: [Color; MAX_MODULES] = [
    Color::rgb(0.52, 0.20, 0.82),
    Color::rgb(0.88, 0.52, 0.08),
    Color::rgb(0.12, 0.40, 0.88),
    Color::rgb(0.12, 0.62, 0.28),
    Color::rgb(0.72, 0.12, 0.52),
    Color::rgb(0.80, 0.15, 0.18),
    Color::rgb(0.08, 0.58, 0.55),
    Color::rgb(0.50, 0.65, 0.08),
];

pub struct DynamicModuleDef {
    pub color: Color,
    pub doc_id: Vec<u8>,
    pub icon_node: Option<Node>,
    pub icon_node_id: String,
    pub name: String,
}

struct Script {
    gauntlets: [Gauntlet; 3],
    input: InputListener,
    render_time: Cell<SystemTime>,
    discovery: ModuleDiscovery,
    module_defs: std::cell::RefCell<Vec<DynamicModuleDef>>,
}

impl GuestScript for Script {
    fn new() -> Self {
        let discovery = ModuleDiscovery::new();

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
            discovery,
            module_defs: std::cell::RefCell::new(Vec::new()),
        }
    }

    fn tick(&self) {
        let mut defs = self.module_defs.borrow_mut();
        let mut changed = false;
        for m in self.discovery.poll() {
            if defs.len() < MAX_MODULES && !defs.iter().any(|d| d.doc_id == m.doc_id) {
                defs.push(DynamicModuleDef {
                    color: m.color,
                    doc_id: m.doc_id,
                    icon_node: None,
                    icon_node_id: m.icon_node_id,
                    name: m.name,
                });
                changed = true;
            }
        }
        if changed {
            defs.sort_by(|a, b| a.name.cmp(&b.name));
            let n = defs.len();
            for g in &self.gauntlets {
                g.rebuild_modules(&defs, &MODULE_PALETTE[..n]);
            }
        }
        // Look up icon nodes for any def that hasn't resolved one yet.
        for def in defs.iter_mut() {
            if def.icon_node.is_none() && !def.icon_node_id.is_empty() {
                if let Some(doc) = get_document(&def.doc_id) {
                    def.icon_node = doc.nodes().into_iter().find(|n| n.id() == def.icon_node_id);
                } else if let Ok(doc_id) = Hash::from_slice(&def.doc_id) {
                    eprintln!("document {doc_id} not found");
                }
            }
        }
        drop(defs);

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
                    let defs = self.module_defs.borrow();
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
                            g.select(sector, &defs, &self.discovery);
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

        let defs = self.module_defs.borrow();
        let n = defs.len();

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

            // Position module icon nodes in world space.
            if n > 0 {
                let menu_tr = g.core.global_transform();
                for (i, def) in defs.iter().enumerate() {
                    let Some(icon) = def.icon_node.as_ref() else {
                        continue;
                    };
                    if new_t == 0.0 {
                        icon.set_scale(Vec3::ZERO);
                    } else {
                        let ca = i as f32 * 2.0 * PI / n as f32;
                        let local = Vec3::new(ICON_R * ca.cos(), ICON_R * ca.sin(), ICON_Z_OFFSET);
                        let rot = menu_tr.rotation;
                        let offset = rot * local;
                        icon.set_translation(menu_tr.translation + offset);
                        icon.set_rotation(rot);
                        icon.set_scale(Vec3::splat(new_t * ICON_SCALE));
                    }
                }
            }

            if !g.open.get() {
                continue;
            }

            let hovered = g.hovered_sector.get();
            let modules = g.modules.borrow();
            for (i, module) in modules.iter().enumerate() {
                let target_raise = if Some(i) == hovered { 1.0_f32 } else { 0.0_f32 };
                let prev_raise = module.raise_t.get();
                let speed = delta / RAISE_SPEED_SECONDS;
                let new_raise = if target_raise > prev_raise {
                    (prev_raise + speed).min(target_raise)
                } else {
                    (prev_raise - speed).max(target_raise)
                };
                if new_raise.to_bits() != prev_raise.to_bits() {
                    module.raise_t.set(new_raise);
                    module
                        .root
                        .set_translation(Vec3::new(0.0, 0.0, new_raise * RAISE_DIST));
                    let c = module.bg_color;
                    let bg_alpha = new_raise.mul_add(BG_ALPHA_HOVER - BG_ALPHA_BASE, BG_ALPHA_BASE);
                    module
                        .bg_material
                        .set_base_color(Color::rgba(c.r, c.g, c.b, bg_alpha));
                }
            }
        }
    }

    fn drop(&self) {}
}
