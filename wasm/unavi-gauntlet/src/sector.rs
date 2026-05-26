use std::{cell::Cell, f32::consts::PI};

use wired_prelude::prelude::*;

use crate::{
    Color, ModuleRef,
    gauntlet::{
        BG_ALPHA_BASE, ICON_R, ICON_Z_OFFSET, OUTLINE_COLOR, OUTLINE_WIDTH, OUTLINE_Z, RING_RADIUS,
        SECTOR_GAP_WORLD, SECTOR_INNER_R, SECTOR_SUBDIVISIONS,
    },
    wired::scene::{
        api::get_document,
        types::{AlphaMode, Document, Material, Prim, Xform},
    },
};

const IDENTITY_QUAT: Quat = Quat::IDENTITY;

pub struct Sector {
    pub module_doc_id: Vec<u8>,
    pub active_state: Cell<bool>,
    pub bg_color: Color,
    pub bg_material: std::cell::RefCell<Material>,
    pub bg_prim: Prim,
    pub name: String,
    pub outline_prim: Prim,
    pub raise_t: Cell<f32>,
    pub root: Prim,
    icon_prim: Prim,
    remote_icon: Option<Prim>,
}

impl Sector {
    pub fn set_bg_color(&self, color: Color) {
        let mut mat = self.bg_material.borrow_mut();
        mat.base_color = Some(color);
        self.bg_prim.set_material(Some(&mat));
    }

    /// Mirror the local icon prim's global transform onto the remote prim so
    /// the module's own icon appears at the correct world-space position.
    pub fn sync_remote_icon(&self) {
        let Some(remote) = &self.remote_icon else {
            return;
        };
        let g = self.icon_prim.global_xform();
        remote.set_xform(Some(Xform {
            translation: g.translation,
            rotation: g.rotation,
            scale: g.scale,
        }));
    }
}

pub fn make_sectors(doc: &Document, modules: &[ModuleRef], colors: &[Color]) -> Vec<Sector> {
    let n = modules.len();
    modules
        .iter()
        .enumerate()
        .map(|(i, module)| make_sector(doc, i, n, module, colors[i]))
        .collect()
}

const fn translation(translation: Vec3) -> Xform {
    Xform {
        translation,
        rotation: IDENTITY_QUAT,
        scale: Vec3::ONE,
    }
}

const fn scale(scale: Vec3) -> Xform {
    Xform {
        translation: Vec3::ZERO,
        rotation: IDENTITY_QUAT,
        scale,
    }
}

fn make_sector(doc: &Document, i: usize, n: usize, module: &ModuleRef, color: Color) -> Sector {
    let bg_material = Material {
        alpha_cutoff: None,
        alpha_mode: Some(AlphaMode::Blend),
        base_color: Some(Color::rgba(color.r, color.g, color.b, BG_ALPHA_BASE)),
        base_color_texture: None,
        double_sided: Some(true),
        emissive: None,
        emissive_texture: None,
        metallic: None,
        metallic_roughness_texture: None,
        normal_texture: None,
        occlusion_texture: None,
        roughness: None,
    };

    let bg = make_sector_prim(doc, i, n);
    bg.set_material(Some(&bg_material));

    let outline_mat = Material {
        alpha_cutoff: None,
        alpha_mode: None,
        base_color: Some(OUTLINE_COLOR),
        base_color_texture: None,
        double_sided: Some(true),
        emissive: None,
        emissive_texture: None,
        metallic: None,
        metallic_roughness_texture: None,
        normal_texture: None,
        occlusion_texture: None,
        roughness: None,
    };
    let outline = make_outline_prim(doc, i, n);
    outline.set_material(Some(&outline_mat));
    outline.set_xform(Some(scale(Vec3::ZERO)));

    let ca = i as f32 * 2.0 * PI / n as f32;
    let icon = doc.create_prim();
    icon.set_xform(Some(translation(Vec3::new(
        ICON_R * ca.cos(),
        ICON_R * ca.sin(),
        ICON_Z_OFFSET,
    ))));

    let remote_icon = module.icon_prim_id.as_deref().and_then(|prim_id| {
        let remote_doc = get_document(&module.doc_id)?;
        remote_doc.get_prim(prim_id)
    });

    let root = doc.create_prim();
    root.add_child(&bg);
    root.add_child(&outline);
    root.add_child(&icon);

    Sector {
        module_doc_id: module.doc_id.clone(),
        active_state: Cell::new(false),
        bg_color: color,
        bg_material: std::cell::RefCell::new(bg_material),
        bg_prim: bg,
        name: module.name.clone(),
        outline_prim: outline,
        raise_t: Cell::new(0.0),
        root,
        icon_prim: icon,
        remote_icon,
    }
}

