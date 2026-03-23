use wired_prelude::wired_math::types::Vec3;

use crate::{
    unavi::shapes::api::{Cuboid, Sphere},
    wired::scene::{
        context::self_document,
        types::{Document, Material, Node},
    },
};

pub const ICON_RADIUS: f32 = 0.025;

pub struct Module {
    pub active: Node,
    pub color: [f32; 4],
    pub icon: Node,
    pub material: Material,
}

pub fn make_modules(base_color: [f32; 4], count: usize) -> Vec<Module> {
    let doc = self_document();
    (0..count)
        .map(|i| {
            let t = i as f32 / count.max(1) as f32;
            let color = [
                (base_color[0] + t * 0.1).min(1.0),
                (base_color[1] + t * 0.1).min(1.0),
                (base_color[2] + t * 0.1).min(1.0),
                base_color[3],
            ];
            make_module(&doc, color)
        })
        .collect()
}

fn make_module(doc: &Document, color: [f32; 4]) -> Module {
    let material = doc.create_material();
    material.set_base_color(&color);

    let icon = doc.create_node();
    icon.set_mesh(Some(&Sphere::new(ICON_RADIUS).mesh()));
    icon.set_material(Some(&material));
    icon.set_scale(Vec3::ZERO);

    let size = 0.05_f32;
    let active = doc.create_node();
    active.set_mesh(Some(&Cuboid::new(size, size, size).mesh()));
    active.set_scale(Vec3::ZERO);

    Module {
        active,
        color,
        icon,
        material,
    }
}
