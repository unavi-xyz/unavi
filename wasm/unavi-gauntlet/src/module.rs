use std::{cell::Cell, f32::consts::PI};

use wired_prelude::wired_math::types::Vec3;

use crate::{
    gauntlet::{
        BG_ALPHA_BASE, ICON_ALPHA_BASE, ICON_Z, RING_RADIUS, SECTOR_GAP_WORLD, SECTOR_INNER_R,
        SECTOR_SUBDIVISIONS,
    },
    unavi::shapes::api::{Cuboid, Sphere},
    wired::scene::{
        context::self_document,
        types::{AlphaMode, Document, Indices, Material, Mesh, Node, PrimitiveTopology},
    },
};

pub const ICON_RADIUS: f32 = 0.025;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ModuleKind {
    Config,
    Inventory,
    Nav,
}

pub struct ModuleDef {
    pub kind: ModuleKind,
    pub name: &'static str,
    pub rgb: [f32; 3],
}

pub struct Module {
    pub active: Node,
    pub bg_color: [f32; 3],
    pub bg_material: Material,
    pub icon_material: Material,
    pub kind: ModuleKind,
    pub name: &'static str,
    pub raise_t: Cell<f32>,
    pub root: Node,
    // Keep handles alive so host doesn't destroy the nodes.
    _bg: Node,
    _icon: Node,
}

pub fn make_modules(defs: &[ModuleDef]) -> Vec<Module> {
    let n = defs.len();
    let doc = self_document();
    defs.iter()
        .enumerate()
        .map(|(i, def)| make_module(&doc, i, n, def))
        .collect()
}

fn make_module(doc: &Document, i: usize, n: usize, def: &ModuleDef) -> Module {
    let angle = i as f32 * 2.0 * PI / n as f32;

    let icon_mat = doc.create_material();
    icon_mat.set_base_color(&[def.rgb[0], def.rgb[1], def.rgb[2], ICON_ALPHA_BASE]);
    icon_mat.set_alpha_mode(Some(AlphaMode::Blend));
    let icon = doc.create_node();
    icon.set_mesh(Some(&Sphere::new(ICON_RADIUS).mesh()));
    icon.set_material(Some(&icon_mat));
    let mid_r = f32::midpoint(SECTOR_INNER_R, RING_RADIUS);
    icon.set_translation(Vec3::new(mid_r * angle.cos(), mid_r * angle.sin(), ICON_Z));

    let bg_material = doc.create_material();
    bg_material.set_base_color(&[def.rgb[0], def.rgb[1], def.rgb[2], BG_ALPHA_BASE]);
    bg_material.set_alpha_mode(Some(AlphaMode::Blend));
    bg_material.set_double_sided(true);
    let bg = doc.create_node();
    bg.set_mesh(Some(&make_sector_mesh(doc, i, n)));
    bg.set_material(Some(&bg_material));

    let root = doc.create_node();
    root.add_child(&bg);
    root.add_child(&icon);

    let active = doc.create_node();
    active.set_mesh(Some(&Cuboid::new(0.15, 0.15, 0.15).mesh()));
    active.set_scale(Vec3::ZERO);

    Module {
        active,
        bg_color: def.rgb,
        bg_material,
        icon_material: icon_mat,
        kind: def.kind,
        name: def.name,
        raise_t: Cell::new(0.0),
        root,
        _bg: bg,
        _icon: icon,
    }
}

fn make_sector_mesh(doc: &Document, i: usize, n: usize) -> Mesh {
    let half_span = PI / n as f32;
    let center_angle = i as f32 * 2.0 * PI / n as f32;
    let subs = SECTOR_SUBDIVISIONS;

    let mut positions: Vec<f32> = Vec::with_capacity(6 * (subs + 1));
    let mut normals: Vec<f32> = Vec::with_capacity(6 * (subs + 1));

    for &r in &[SECTOR_INNER_R, RING_RADIUS] {
        let half_gap = SECTOR_GAP_WORLD / (2.0 * r);
        let start = center_angle - half_span + half_gap;
        let end = center_angle + half_span - half_gap;
        for j in 0..=subs {
            let t = j as f32 / subs as f32;
            let a = t.mul_add(end - start, start);
            positions.extend_from_slice(&[r * a.cos(), r * a.sin(), 0.0]);
            normals.extend_from_slice(&[0.0, 0.0, 1.0]);
        }
    }

    let mut indices: Vec<u16> = Vec::with_capacity(6 * subs);
    for j in 0..subs {
        let i0 = j as u16;
        let i1 = (j + 1) as u16;
        let i2 = (subs + 1 + j) as u16;
        let i3 = (subs + 1 + j + 1) as u16;
        indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
    }

    let mesh = doc.create_mesh();
    mesh.set_topology(PrimitiveTopology::TriangleList);
    mesh.set_positions(Some(&positions));
    mesh.set_normals(Some(&normals));
    mesh.set_indices(Some(&Indices::Half(indices)));
    mesh
}