fn make_outline_prim(doc: &Document, i: usize, n: usize) -> Prim {
    let half_span = PI / n as f32;
    let center_angle = i as f32 * 2.0 * PI / n as f32;
    let subs = SECTOR_SUBDIVISIONS;

    let mut positions: Vec<f32> = Vec::with_capacity(3 * 2 * (subs + 1));
    let mut normals: Vec<f32> = Vec::with_capacity(3 * 2 * (subs + 1));
    let mut indices: Vec<u32> = Vec::with_capacity(6 * subs);

    for r in [RING_RADIUS, RING_RADIUS + OUTLINE_WIDTH] {
        let half_gap = SECTOR_GAP_WORLD / (2.0 * r);
        let start = center_angle - half_span + half_gap;
        let end = center_angle + half_span - half_gap;
        for j in 0..=subs {
            let t = j as f32 / subs as f32;
            let a = t.mul_add(end - start, start);
            positions.extend_from_slice(&[r * a.cos(), r * a.sin(), OUTLINE_Z]);
            normals.extend_from_slice(&[0.0, 0.0, 1.0]);
        }
    }
    for j in 0..subs as u32 {
        let i0 = j;
        let i1 = j + 1;
        let i2 = subs as u32 + 1 + j;
        let i3 = subs as u32 + 1 + j + 1;
        indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
    }

    let prim = doc.create_prim();
    prim.set_mesh_stream("POSITION", Some(&positions));
    prim.set_mesh_stream("NORMAL", Some(&normals));
    prim.set_mesh_indices_u32(Some(&indices));
    prim
}

fn make_sector_prim(doc: &Document, i: usize, n: usize) -> Prim {
    let half_span = PI / n as f32;
    let center_angle = i as f32 * 2.0 * PI / n as f32;
    let subs = SECTOR_SUBDIVISIONS;

    let mut positions: Vec<f32> = Vec::with_capacity(3 * 2 * (subs + 1));
    let mut normals: Vec<f32> = Vec::with_capacity(3 * 2 * (subs + 1));
    let mut indices: Vec<u32> = Vec::with_capacity(6 * subs);

    for r in [SECTOR_INNER_R, RING_RADIUS] {
        let half_gap = SECTOR_GAP_WORLD / (2.0 * r);
        let start = center_angle - half_span + half_gap;
        let end = center_angle + half_span - half_gap;
        for j in 0..=subs {
            let frac = j as f32 / subs as f32;
            let angle = frac.mul_add(end - start, start);
            positions.extend_from_slice(&[r * angle.cos(), r * angle.sin(), 0.0]);
            normals.extend_from_slice(&[0.0, 0.0, 1.0]);
        }
    }

    for j in 0..subs as u32 {
        let i0 = j;
        let i1 = j + 1;
        let i2 = subs as u32 + 1 + j;
        let i3 = subs as u32 + 1 + j + 1;
        indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
    }

    let prim = doc.create_prim();
    prim.set_mesh_stream("POSITION", Some(&positions));
    prim.set_mesh_stream("NORMAL", Some(&normals));
    prim.set_mesh_indices_u32(Some(&indices));
    prim
}
