use wired_prelude::wired_math::types::Vec3;

use crate::{
    unavi::shapes::api::{Cuboid, Cylinder, Torus},
    wired::scene::types::{Collider, ColliderCylinder, Document, Node},
};

const TABLE_W: f32 = 0.50;
const TABLE_D: f32 = 0.32;
const BASE_H: f32 = 0.008;
const LIP_H: f32 = 0.018;
const LIP_T: f32 = 0.006;
const LIP_Y: f32 = BASE_H * 0.5 + LIP_H * 0.5;
const X_LIP_X: f32 = TABLE_W * 0.5 - LIP_T * 0.5;
const Z_LIP_Z: f32 = TABLE_D * 0.5 - LIP_T * 0.5;

const BASIN_X: f32 = 0.29;
const BASIN_RADIUS: f32 = 0.26;
const BASIN_HEIGHT: f32 = 0.09;
const BASIN_Y: f32 = -0.05;

const RING_RADIUS: f32 = 0.28;
const RING_THICKNESS: f32 = 0.020;
const RING_X: f32 = BASIN_X;
const RING_Y: f32 = -0.22;
const RING_COLLIDER_RADIUS: f32 = RING_RADIUS + 0.03;
const RING_COLLIDER_HEIGHT: f32 = RING_THICKNESS * 2.0;

pub struct NavActive {
    pub root: Node,
    _nodes: Vec<Node>,
}

impl NavActive {
    pub fn new(doc: &Document, color: [f32; 3]) -> Self {
        let mut nodes = Vec::new();

        let root = doc.create_node();
        root.set_scale(Vec3::ZERO);

        let filter_table = make_filter_table(doc, color, &mut nodes);
        filter_table.set_translation(Vec3::new(-0.30, 0.0, 0.0));
        root.add_child(&filter_table);
        nodes.push(filter_table);

        let basin = make_basin(doc, &mut nodes);
        basin.set_translation(Vec3::new(BASIN_X, BASIN_Y, 0.0));
        root.add_child(&basin);
        nodes.push(basin);

        let ring_mat = doc.create_material();
        ring_mat.set_base_color(&[1.0, 1.0, 1.0, 1.0]);
        ring_mat.set_double_sided(true);

        let page_ring = doc.create_node();
        page_ring.set_mesh(Some(&Torus::new(RING_THICKNESS, RING_RADIUS).mesh()));
        page_ring.set_material(Some(&ring_mat));
        page_ring.set_translation(Vec3::new(RING_X, RING_Y, 0.0));
        page_ring.set_collider(Some(&Collider::Cylinder(ColliderCylinder {
            height: RING_COLLIDER_HEIGHT,
            radius: RING_COLLIDER_RADIUS,
        })));
        root.add_child(&page_ring);
        nodes.push(page_ring);

        Self {
            root,
            _nodes: nodes,
        }
    }
}

fn make_filter_table(doc: &Document, color: [f32; 3], nodes: &mut Vec<Node>) -> Node {
    let mat = doc.create_material();
    mat.set_base_color(&[color[0], color[1], color[2], 1.0]);
    mat.set_double_sided(true);

    let group = doc.create_node();

    let base = doc.create_node();
    let base_shape = Cuboid::new(TABLE_W, BASE_H, TABLE_D);
    base.set_collider(Some(&base_shape.collider()));
    base.set_mesh(Some(&base_shape.mesh()));
    base.set_material(Some(&mat));
    group.add_child(&base);
    nodes.push(base);

    // X-axis rim lips (left and right).
    let x_lip_shape = Cuboid::new(LIP_T, LIP_H, TABLE_D);
    for x_sign in [-1.0_f32, 1.0_f32] {
        let lip = doc.create_node();
        lip.set_collider(Some(&x_lip_shape.collider()));
        lip.set_mesh(Some(&x_lip_shape.mesh()));
        lip.set_material(Some(&mat));
        lip.set_translation(Vec3::new(x_sign * X_LIP_X, LIP_Y, 0.0));
        group.add_child(&lip);
        nodes.push(lip);
    }

    // Z-axis rim lips (front and back).
    let z_lip_shape = Cuboid::new(TABLE_W, LIP_H, LIP_T);
    for z_sign in [-1.0_f32, 1.0_f32] {
        let lip = doc.create_node();
        lip.set_collider(Some(&z_lip_shape.collider()));
        lip.set_mesh(Some(&z_lip_shape.mesh()));
        lip.set_material(Some(&mat));
        lip.set_translation(Vec3::new(0.0, LIP_Y, z_sign * Z_LIP_Z));
        group.add_child(&lip);
        nodes.push(lip);
    }

    // Center divider.
    let divider = doc.create_node();
    let divider_shape = Cuboid::new(LIP_T, LIP_H, TABLE_D);
    divider.set_collider(Some(&divider_shape.collider()));
    divider.set_mesh(Some(&divider_shape.mesh()));
    divider.set_material(Some(&mat));
    divider.set_translation(Vec3::new(0.0, LIP_Y, 0.0));
    group.add_child(&divider);
    nodes.push(divider);

    group
}

fn make_basin(doc: &Document, nodes: &mut Vec<Node>) -> Node {
    let group = doc.create_node();

    let mat = doc.create_material();
    mat.set_base_color(&[0.88, 0.88, 0.92, 1.0]);
    mat.set_double_sided(true);

    let cylinder = Cylinder::new(BASIN_RADIUS, BASIN_HEIGHT);
    let dish = doc.create_node();
    dish.set_mesh(Some(&cylinder.mesh()));
    dish.set_material(Some(&mat));
    dish.set_collider(Some(&cylinder.collider()));
    group.add_child(&dish);
    nodes.push(dish);

    group
}
