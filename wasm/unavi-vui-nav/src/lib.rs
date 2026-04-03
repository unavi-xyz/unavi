use std::cell::RefCell;

use wired_prelude::{wired_math::types::Vec3, wired_scene::types::Color};

use crate::{
    unavi::{
        shapes::api::{Cuboid, Cylinder, Torus},
        vui_module::api::{ModuleEvent, VuiModule},
    },
    wired::{
        scene::{
            context::self_document,
            types::{Collider, ColliderCylinder, Material, Node, RigidBodyKind},
        },
        wds::{context::get_wds, types::QueryFuture},
    },
};

wired_prelude::generate_script!(Script);

const NAME: &str = "Nav";

const BASE_H: f32 = 0.016;
const BASIN_HEIGHT: f32 = 0.18;
const BASIN_RADIUS: f32 = 0.52;
const BASIN_X: f32 = 0.58;
const BASIN_Y: f32 = -0.10;
const ICON_MINOR_R: f32 = 0.012;
const ICON_MAJOR_R: f32 = 0.028;
const LIP_H: f32 = 0.036;
const LIP_T: f32 = 0.012;
const LIP_Y: f32 = BASE_H * 0.5 + LIP_H * 0.5;
const RING_COLLIDER_HEIGHT: f32 = RING_THICKNESS * 2.0;
const RING_COLLIDER_RADIUS: f32 = RING_RADIUS + 0.06;
const RING_RADIUS: f32 = 0.56;
const RING_THICKNESS: f32 = 0.040;
const TABLE_D: f32 = 0.64;
const TABLE_W: f32 = 1.00;
const X_LIP_X: f32 = TABLE_W * 0.5 - LIP_T * 0.5;
const Z_LIP_Z: f32 = TABLE_D * 0.5 - LIP_T * 0.5;

struct Script {
    root: Node,
    ring: Node,
    _nodes: Vec<Node>,
    module: VuiModule,
    beacon_query: RefCell<Option<QueryFuture>>,
    color_mat: Material,
}

impl GuestScript for Script {
    fn new() -> Self {
        let doc = self_document();
        let mut nodes = Vec::new();

        let color_mat = doc.create_material();
        color_mat.set_double_sided(true);

        let root = doc.create_node();
        root.set_scale(Vec3::ZERO);

        let filter_table = make_filter_table(&doc, &color_mat, &mut nodes);
        filter_table.set_translation(Vec3::new(BASIN_X, 0.0, 0.0));
        root.add_child(&filter_table);
        nodes.push(filter_table);

        let basin = make_basin(&doc, &mut nodes);
        basin.set_translation(Vec3::new(-BASIN_X, BASIN_Y, 0.0));
        root.add_child(&basin);
        nodes.push(basin);

        let ring_mat = doc.create_material();
        ring_mat.set_base_color(Color::WHITE);
        ring_mat.set_double_sided(true);

        let ring = doc.create_node();
        ring.set_mesh(Some(&Torus::new(RING_THICKNESS, RING_RADIUS).mesh()));
        ring.set_material(Some(&ring_mat));
        ring.set_collider(Some(&Collider::Cylinder(ColliderCylinder {
            height: RING_COLLIDER_HEIGHT,
            radius: RING_COLLIDER_RADIUS,
        })));
        ring.set_rigid_body(Some(RigidBodyKind::Dynamic));
        ring.set_scale(Vec3::ZERO);

        let icon = doc.create_node();
        icon.set_mesh(Some(&Torus::new(ICON_MINOR_R, ICON_MAJOR_R).mesh()));
        icon.set_material(Some(&color_mat));
        icon.set_scale(Vec3::ZERO);
        let module = VuiModule::new(NAME, &icon);
        nodes.push(icon);

        Self {
            root,
            ring,
            _nodes: nodes,
            module,
            beacon_query: RefCell::new(None),
            color_mat,
        }
    }

