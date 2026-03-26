use std::{
    cell::{Cell, RefCell},
    f32::consts::PI,
};

use wired_prelude::wired_math::types::{Quat, Vec3};

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

pub const DEFAULT_AGENT_HEIGHT: f32 = 1.7;
pub const MODULE_FORWARD_DIST: f32 = 0.9;
pub const MODULE_HEIGHT_OFFSET: f32 = 0.08;

pub const BG_ALPHA_BASE: f32 = 0.3;
pub const BG_ALPHA_HOVER: f32 = 0.9;
pub const ICON_Z: f32 = 0.006;
pub const OPEN_SPEED_SECONDS: f32 = 0.18;
pub const RAISE_DIST: f32 = 0.015;
pub const RAISE_SPEED_SECONDS: f32 = 0.07;
pub const RING_RADIUS: f32 = 0.14;
pub const CLOSE_ON_MOVE_THRESHOLD_SQ: f32 = 0.04;
pub const SECTOR_GAP_WORLD: f32 = 0.012;
pub const SECTOR_INNER_R: f32 = 0.03;
pub const OUTLINE_COLOR: [f32; 3] = [1.0, 1.0, 1.0];
pub const OUTLINE_WIDTH: f32 = 0.005;
pub const OUTLINE_Z: f32 = 0.001;
pub const SECTOR_SUBDIVISIONS: usize = 40;
pub const Z_OFFSET: f32 = -0.5;

fn place_module(root: &Node, bone: &Node) {
    let tr = bone.global_transform();
    let forward = tr.rotation * Vec3::new(0.0, 0.0, -1.0);

    // Horizontal forward (XZ only), normalised.
    let fwd_len = forward.x.hypot(forward.z);
    let forward_h = if fwd_len > 1e-3 {
        Vec3::new(forward.x / fwd_len, 0.0, forward.z / fwd_len)
    } else {
        Vec3::new(0.0, 0.0, -1.0)
    };

    let agent = local_agent();

    // Waist height from Hips bone.
    let waist_y = agent
        .bone(BoneName::Hips)
        .map_or(1.0, |h| h.global_transform().translation.y);

    // Scale from agent height (Head Y − LeftFoot Y).
    let head_y = agent
        .bone(BoneName::Head)
        .map_or(DEFAULT_AGENT_HEIGHT, |h| h.global_transform().translation.y);
    let foot_y = agent
        .bone(BoneName::LeftFoot)
        .map_or(0.0, |f| f.global_transform().translation.y);
    let agent_height = (head_y - foot_y).max(0.5);
    let scale = agent_height / DEFAULT_AGENT_HEIGHT;

    // World position: in front of agent at waist height.
    let pos = Vec3 {
        x: forward_h.x.mul_add(MODULE_FORWARD_DIST, tr.translation.x),
        y: waist_y + MODULE_HEIGHT_OFFSET,
        z: forward_h.z.mul_add(MODULE_FORWARD_DIST, tr.translation.z),
    };

    // Yaw rotation so local -Z faces the agent.
    // Rotating local Z → forward_h makes local -Z → agent.
    let angle = forward_h.x.atan2(forward_h.z);
    let half = angle / 2.0;
    let rotation = Quat {
        x: 0.0,
        y: half.sin(),
        z: 0.0,
        w: half.cos(),
    };

    root.set_translation(pos);
    root.set_rotation(rotation);
    root.set_scale(Vec3 {
        x: scale,
        y: scale,
        z: scale,
    });
}

pub enum Target {
    Bone(BoneName),
    Camera,
}

pub struct Gauntlet {
    pub bone: RefCell<Option<Node>>,
    pub core: Node,
    pub hovered_sector: Cell<Option<usize>>,
    pub modules: Vec<Module>,
    pub open: Cell<bool>,
    pub open_pos: Cell<Option<Vec3>>,
    pub pressed: Cell<bool>,
    /// Logical scale [0.0, 1.0] tracked on the guest side to avoid redundant host calls.
    pub scale_t: Cell<f32>,
    pub target: Target,
}

impl Gauntlet {
    pub const fn new(core: Node, target: Target, modules: Vec<Module>) -> Self {
        Self {
            bone: RefCell::new(None),
            core,
            hovered_sector: Cell::new(None),
            modules,
            open: Cell::new(false),
            open_pos: Cell::new(None),
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
            *bone_ref = Some(node);
            true
        })
    }

    /// Position core and add root nodes to it.
    pub fn open_menu(&self, open_pos: Vec3) {
        let bone_ref = self.bone.borrow();
        let Some(bone) = bone_ref.as_ref() else {
            return;
        };

        let mut tr = bone.global_transform();
        tr.translation += tr.rotation * Vec3::new(0.0, 0.0, Z_OFFSET);
        tr.scale = Vec3::ZERO;
        self.core.set_transform(tr);

        self.open_pos.set(Some(open_pos));
        for module in &self.modules {
            module.raise_t.set(0.0);
            module.root.set_scale(Vec3::ONE);
            module.root.set_translation(Vec3::ZERO);
        }
    }

    /// Hide root nodes without detaching them.
    pub fn close_menu(&self) {
        self.hovered_sector.set(None);
        self.open_pos.set(None);
        for module in &self.modules {
            if module.raise_t.get() != 0.0 {
                module.raise_t.set(0.0);
                module.root.set_translation(Vec3::ZERO);
                let c = module.bg_color;
                module
                    .bg_material
                    .set_base_color(&[c[0], c[1], c[2], BG_ALPHA_BASE]);
            }
        }
    }

    /// Toggle a module's active state by sector index.
    pub fn select(&self, sector: usize) {
        let module = &self.modules[sector];
        if module.active_state.get() {
            println!("deactivated {}", module.name);
            module.active_state.set(false);
            module.active.deactivate();
            module.outline_node.set_scale(Vec3::ZERO);
        } else {
            println!("activated {}", module.name);
            module.active_state.set(true);
            module.active.activate();
            module.outline_node.set_scale(Vec3::ONE);
            // Override position / rotation / scale with computed placement.
            let bone_ref = self.bone.borrow();
            if let Some(bone) = bone_ref.as_ref() {
                place_module(module.active.root(), bone);
                // Place the nav ring above the basin (left side of the nav root).
                if let crate::modules::ModuleActive::Nav(nav) = &module.active {
                    let root_tr = module.active.root().global_transform();
                    nav.ring.set_translation(Vec3 {
                        x: root_tr.translation.x
                            - crate::modules::nav::BASIN_X,
                        y: root_tr.translation.y + 0.5,
                        z: root_tr.translation.z,
                    });
                }
            }
        }
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

        // Ray-plane intersection: find where the look direction hits the menu plane.
        let menu_normal = menu_tr.rotation * Vec3::Z;
        let origin_to_menu = menu_tr.translation - bone_tr.translation;
        let denom = forward.dot(menu_normal);
        if denom.abs() < 1e-6 {
            self.hovered_sector.set(None);
            return;
        }
        let t = origin_to_menu.dot(menu_normal) / denom;
        if t < 0.0 {
            self.hovered_sector.set(None);
            return;
        }
        let cursor_rel = bone_tr.translation + forward * t - menu_tr.translation;

        let right = menu_tr.rotation * Vec3::X;
        let up = menu_tr.rotation * Vec3::Y;
        let x = cursor_rel.dot(right);
        let y = cursor_rel.dot(up);
        let dist = x.hypot(y);

        if dist < SECTOR_INNER_R {
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
