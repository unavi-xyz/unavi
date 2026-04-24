use wired_prelude::prelude::*;

use crate::{
    unavi::{
        shapes::api::Cuboid,
        vui_module::api::{ModuleEvent, VuiModule},
    },
    wired::scene::{
        api::self_document,
        types::{Material, Mesh, Node, RigidBodyKind},
    },
};

wired_prelude::generate_script!(Script);

const NAME: &str = "Inventory";

const BASE_H: f32 = 0.016;
const ICON_SIZE: f32 = 0.040;
const LIP_H: f32 = 0.036;
const LIP_T: f32 = 0.012;
const LIP_Y: f32 = BASE_H * 0.5 + LIP_H * 0.5;
const TABLE_D: f32 = 0.44;
const TABLE_W: f32 = 0.60;
const X_LIP_X: f32 = TABLE_W * 0.5 - LIP_T * 0.5;
const Z_LIP_Z: f32 = TABLE_D * 0.5 - LIP_T * 0.5;

struct Script {
    root: Node,
    _nodes: Vec<Node>,
    _icon_mesh: Mesh,
    module: VuiModule,
    color_mat: Material,
}

impl GuestScript for Script {
    fn new() -> Self {
        let doc = self_document();

        let color_mat = doc.create_material();
        color_mat.set_double_sided(true);

        let root = doc.create_node();
        root.set_scale(Vec3::ZERO);

        let mut nodes = Vec::new();

        let base = doc.create_node();
        let base_shape = Cuboid::new(Vec3::new(TABLE_W, BASE_H, TABLE_D));
        base.set_collider(Some(&base_shape.collider()));
        base.set_rigid_body(Some(RigidBodyKind::Fixed));
        base.set_mesh(Some(&base_shape.mesh()));
        base.set_material(Some(&color_mat));
        root.add_child(&base);
        nodes.push(base);

        let x_lip_shape = Cuboid::new(Vec3::new(LIP_T, LIP_H, TABLE_D));
        for x_sign in [-1.0_f32, 1.0_f32] {
            let lip = doc.create_node();
            lip.set_collider(Some(&x_lip_shape.collider()));
            lip.set_rigid_body(Some(RigidBodyKind::Fixed));
            lip.set_mesh(Some(&x_lip_shape.mesh()));
            lip.set_material(Some(&color_mat));
            lip.set_translation(Vec3::new(x_sign * X_LIP_X, LIP_Y, 0.0));
            root.add_child(&lip);
            nodes.push(lip);
        }

        let z_lip_shape = Cuboid::new(Vec3::new(TABLE_W, LIP_H, LIP_T));
        for z_sign in [-1.0_f32, 1.0_f32] {
            let lip = doc.create_node();
            lip.set_collider(Some(&z_lip_shape.collider()));
            lip.set_rigid_body(Some(RigidBodyKind::Fixed));
            lip.set_mesh(Some(&z_lip_shape.mesh()));
            lip.set_material(Some(&color_mat));
            lip.set_translation(Vec3::new(0.0, LIP_Y, z_sign * Z_LIP_Z));
            root.add_child(&lip);
            nodes.push(lip);
        }

        let icon_mesh = Cuboid::new(Vec3::splat(ICON_SIZE)).mesh();
        let module = VuiModule::new(NAME, &icon_mesh);

        Self {
            root,
            _nodes: nodes,
            _icon_mesh: icon_mesh,
            module,
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
                }
                ModuleEvent::Deactivate => {
                    self.root.set_scale(Vec3::ZERO);
                }
                ModuleEvent::SetColor(color) => {
                    self.color_mat.set_base_color(color);
                }
            }
        }
    }

    fn render(&self) {}
    fn drop(&self) {}
}
