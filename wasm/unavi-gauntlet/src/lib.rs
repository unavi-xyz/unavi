mod gauntlet;
mod module;

use std::{cell::Cell, time::SystemTime};

use wired_prelude::wired_math::types::Vec3;

use crate::{
    gauntlet::{
        BG_ALPHA_BASE, BG_ALPHA_HOVER, CLOSE_ON_MOVE_THRESHOLD_SQ, Gauntlet, OPEN_SPEED_SECONDS,
        RAISE_DIST, RAISE_SPEED_SECONDS, Target,
    },
    wired::{
        agent::types::BoneName,
        event::{
            api::{register_emitter, register_receptor},
            types::{EventEmitter, EventReceptor},
        },
        input::{
            system_api::system_input_listener,
            types::{InputAction, InputDevice, InputListener},
        },
    },
};

wired_prelude::generate_script!(Script);

pub const MAX_MODULES: usize = 8;

pub const MODULE_PALETTE: [[f32; 3]; MAX_MODULES] = [
    [0.52, 0.20, 0.82],
    [0.88, 0.52, 0.08],
    [0.12, 0.40, 0.88],
    [0.12, 0.62, 0.28],
    [0.72, 0.12, 0.52],
    [0.80, 0.15, 0.18],
    [0.08, 0.58, 0.55],
    [0.50, 0.65, 0.08],
];

const CH_REGISTER: &str = "unavi::gauntlet::register";
const CH_REGISTER_REQUEST: &str = "unavi::gauntlet::register-request";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct RegisterPayload {
    pub name: String,
    pub icon: String,
    pub color: [f32; 3],
}

pub struct DynamicModuleDef {
    pub name: String,
    #[allow(dead_code)]
    pub icon: String,
    pub color: [f32; 3],
    pub doc_id: Vec<u8>,
}

struct Script {
    gauntlets: [Gauntlet; 3],
    input: InputListener,
    render_time: Cell<SystemTime>,
    _emitter: EventEmitter,
    register_receptor: EventReceptor,
    module_defs: std::cell::RefCell<Vec<DynamicModuleDef>>,
}

impl GuestScript for Script {
    fn new() -> Self {
        // Broadcast emitter for register-request
        let emitter = register_emitter(None, f32::MAX, &[]);
        let register_receptor = register_receptor(&[CH_REGISTER.to_string()], None, f32::MAX, &[]);

        // Announce ourselves so modules can discover us
        emitter.emit(CH_REGISTER_REQUEST, &[]);

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
            _emitter: emitter,
            register_receptor,
            module_defs: std::cell::RefCell::new(Vec::new()),
        }
    }

    fn tick(&self) {
        // Drain register events — add new module defs and rebuild ring.
        let mut defs = self.module_defs.borrow_mut();
        let mut changed = false;
        while let Some(event) = self.register_receptor.poll() {
            if let Some(def) = parse_register_payload(&event.payload, event.sender_document)
                && defs.len() < MAX_MODULES
            {
                defs.push(def);
                changed = true;
            }
        }
        if changed {
            let n = defs.len();
            for g in &self.gauntlets {
                g.rebuild_modules(&defs, &MODULE_PALETTE[..n]);
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
                            g.select(sector, &defs);
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
                        .set_base_color(&[c[0], c[1], c[2], bg_alpha]);
                }
            }
        }
    }

    fn drop(&self) {}
}

fn parse_register_payload(payload: &[u8], sender_document: Vec<u8>) -> Option<DynamicModuleDef> {
    let reg: RegisterPayload = postcard::from_bytes(payload).ok()?;
    Some(DynamicModuleDef {
        name: reg.name,
        icon: reg.icon,
        color: reg.color,
        doc_id: sender_document,
    })
}
