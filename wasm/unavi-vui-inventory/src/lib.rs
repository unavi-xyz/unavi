use wired_prelude::{wired_math::types::Vec3, wired_scene::types::Color};

use crate::{
    unavi::shapes::api::Cuboid,
    unavi::vui_module::api::{ModuleEvent, VuiModule},
    wired::scene::{
        context::self_document,
        types::{Node, RigidBodyKind},
    },
};

wired_prelude::generate_script!(Script);

const COLOR: Color = Color::rgb(0.52, 0.20, 0.82);
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
    module: VuiModule,
}

impl GuestScript for Script {
    fn new() -> Self {
        let doc = self_document();

        let mat = doc.create_material();
        mat.set_base_color(COLOR);
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

        let icon = doc.create_node();
        let icon_shape = Cuboid::new(ICON_SIZE, ICON_SIZE, ICON_SIZE);
        icon.set_mesh(Some(&icon_shape.mesh()));
        icon.set_material(Some(&mat));
        icon.set_scale(Vec3::ZERO);
        let module = VuiModule::new(NAME, COLOR, &icon);
        nodes.push(icon);

        Self {
            root,
            _nodes: nodes,
            module,
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
            }
        }
    }

    fn render(&self) {}
    fn drop(&self) {}
}
