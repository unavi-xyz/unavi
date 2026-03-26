use wired_prelude::wired_math::types::{Quat, Vec3};

use crate::{
    unavi::shapes::api::{Cuboid, Cylinder, Torus},
    wired::{
        event::{
            api::{register_emitter, register_receptor},
            types::{EventEmitter, EventReceptor},
        },
        scene::{
            context::self_document,
            types::{Collider, ColliderCylinder, Node, RigidBodyKind},
        },
    },
};

wired_prelude::generate_script!(Script);

const CH_ACTIVATE: &str = "unavi::gauntlet::activate";
const CH_DEACTIVATE: &str = "unavi::gauntlet::deactivate";
const CH_REGISTER: &str = "unavi::gauntlet::register";
const CH_REGISTER_REQUEST: &str = "unavi::gauntlet::register-request";

const COLOR: [f32; 3] = [0.88, 0.52, 0.08];
const ICON: &str = "torus";
const NAME: &str = "Nav";

const BASE_H: f32 = 0.016;
const BASIN_HEIGHT: f32 = 0.18;
const BASIN_RADIUS: f32 = 0.52;
const BASIN_X: f32 = 0.58;
const BASIN_Y: f32 = -0.10;
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

#[derive(serde::Serialize, serde::Deserialize)]
struct RegisterPayload<'a> {
    name: &'a str,
    icon: &'a str,
    color: [f32; 3],
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ActivatePayload {
    translation: [f32; 3],
    rotation: [f32; 4],
    scale: [f32; 3],
}

struct Script {
    root: Node,
    ring: Node,
    _nodes: Vec<Node>,
    _emitter: EventEmitter,
    request_receptor: EventReceptor,
    activate_receptor: EventReceptor,
}

impl GuestScript for Script {
    fn new() -> Self {
        let doc = self_document();
        let mut nodes = Vec::new();

        let color_mat = doc.create_material();
        color_mat.set_base_color(&[COLOR[0], COLOR[1], COLOR[2], 1.0]);
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
        ring_mat.set_base_color(&[1.0, 1.0, 1.0, 1.0]);
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

        // Listen for register-request from gauntlet
        let request_receptor =
            register_receptor(&[CH_REGISTER_REQUEST.to_string()], None, f32::MAX, &[]);

        // Listen for activate/deactivate
        let activate_receptor = register_receptor(
            &[CH_ACTIVATE.to_string(), CH_DEACTIVATE.to_string()],
            None,
            f32::MAX,
            &[],
        );

        Self {
            root,
            ring,
            _nodes: nodes,
            _emitter: register_emitter(None, f32::MAX, &[]),
            request_receptor,
            activate_receptor,
        }
    }

    fn tick(&self) {
        // Reply to register-request with our registration
        while let Some(event) = self.request_receptor.poll() {
            let payload = postcard::to_allocvec(&RegisterPayload {
                name: NAME,
                icon: ICON,
                color: COLOR,
            })
            .expect("encode register");
            let emitter = register_emitter(None, f32::MAX, &[event.sender_document]);
            emitter.emit(CH_REGISTER, &payload);
        }

        // Handle activate/deactivate
        while let Some(event) = self.activate_receptor.poll() {
            match event.channel.as_str() {
                CH_ACTIVATE => {
                    if let Some((pos, rot, scale)) = decode_transform(&event.payload) {
                        self.root.set_translation(pos);
                        self.root.set_rotation(rot);
                        self.root.set_scale(scale);
                        self.ring.set_translation(Vec3 {
                            x: pos.x - BASIN_X,
                            y: pos.y + 0.5,
                            z: pos.z,
                        });
                        self.ring.set_scale(Vec3::ONE);
                    }
                }
                CH_DEACTIVATE => {
                    self.root.set_scale(Vec3::ZERO);
                    self.ring.set_scale(Vec3::ZERO);
                }
                _ => {}
            }
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
    mat.set_base_color(&[0.88, 0.88, 0.92, 1.0]);
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

fn decode_transform(payload: &[u8]) -> Option<(Vec3, Quat, Vec3)> {
    let p: ActivatePayload = postcard::from_bytes(payload).ok()?;
    Some((
        Vec3 {
            x: p.translation[0],
            y: p.translation[1],
            z: p.translation[2],
        },
        Quat {
            x: p.rotation[0],
            y: p.rotation[1],
            z: p.rotation[2],
            w: p.rotation[3],
        },
        Vec3 {
            x: p.scale[0],
            y: p.scale[1],
            z: p.scale[2],
        },
    ))
}
