mod gauntlet;
mod module;

use std::{cell::Cell, time::SystemTime};

use wired_prelude::wired_math::types::Vec3;

use crate::{
    gauntlet::{
        BG_ALPHA_BASE, BG_ALPHA_HOVER, CLOSE_ON_MOVE_THRESHOLD_SQ, Gauntlet, OPEN_SPEED_SECONDS,
        RAISE_DIST, RAISE_SPEED_SECONDS, Target,
    },
    module::{ModuleDef, ModuleKind},
    wired::{
        agent::types::BoneName,
        input::{
            system_api::system_input_listener,
            types::{InputAction, InputDevice, InputListener},
        },
    },
};

wired_prelude::generate_script!(Script);

const MODULE_DEFS: [ModuleDef; 3] = [
    ModuleDef {
        kind: ModuleKind::Config,
        name: "Config",
        rgb: [0.60, 0.35, 0.75],
    },
    ModuleDef {
        kind: ModuleKind::Inventory,
        name: "Inventory",
        rgb: [0.35, 0.75, 0.40],
    },
    ModuleDef {
        kind: ModuleKind::Nav,
        name: "Nav",
        rgb: [0.30, 0.55, 0.90],
    },
];

struct Script {
    gauntlets: [Gauntlet; 3],
    input: InputListener,
    render_time: Cell<SystemTime>,
}

impl GuestScript for Script {
    fn new() -> Self {
        use crate::wired::scene::context::self_document;

        let doc = self_document();

        let gauntlets = [
            Target::Camera,
            Target::Bone(BoneName::LeftHand),
            Target::Bone(BoneName::RightHand),
        ]
        .map(|target| {
            let modules = module::make_modules(&MODULE_DEFS);
            let core = doc.create_node();
            core.set_scale(Vec3::ZERO);
            for m in &modules {
                m.root.set_scale(Vec3::ZERO);
                core.add_child(&m.root);
            }
            Gauntlet::new(core, target, modules)
        });

        Self {
            gauntlets,
            input: system_input_listener(),
            render_time: Cell::new(SystemTime::now()),
        }
    }

    fn tick(&self) {
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
                            g.select(sector);
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
            for (i, module) in g.modules.iter().enumerate() {
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
