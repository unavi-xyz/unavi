use wired_prelude::wired_math::types::Vec3;

use crate::{
    unavi::shapes::api::Cuboid,
    wired::scene::types::{Document, Node, RigidBodyKind},
};

const TABLE_W: f32 = 0.60;
const TABLE_D: f32 = 0.44;
const BASE_H: f32 = 0.016;
const LIP_H: f32 = 0.036;
const LIP_T: f32 = 0.012;
const LIP_Y: f32 = BASE_H * 0.5 + LIP_H * 0.5;
const X_LIP_X: f32 = TABLE_W * 0.5 - LIP_T * 0.5;
const Z_LIP_Z: f32 = TABLE_D * 0.5 - LIP_T * 0.5;

pub struct InventoryActive {
    pub root: Node,
    _nodes: Vec<Node>,
}

impl InventoryActive {
    pub fn new(doc: &Document, color: [f32; 3]) -> Self {
        let mut nodes = Vec::new();

        let mat = doc.create_material();
        mat.set_base_color(&[color[0], color[1], color[2], 1.0]);
        mat.set_double_sided(true);

        let root = doc.create_node();
        root.set_scale(Vec3::ZERO);

        // Base surface.
        let base = doc.create_node();
        let base_shape = Cuboid::new(TABLE_W, BASE_H, TABLE_D);
        base.set_collider(Some(&base_shape.collider()));
        base.set_rigid_body(Some(RigidBodyKind::Fixed));
        base.set_mesh(Some(&base_shape.mesh()));
        base.set_material(Some(&mat));
        root.add_child(&base);
        nodes.push(base);

        // X-axis rim lips (left and right).
        let x_lip_shape = Cuboid::new(LIP_T, LIP_H, TABLE_D);
        for x_sign in [-1.0_f32, 1.0_f32] {
            let lip = doc.create_node();
            lip.set_collider(Some(&x_lip_shape.collider()));
            lip.set_rigid_body(Some(RigidBodyKind::Fixed));
            lip.set_mesh(Some(&x_lip_shape.mesh()));
            lip.set_material(Some(&mat));
            lip.set_translation(Vec3::new(x_sign * X_LIP_X, LIP_Y, 0.0));
            root.add_child(&lip);
            nodes.push(lip);
        }

        // Z-axis rim lips (front and back).
        let z_lip_shape = Cuboid::new(TABLE_W, LIP_H, LIP_T);
        for z_sign in [-1.0_f32, 1.0_f32] {
            let lip = doc.create_node();
            lip.set_collider(Some(&z_lip_shape.collider()));
            lip.set_rigid_body(Some(RigidBodyKind::Fixed));
            lip.set_mesh(Some(&z_lip_shape.mesh()));
            lip.set_material(Some(&mat));
            lip.set_translation(Vec3::new(0.0, LIP_Y, z_sign * Z_LIP_Z));
            root.add_child(&lip);
            nodes.push(lip);
        }

        Self {
            root,
            _nodes: nodes,
        }
    }
}
