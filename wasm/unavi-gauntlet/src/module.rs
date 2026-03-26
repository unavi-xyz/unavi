use std::cell::Cell;
use std::f32::consts::PI;

use crate::{
    DynamicModuleDef,
    gauntlet::{
        BG_ALPHA_BASE, ICON_Z, OUTLINE_COLOR, OUTLINE_WIDTH, OUTLINE_Z, RING_RADIUS,
        SECTOR_GAP_WORLD, SECTOR_INNER_R, SECTOR_SUBDIVISIONS,
    },
    unavi::shapes::api::{Cuboid, Torus},
    wired::scene::types::{
        AlphaMode, Document, Indices, Material, Mesh, Node, PrimitiveTopology,
    },
};

use wired_prelude::wired_math::types::Quat;

pub struct Module {
    pub active_state: Cell<bool>,
    pub bg_color: [f32; 3],
    pub bg_material: Material,
    pub name: String,
    pub outline_node: Node,
    pub raise_t: Cell<f32>,
    pub root: Node,
    _bg: Node,
    _icon: Node,
}

pub fn make_modules(doc: &Document, defs: &[DynamicModuleDef], colors: &[[f32; 3]]) -> Vec<Module> {
    let n = defs.len();
    defs.iter()
        .enumerate()
        .map(|(i, def)| make_module(doc, i, n, &def.name, def.icon.as_str(), colors[i]))
        .collect()
}

fn make_module(
    doc: &Document,
    i: usize,
    n: usize,
    name: &str,
    icon: &str,
    color: [f32; 3],
) -> Module {
    let angle = i as f32 * 2.0 * PI / n as f32;

    let icon_mat = doc.create_material();
    icon_mat.set_base_color(&[color[0], color[1], color[2], 1.0]);
    icon_mat.set_unlit(true);

    let icon_node = doc.create_node();
    icon_node.set_mesh(Some(&make_icon_mesh(doc, icon)));
    icon_node.set_material(Some(&icon_mat));
    if icon == "torus" {
        // Rotate 90° around X so the torus ring faces the viewer.
        let half = PI / 4.0;
        icon_node.set_rotation(Quat {
            x: half.sin(),
            y: 0.0,
            z: 0.0,
            w: half.cos(),
        });
    }
    let mid_r = f32::midpoint(SECTOR_INNER_R, RING_RADIUS);
    icon_node.set_translation(wired_prelude::wired_math::types::Vec3::new(
        mid_r * angle.cos(),
        mid_r * angle.sin(),
        ICON_Z,
    ));

    let bg_material = doc.create_material();
    bg_material.set_base_color(&[color[0], color[1], color[2], BG_ALPHA_BASE]);
    bg_material.set_alpha_mode(Some(AlphaMode::Add));
    bg_material.set_double_sided(true);
    bg_material.set_unlit(true);
    let bg = doc.create_node();
    bg.set_mesh(Some(&make_sector_mesh(doc, i, n)));
    bg.set_material(Some(&bg_material));

    let outline_mat = doc.create_material();
    outline_mat.set_base_color(&[OUTLINE_COLOR[0], OUTLINE_COLOR[1], OUTLINE_COLOR[2], 1.0]);
    outline_mat.set_double_sided(true);
    outline_mat.set_unlit(true);
    let outline = doc.create_node();
    outline.set_mesh(Some(&make_outline_mesh(doc, i, n)));
    outline.set_material(Some(&outline_mat));
    outline.set_scale(wired_prelude::wired_math::types::Vec3::ZERO);

    let root = doc.create_node();
    root.add_child(&bg);
    root.add_child(&icon_node);
    root.add_child(&outline);

    Module {
        active_state: Cell::new(false),
        bg_color: color,
        bg_material,
        name: name.to_string(),
        outline_node: outline,
        raise_t: Cell::new(0.0),
        root,
        _bg: bg,
        _icon: icon_node,
    }
}

fn make_icon_mesh(_doc: &Document, icon: &str) -> Mesh {
    let side = 0.025_f32 * 1.3;
    match icon {
        "torus" => Torus::new(side * 0.3, side).mesh(),
        _ => Cuboid::new(side, side, side).mesh(),
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
