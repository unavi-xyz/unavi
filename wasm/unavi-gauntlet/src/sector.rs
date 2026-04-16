use std::cell::Cell;
use std::f32::consts::PI;

use crate::{
    Color, ModuleRef,
    gauntlet::{
        BG_ALPHA_BASE, ICON_R, ICON_Z_OFFSET, OUTLINE_COLOR, OUTLINE_WIDTH, OUTLINE_Z, RING_RADIUS,
        SECTOR_GAP_WORLD, SECTOR_INNER_R, SECTOR_SUBDIVISIONS,
    },
    wired::scene::types::{AlphaMode, Document, Indices, Material, Mesh, Node, PrimitiveTopology},
};

use wired_prelude::prelude::*;

pub struct Sector {
    pub module_doc_id: Vec<u8>,
    pub active_state: Cell<bool>,
    pub bg_color: Color,
    pub bg_material: Material,
    pub name: String,
    pub outline_node: Node,
    pub raise_t: Cell<f32>,
    pub root: Node,
    _bg: Node,
    _icon_mesh: Mesh,
    _icon_node: Node,
}

pub fn make_sectors(doc: &Document, modules: &[ModuleRef], colors: &[Color]) -> Vec<Sector> {
    let n = modules.len();
    modules
        .iter()
        .enumerate()
        .map(|(i, module)| make_sector(doc, i, n, module, colors[i]))
        .collect()
}

fn make_sector(doc: &Document, i: usize, n: usize, module: &ModuleRef, color: Color) -> Sector {
    let bg_material = doc.create_material();
    bg_material.set_base_color(Color::rgba(color.r, color.g, color.b, BG_ALPHA_BASE));
    bg_material.set_alpha_mode(Some(AlphaMode::Blend));
    bg_material.set_double_sided(true);
    bg_material.set_unlit(true);
    let bg = doc.create_node();
    bg.set_mesh(Some(&make_sector_mesh(doc, i, n)));
    bg.set_material(Some(&bg_material));

    let outline_mat = doc.create_material();
    outline_mat.set_base_color(OUTLINE_COLOR);
    outline_mat.set_double_sided(true);
    outline_mat.set_unlit(true);
    let outline = doc.create_node();
    outline.set_mesh(Some(&make_outline_mesh(doc, i, n)));
    outline.set_material(Some(&outline_mat));
    outline.set_scale(Vec3::ZERO);

    let ca = i as f32 * 2.0 * PI / n as f32;
    let icon_mesh = doc.create_mesh();
    let icon_node = doc.create_node();
    if let Some(src) = &module.icon_mesh {
        let positions = src.positions().unwrap_or_default();
        let normals = src.normals().unwrap_or_default();
        icon_mesh.set_topology(PrimitiveTopology::TriangleList);
        icon_mesh.set_positions(Some(&positions));
        icon_mesh.set_normals(Some(&normals));
        icon_mesh.set_indices(src.indices().as_ref());
        icon_node.set_mesh(Some(&icon_mesh));
        icon_node.set_material(Some(&bg_material));
    }
    icon_node.set_translation(Vec3::new(
        ICON_R * ca.cos(),
        ICON_R * ca.sin(),
        ICON_Z_OFFSET,
    ));

    let root = doc.create_node();
    root.add_child(&bg);
    root.add_child(&outline);
    root.add_child(&icon_node);

    Sector {
        module_doc_id: module.doc_id.clone(),
        active_state: Cell::new(false),
        bg_color: color,
        bg_material,
        name: module.name.clone(),
        outline_node: outline,
        raise_t: Cell::new(0.0),
        root,
        _bg: bg,
        _icon_mesh: icon_mesh,
        _icon_node: icon_node,
    }
}

fn make_outline_mesh(doc: &Document, i: usize, n: usize) -> Mesh {
    let half_span = PI / n as f32;
    let center_angle = i as f32 * 2.0 * PI / n as f32;
    let subs = SECTOR_SUBDIVISIONS;

    let mut positions: Vec<f32> = Vec::with_capacity(3 * 2 * (subs + 1));
    let mut normals: Vec<f32> = Vec::with_capacity(3 * 2 * (subs + 1));
    let mut indices: Vec<u16> = Vec::with_capacity(6 * subs);

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

fn make_sector_mesh(doc: &Document, i: usize, n: usize) -> Mesh {
    let half_span = PI / n as f32;
    let center_angle = i as f32 * 2.0 * PI / n as f32;
    let subs = SECTOR_SUBDIVISIONS;
    let subs16 = subs as u16;

    let mut positions: Vec<f32> = Vec::with_capacity(3 * 2 * (subs + 1));
    let mut normals: Vec<f32> = Vec::with_capacity(3 * 2 * (subs + 1));
    let mut indices: Vec<u16> = Vec::with_capacity(6 * subs);

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

    for j in 0..subs16 {
        let i0 = j;
        let i1 = j + 1;
        let i2 = subs16 + 1 + j;
        let i3 = subs16 + 1 + j + 1;
        indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
    }

    let mesh = doc.create_mesh();
    mesh.set_topology(PrimitiveTopology::TriangleList);
    mesh.set_positions(Some(&positions));
    mesh.set_normals(Some(&normals));
    mesh.set_indices(Some(&Indices::Half(indices)));
    mesh
}
