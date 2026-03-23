use std::{
    cell::{Cell, RefCell},
    f32::consts::PI,
};

use wired_prelude::wired_math::types::Vec3;

use crate::{
    module::Module,
    wired::{
        agent::{
            context::{local_agent, local_camera},
            types::BoneName,
        },
        scene::types::Node,
    },
};

pub const CURSOR_PROJ_DIST: f32 = 0.3;
pub const DEAD_ZONE_RADIUS: f32 = 0.03;
pub const OPEN_SPEED_SECONDS: f32 = 0.05;
pub const RING_RADIUS: f32 = 0.2;
pub const Z_OFFSET: f32 = -0.5;

pub enum Target {
    Bone(BoneName),
    Camera,
}

pub struct Gauntlet {
    pub active_idx: Cell<Option<usize>>,
    pub bone: RefCell<Option<Node>>,
    pub core: Node,
    pub hovered_sector: Cell<Option<usize>>,
    pub modules: Vec<Module>,
    pub open: Cell<bool>,
    pub pressed: Cell<bool>,
    /// Logical scale [0.0, 1.0] tracked on the guest side to avoid redundant host calls.
    pub scale_t: Cell<f32>,
    pub target: Target,
}

impl Gauntlet {
    pub const fn new(core: Node, target: Target, modules: Vec<Module>) -> Self {
        Self {
            active_idx: Cell::new(None),
            bone: RefCell::new(None),
            core,
            hovered_sector: Cell::new(None),
            modules,
            open: Cell::new(false),
            pressed: Cell::new(false),
            scale_t: Cell::new(0.0),
            target,
        }
    }

    /// Lazily resolves the bone node and attaches active module nodes.
    /// Returns `true` once the bone is available.
    pub fn lazy_init_bone(&self) -> bool {
        let mut bone_ref = self.bone.borrow_mut();
        if bone_ref.is_some() {
            return true;
        }

        let node = match self.target {
            Target::Camera => Some(local_camera()),
            Target::Bone(b) => local_agent().bone(b),
        };

        node.is_some_and(|node| {
            for module in &self.modules {
                node.add_child(&module.active);
                module.active.set_translation(Vec3::new(0.0, 0.1, -0.05));
            }
            *bone_ref = Some(node);
            true
        })
    }

    /// Position core and lay out icon nodes in a ring.
    pub fn open_menu(&self) {
        let bone_ref = self.bone.borrow();
        let Some(bone) = bone_ref.as_ref() else {
            return;
        };

        let mut tr = bone.global_transform();
        tr.translation += tr.rotation * Vec3::new(0.0, 0.0, Z_OFFSET);
        tr.scale = Vec3::ZERO;
        self.core.set_transform(tr);

        let n = self.modules.len();
        for (i, module) in self.modules.iter().enumerate() {
            let angle = i as f32 * 2.0 * PI / n as f32;
            let local = Vec3::new(RING_RADIUS * angle.cos(), RING_RADIUS * angle.sin(), 0.0);
            self.core.add_child(&module.icon);
            module.icon.set_translation(local);
            module.icon.set_scale(Vec3::ONE);
        }
    }

    /// Remove icon nodes from core, resetting their scale.
    pub fn close_menu(&self) {
        for module in &self.modules {
            module.icon.set_scale(Vec3::ZERO);
            self.core.remove_child(&module.icon);
        }
    }

    /// Select a module by sector index: show its active node, close the menu.
    pub fn select(&self, sector: usize) {
        println!("selected module {sector}");
        if let Some(prev) = self.active_idx.get() {
            self.modules[prev].active.set_scale(Vec3::ZERO);
        }
        self.modules[sector].active.set_scale(Vec3::ONE);
        self.active_idx.set(Some(sector));
        self.open.set(false);
        self.close_menu();
    }

    /// Compute which sector the gauntlet is currently pointing at.
    pub fn update_hovered_sector(&self) {
        if !self.open.get() || self.modules.is_empty() {
            if self.hovered_sector.get().is_some() {
                self.hovered_sector.set(None);
            }
            return;
        }

        let bone_ref = self.bone.borrow();
        let Some(bone) = bone_ref.as_ref() else {
            return;
        };

        let bone_tr = bone.global_transform();
        let menu_tr = self.core.global_transform();

        let forward = bone_tr.rotation * Vec3::new(0.0, 0.0, -1.0);
        let cursor = bone_tr.translation + forward * CURSOR_PROJ_DIST;
        let cursor_rel = cursor - menu_tr.translation;

        let right = menu_tr.rotation * Vec3::X;
        let up = menu_tr.rotation * Vec3::Y;
        let x = cursor_rel.dot(right);
        let y = cursor_rel.dot(up);
        let dist = x.hypot(y);

        if dist < DEAD_ZONE_RADIUS {
            self.hovered_sector.set(None);
            return;
        }

        let angle = y.atan2(x);
        let angle = if angle < 0.0 {
            2.0f32.mul_add(PI, angle)
        } else {
            angle
        };
        let n = self.modules.len();
        let sector = (angle * n as f32 / (2.0 * PI)).round() as usize % n;
        self.hovered_sector.set(Some(sector));
    }
}
