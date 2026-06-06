use std::{
    cell::{
        Cell,
        RefCell,
    },
    f32::consts::PI,
};

use wired_prelude::prelude::*;

use crate::{
    Color,
    ModuleRef,
    sector::{
        Sector,
        make_sectors,
    },
    unavi::vui_module::api::VuiModuleRegistry,
    wired::{
        agent::{
            api::{
                local_agent,
                local_camera,
            },
            types::BoneName,
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

pub const DEFAULT_AGENT_HEIGHT: f32 = 1.7;
pub const MODULE_FORWARD_DIST: f32 = 0.9;
pub const MODULE_HEIGHT_OFFSET: f32 = 0.08;

pub const BG_ALPHA_BASE: f32 = 0.4;
pub const BG_ALPHA_HOVER: f32 = 0.8;
pub const CLOSE_ON_MOVE_THRESHOLD_SQ: f32 = 0.04;
pub const ICON_R: f32 = f32::midpoint(SECTOR_INNER_R, RING_RADIUS);
pub const ICON_Z_OFFSET: f32 = 0.004;
pub const OPEN_SPEED_SECONDS: f32 = 0.18;
pub const OUTLINE_COLOR: Color = Color::WHITE;
pub const OUTLINE_WIDTH: f32 = 0.005;
pub const OUTLINE_Z: f32 = 0.001;
pub const RAISE_DIST: f32 = 0.015;
pub const RAISE_SPEED_SECONDS: f32 = 0.07;
pub const RING_RADIUS: f32 = 0.14;
pub const SECTOR_GAP_WORLD: f32 = 0.012;
pub const SECTOR_INNER_R: f32 = 0.03;
pub const SECTOR_SUBDIVISIONS: usize = 40;
pub const Z_OFFSET: f32 = -0.5;

const fn xform_full(translation: Vec3, rotation: Quat, scale: Vec3) -> Xform {
    Xform {
        translation,
        rotation,
        scale,
    }
}

const fn xform_scale(scale: Vec3) -> Xform {
    xform_full(Vec3::ZERO, Quat::IDENTITY, scale)
}

fn global_transform(prim: &Prim) -> Transform {
    prim.global_xform()
}

fn place_sector_transform(bone: &Prim) -> Transform {
    let tr = global_transform(bone);
    let forward = tr.rotation * Vec3::new(0.0, 0.0, -1.0);

    let fwd_len = forward.x.hypot(forward.z);
    let forward_h = if fwd_len > 1.0e-3 {
        Vec3::new(forward.x / fwd_len, 0.0, forward.z / fwd_len)
    } else {
        Vec3::new(0.0, 0.0, -1.0)
    };

    let agent = local_agent().expect("local_agent");

    let waist_y = agent
        .bone(BoneName::Hips)
        .map_or(1.0, |h| global_transform(&h).translation.y);

    let head_y = agent
        .bone(BoneName::Head)
        .map_or(DEFAULT_AGENT_HEIGHT, |h| global_transform(&h).translation.y);
    let foot_y = agent
        .bone(BoneName::LeftFoot)
        .map_or(0.0, |f| global_transform(&f).translation.y);
    let agent_height = (head_y - foot_y).max(0.5);
    let scale_f = agent_height / DEFAULT_AGENT_HEIGHT;

    let translation = Vec3 {
        x: forward_h.x.mul_add(MODULE_FORWARD_DIST, tr.translation.x),
        y: waist_y + MODULE_HEIGHT_OFFSET,
        z: forward_h.z.mul_add(MODULE_FORWARD_DIST, tr.translation.z),
    };

    let angle = forward_h.x.atan2(forward_h.z);
    let half = angle / 2.0;
    let rotation = Quat {
        x: 0.0,
        y: half.sin(),
        z: 0.0,
        w: half.cos(),
    };

    Transform {
        translation,
        rotation,
        scale: Vec3::splat(scale_f),
    }
}

pub enum Target {
    Bone(BoneName),
    Camera,
}

pub struct Gauntlet {
    pub bone:           RefCell<Option<Prim>>,
    pub core:           Prim,
    pub hovered_sector: Cell<Option<usize>>,
    pub sectors:        RefCell<Vec<Sector>>,
    pub open:           Cell<bool>,
    pub open_pos:       Cell<Option<Vec3>>,
    pub pressed:        Cell<bool>,
    pub scale_t:        Cell<f32>,
    pub target:         Target,
}

impl Gauntlet {
    pub fn new(target: Target) -> Self {
        let doc = self_document().expect("self_document");
        let core = doc.create_prim();
        core.set_xform(Some(xform_scale(Vec3::ZERO)));
        Self {
            bone: RefCell::new(None),
            core,
            hovered_sector: Cell::new(None),
            sectors: RefCell::new(Vec::new()),
            open: Cell::new(false),
            open_pos: Cell::new(None),
            pressed: Cell::new(false),
            scale_t: Cell::new(0.0),
            target,
        }
    }

    pub fn rebuild_sectors(&self, modules: &[ModuleRef], colors: &[Color]) {
        let doc = self_document().expect("self_document");

        for s in self.sectors.borrow().iter() {
            self.core.remove_child(&s.root);
            doc.remove_prim(&s.root);
        }

        let new_sectors = make_sectors(&doc, modules, colors);
        for s in &new_sectors {
            s.root.set_xform(Some(xform_scale(Vec3::ZERO)));
            self.core.add_child(&s.root);
        }

        *self.sectors.borrow_mut() = new_sectors;
    }

    pub fn lazy_init_bone(&self) -> bool {
        let mut bone_ref = self.bone.borrow_mut();
        if bone_ref.is_some() {
            return true;
        }

        let prim = match self.target {
            Target::Camera => Some(local_camera().expect("local_camera")),
            Target::Bone(b) => local_agent().expect("local_agent").bone(b),
        };

        prim.is_some_and(|prim| {
            *bone_ref = Some(prim);
            true
        })
    }

    pub fn track_bone(&self) {
        let bone_ref = self.bone.borrow();
        let Some(bone) = bone_ref.as_ref() else {
            return;
        };
        let tr = global_transform(bone);
        let pos = tr.translation + tr.rotation * Vec3::new(0.0, 0.0, Z_OFFSET);
        self.core.set_xform(Some(xform_full(
            pos,
            tr.rotation,
            Vec3::splat(self.scale_t.get()),
        )));
    }

    pub fn apply_scale(&self) {
        let cur = self.core.xform().unwrap_or(Xform {
            translation: Vec3::ZERO,
            rotation:    Quat::IDENTITY,
            scale:       Vec3::ONE,
        });
        self.core.set_xform(Some(Xform {
            translation: cur.translation,
            rotation:    cur.rotation,
            scale:       Vec3::splat(self.scale_t.get()),
        }));
    }

    pub fn open_menu(&self, open_pos: Vec3) {
        let bone_ref = self.bone.borrow();
        let Some(bone) = bone_ref.as_ref() else {
            return;
        };

        let tr = global_transform(bone);
        let translation = tr.translation + tr.rotation * Vec3::new(0.0, 0.0, Z_OFFSET);
        self.core
            .set_xform(Some(xform_full(translation, tr.rotation, Vec3::ZERO)));

        self.open_pos.set(Some(open_pos));
        let sectors = self.sectors.borrow();
        for sector in sectors.iter() {
            sector.raise_t.set(0.0);
            sector.root.set_xform(Some(xform_scale(Vec3::ONE)));
            let c = sector.bg_color;
            sector.set_bg_color(Color::rgba(c.r, c.g, c.b, BG_ALPHA_BASE));
        }
    }

    pub fn close_menu(&self) {
        self.hovered_sector.set(None);
        self.open_pos.set(None);
        let sectors = self.sectors.borrow();
        for sector in sectors.iter() {
            if sector.raise_t.get() != 0.0 {
                sector.raise_t.set(0.0);
                sector.root.set_xform(Some(xform_scale(Vec3::ONE)));
                let c = sector.bg_color;
                sector.set_bg_color(Color::rgba(c.r, c.g, c.b, BG_ALPHA_BASE));
            }
        }
    }

    pub fn select(&self, idx: usize, modules: &[ModuleRef], registry: &VuiModuleRegistry) {
        let sectors = self.sectors.borrow();
        let Some(sector) = sectors.get(idx) else {
            return;
        };
        let Some(module) = modules.get(idx) else {
            return;
        };
        if sector.active_state.get() {
            println!("Deactivated {}", sector.name);
            sector.active_state.set(false);
            sector.outline_prim.set_xform(Some(xform_scale(Vec3::ZERO)));
            registry.deactivate(&module.doc_id);
        } else {
            println!("Activated {}", sector.name);
            sector.active_state.set(true);
            sector.outline_prim.set_xform(Some(xform_scale(Vec3::ONE)));
            let bone_ref = self.bone.borrow();
            if let Some(bone) = bone_ref.as_ref() {
                let transform = place_sector_transform(bone);
                registry.activate(&module.doc_id, transform);
            }
        }
        self.open.set(false);
        drop(sectors);
        self.close_menu();
    }

    pub fn update_hovered_sector(&self) {
        let sectors = self.sectors.borrow();
        if !self.open.get() || sectors.is_empty() {
            if self.hovered_sector.get().is_some() {
                self.hovered_sector.set(None);
            }
            return;
        }

        let bone_ref = self.bone.borrow();
        let Some(bone) = bone_ref.as_ref() else {
            return;
        };

        let bone_tr = global_transform(bone);
        let menu_tr = global_transform(&self.core);

        let forward = bone_tr.rotation * Vec3::new(0.0, 0.0, -1.0);

        let menu_normal = menu_tr.rotation * Vec3::Z;
        let origin_to_menu = menu_tr.translation - bone_tr.translation;
        let denom = forward.dot(menu_normal);
        if denom.abs() < 1.0e-6 {
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
        let n = sectors.len();
        let sector = (angle * n as f32 / (2.0 * PI)).round() as usize % n;
        self.hovered_sector.set(Some(sector));
    }
}
