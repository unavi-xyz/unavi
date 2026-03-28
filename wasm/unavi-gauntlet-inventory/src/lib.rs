use wired_prelude::wired_math::types::{Quat, Vec3};

use crate::{
    unavi::shapes::api::Cuboid,
    wired::{
        event::{
            api::{register_emitter, register_receptor},
            types::{EventEmitter, EventReceptor},
        },
        scene::{
            context::self_document,
            types::{Node, RigidBodyKind},
        },
    },
};

wired_prelude::generate_script!(Script);

const CH_ACTIVATE: &str = "unavi::gauntlet::activate";
const CH_DEACTIVATE: &str = "unavi::gauntlet::deactivate";
const CH_REGISTER: &str = "unavi::gauntlet::register";
const CH_REGISTER_REQUEST: &str = "unavi::gauntlet::register-request";

const COLOR: [f32; 3] = [0.52, 0.20, 0.82];
const NAME: &str = "Inventory";

const BASE_H: f32 = 0.016;
const LIP_H: f32 = 0.036;
const LIP_T: f32 = 0.012;
const LIP_Y: f32 = BASE_H * 0.5 + LIP_H * 0.5;
const TABLE_D: f32 = 0.44;
const TABLE_W: f32 = 0.60;
const X_LIP_X: f32 = TABLE_W * 0.5 - LIP_T * 0.5;
const Z_LIP_Z: f32 = TABLE_D * 0.5 - LIP_T * 0.5;

#[derive(serde::Serialize, serde::Deserialize)]
struct RegisterPayload<'a> {
    name: &'a str,
    icon_node_id: &'a str,
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
    icon: Node,
    _nodes: Vec<Node>,
    _emitter: EventEmitter,
    request_receptor: EventReceptor,
    activate_receptor: EventReceptor,
}

impl GuestScript for Script {
    fn new() -> Self {
        let doc = self_document();
        let icon = make_icon(&doc);

        let mat = doc.create_material();
        mat.set_base_color(&[COLOR[0], COLOR[1], COLOR[2], 1.0]);
        mat.set_double_sided(true);

        let root = doc.create_node();
        root.set_scale(Vec3::ZERO);

        let mut nodes = Vec::new();

        let base = doc.create_node();
        let base_shape = Cuboid::new(TABLE_W, BASE_H, TABLE_D);
        base.set_collider(Some(&base_shape.collider()));
        base.set_rigid_body(Some(RigidBodyKind::Fixed));
        base.set_mesh(Some(&base_shape.mesh()));
        base.set_material(Some(&mat));
        root.add_child(&base);
        nodes.push(base);

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

        // Listen for register-request from gauntlet, then reply with our registration
        let request_receptor =
            register_receptor(&[CH_REGISTER_REQUEST.to_string()], None, f32::MAX, &[]);

        // Broadcast emitter (set target-doc per reply in tick)
        let emitter = register_emitter(None, f32::MAX, &[]);

        // Listen for activate/deactivate from any source
        let activate_receptor = register_receptor(
            &[CH_ACTIVATE.to_string(), CH_DEACTIVATE.to_string()],
            None,
            f32::MAX,
            &[],
        );

        // Emit our registration to any gauntlet doc that has already broadcast
        // (handled reactively in tick when we receive register-request)
        drop(emitter);

        Self {
            root,
            icon,
            _nodes: nodes,
            _emitter: register_emitter(None, f32::MAX, &[]),
            request_receptor,
            activate_receptor,
        }
    }

    fn tick(&self) {
        // Reply to register-request with our registration
        while let Some(event) = self.request_receptor.poll() {
            let icon_node_id = self.icon.id();
            let payload = postcard::to_allocvec(&RegisterPayload {
                name: NAME,
                icon_node_id: &icon_node_id,
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
                    }
                }
                CH_DEACTIVATE => {
                    self.root.set_scale(Vec3::ZERO);
                }
                _ => {}
            }
        }
    }

    fn render(&self) {}
    fn drop(&self) {}
}

fn make_icon(doc: &crate::wired::scene::types::Document) -> Node {
    let side = 0.030_f32;
    let shape = Cuboid::new(side, side, side);
    let mat = doc.create_material();
    mat.set_base_color(&[COLOR[0], COLOR[1], COLOR[2], 1.0]);
    mat.set_unlit(true);
    let node = doc.create_node();
    node.set_mesh(Some(&shape.mesh()));
    node.set_material(Some(&mat));
    node
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
