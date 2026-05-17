use std::time::SystemTime;

use wired_prelude::prelude::*;

use crate::{
    gauntlet::{
        BG_ALPHA_BASE, BG_ALPHA_HOVER, CLOSE_ON_MOVE_THRESHOLD_SQ, Gauntlet, OPEN_SPEED_SECONDS,
        RAISE_DIST, RAISE_SPEED_SECONDS, Target,
    },
    unavi::vui_module::api::VuiModuleRegistry,
    wired::{
        agent::types::BoneName,
        input::{
            context::register_global_input_listener,
            types::{InputAction, InputDevice, InputListener},
        },
        scene::types::Xform,
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
    pub icon_prim_id: Option<String>,
    pub name: String,
}

struct Script {
    gauntlets: [Gauntlet; 3],
    input: InputListener,
    module_refs: Vec<ModuleRef>,
    registry: VuiModuleRegistry,
    render_time: SystemTime,
}

impl ScriptBehavior for Script {
    fn init() -> Self {
        let registry = VuiModuleRegistry::new();

        let gauntlets = [
            Target::Camera,
            Target::Bone(BoneName::LeftHand),
            Target::Bone(BoneName::RightHand),
        ]
        .map(Gauntlet::new);

        Self {
            gauntlets,
            input: register_global_input_listener(),
            module_refs: Vec::new(),
            registry,
            render_time: SystemTime::now(),
        }
    }

    #[expect(clippy::too_many_lines)]
    fn tick(&mut self) {
        let mut changed = false;

        for m in self.registry.poll() {
            println!("Found module: {}", m.name);
            if self.module_refs.len() < MAX_MODULES
                && !self.module_refs.iter().any(|d| d.doc_id == m.doc_id)
            {
                self.module_refs.push(ModuleRef {
                    doc_id: m.doc_id,
                    icon_prim_id: if m.icon_prim_id.is_empty() {
                        None
                    } else {
                        Some(m.icon_prim_id)
                    },
                    name: m.name,
                });
                changed = true;
            }
        }

        if changed {
            self.module_refs.sort_by(|a, b| a.name.cmp(&b.name));
            let n = self.module_refs.len();
            for g in &self.gauntlets {
                g.rebuild_sectors(&self.module_refs, &palette(n));
                for s in g.sectors.borrow().as_slice() {
                    self.registry.set_color(&s.module_doc_id, s.bg_color);
                }
            }
        }

        if !self
            .gauntlets
            .iter()
            .all(gauntlet::Gauntlet::lazy_init_bone)
        {
            return;
        }

        let camera_pos: Option<Vec3> = {
            let bone_ref = self.gauntlets[0].bone.borrow();
            bone_ref.as_ref().map(|b| b.global_xform().translation)
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
                            g.select(sector, &self.module_refs, &self.registry);
                        }
                    }
                }
                InputAction::GrabUp => {}
            }
        }
    }

    fn render(&mut self) {
        let delta = self.render_time.elapsed().expect("elapsed").as_secs_f32();
        self.render_time = SystemTime::now();

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
                g.core.set_xform(Some(Xform {
                    translation: Vec3::ZERO,
                    rotation: Quat::IDENTITY,
                    scale: Vec3::splat(new_t),
                }));
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
                    sector.root.set_xform(Some(Xform {
                        translation: Vec3::new(0.0, 0.0, new_raise * RAISE_DIST),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::ONE,
                    }));
                    let c = sector.bg_color;
                    let bg_alpha = new_raise.mul_add(BG_ALPHA_HOVER - BG_ALPHA_BASE, BG_ALPHA_BASE);
                    sector.set_bg_color(Color::rgba(c.r, c.g, c.b, bg_alpha));
                }
            }
        }
    }
}