    fn tick(&self) {
        while let Some(event) = self.module.poll() {
            match event {
                ModuleEvent::Activate(t) => {
                    self.root.set_translation(t.translation);
                    self.root.set_rotation(t.rotation);
                    self.root.set_scale(t.scale);
                    self.ring.set_translation(Vec3 {
                        x: t.translation.x - BASIN_X,
                        y: t.translation.y + 0.5,
                        z: t.translation.z,
                    });
                    // TODO fix ring position, grab, add phys joint
                    // self.ring.set_scale(Vec3::ONE);

                    let fut = get_wds().query(None);
                    *self.beacon_query.borrow_mut() = Some(fut);
                }
                ModuleEvent::Deactivate => {
                    self.root.set_scale(Vec3::ZERO);
                    self.ring.set_scale(Vec3::ZERO);
                    *self.beacon_query.borrow_mut() = None;
                }
                ModuleEvent::SetColor(color) => {
                    self.color_mat.set_base_color(color);
                }
            }
        }

        let mut remove_query = false;

        if let Some(fut) = self.beacon_query.borrow().as_ref()
            && let Some(result) = fut.poll()
        {
            match result {
                Ok(ids) => {
                    for id in ids {
                        let id = blake3::Hash::from_slice(&id).expect("valid hash");
                        println!("beacon record: {id}");
                    }

                    remove_query = true;
                }
                Err(()) => eprintln!("wds query error"),
            }
        }

        if remove_query {
            *self.beacon_query.borrow_mut() = None;
        }
    }

    fn render(&self) {}
    fn drop(&self) {}
}

fn make_filter_table(
    doc: &crate::wired::scene::types::Document,
    mat: &crate::wired::scene::types::Material,
    nodes: &mut Vec<Node>,
) -> Node {
    let group = doc.create_node();

    let base = doc.create_node();
    let base_shape = Cuboid::new(TABLE_W, BASE_H, TABLE_D);
    base.set_collider(Some(&base_shape.collider()));
    base.set_rigid_body(Some(RigidBodyKind::Fixed));
    base.set_mesh(Some(&base_shape.mesh()));
    base.set_material(Some(mat));
    group.add_child(&base);
    nodes.push(base);

    let x_lip_shape = Cuboid::new(LIP_T, LIP_H, TABLE_D);
    for x_sign in [-1.0_f32, 1.0_f32] {
        let lip = doc.create_node();
        lip.set_collider(Some(&x_lip_shape.collider()));
        lip.set_rigid_body(Some(RigidBodyKind::Fixed));
        lip.set_mesh(Some(&x_lip_shape.mesh()));
        lip.set_material(Some(mat));
        lip.set_translation(Vec3::new(x_sign * X_LIP_X, LIP_Y, 0.0));
        group.add_child(&lip);
        nodes.push(lip);
    }

    let z_lip_shape = Cuboid::new(TABLE_W, LIP_H, LIP_T);
    for z_sign in [-1.0_f32, 1.0_f32] {
        let lip = doc.create_node();
        lip.set_collider(Some(&z_lip_shape.collider()));
        lip.set_rigid_body(Some(RigidBodyKind::Fixed));
        lip.set_mesh(Some(&z_lip_shape.mesh()));
        lip.set_material(Some(mat));
        lip.set_translation(Vec3::new(0.0, LIP_Y, z_sign * Z_LIP_Z));
        group.add_child(&lip);
        nodes.push(lip);
    }

    let divider = doc.create_node();
    let divider_shape = Cuboid::new(LIP_T, LIP_H, TABLE_D);
    divider.set_collider(Some(&divider_shape.collider()));
    divider.set_rigid_body(Some(RigidBodyKind::Fixed));
    divider.set_mesh(Some(&divider_shape.mesh()));
    divider.set_material(Some(mat));
    divider.set_translation(Vec3::new(0.0, LIP_Y, 0.0));
    group.add_child(&divider);
    nodes.push(divider);

    group
}

fn make_basin(doc: &crate::wired::scene::types::Document, nodes: &mut Vec<Node>) -> Node {
    let group = doc.create_node();

    let mat = doc.create_material();
    mat.set_base_color(Color::rgb(0.88, 0.88, 0.92));
    mat.set_double_sided(true);

    let cylinder = Cylinder::new(BASIN_RADIUS, BASIN_HEIGHT);
    let dish = doc.create_node();
    dish.set_mesh(Some(&cylinder.mesh()));
    dish.set_material(Some(&mat));
    dish.set_collider(Some(&cylinder.collider()));
    dish.set_rigid_body(Some(RigidBodyKind::Fixed));
    group.add_child(&dish);
    nodes.push(dish);

    group
}
