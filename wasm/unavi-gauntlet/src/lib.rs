mod gauntlet;
mod module;

use std::{cell::Cell, time::SystemTime};

use wired_prelude::wired_math::types::Vec3;

use crate::{
    gauntlet::{Gauntlet, OPEN_SPEED_SECONDS, Target},
    wired::{
        agent::types::BoneName,
        input::{
            system_api::system_input_listener,
            types::{InputAction, InputDevice, InputListener},
        },
    },
};

wired_prelude::generate_script!(Script);

const MODULES_PER_GAUNTLET: usize = 2;

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
            (Target::Camera, [0.8, 0.9, 1.0, 0.7]),
            (Target::Bone(BoneName::LeftHand), [0.9, 0.8, 1.0, 0.7]),
            (Target::Bone(BoneName::RightHand), [1.0, 0.8, 0.9, 0.7]),
        ]
        .map(|(target, base_color)| {
            let modules = module::make_modules(base_color, MODULES_PER_GAUNTLET);
            let core = doc.create_node();
            core.set_scale(Vec3::ZERO);
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
                            g.open_menu();
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
                if Some(i) == hovered {
                    module.material.set_base_color(&[1.0, 1.0, 1.0, 1.0]);
                    module.icon.set_scale(Vec3::splat(1.5));
                } else {
                    module.material.set_base_color(&module.color);
                    module.icon.set_scale(Vec3::ONE);
                }
            }
        }
    }

    fn drop(&self) {}
}
