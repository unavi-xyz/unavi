use wired_prelude::wired_math::types::Vec3;

use crate::{
    unavi::shapes::api::Cuboid,
    wired::scene::{
        context::{load_hsd, self_document},
        types::RigidBodyKind,
    },
};

wired_prelude::generate_script!(Script);

const GROUND_SIZE: f32 = 30.0;
const GROUND_THICK: f32 = 0.5;

struct Script;

impl GuestScript for Script {
    fn new() -> Self {
        let doc = self_document();

        let shape = Cuboid::new(GROUND_SIZE, GROUND_THICK, GROUND_SIZE);

        let node = doc.create_node();
        node.set_mesh(Some(&shape.mesh()));
        node.set_collider(Some(&shape.collider()));
        node.set_rigid_body(Some(RigidBodyKind::Fixed));
        node.set_translation(Vec3::new(0.0, -GROUND_THICK / 2.0, 0.0));

        let ground_mat = doc.materials().into_iter().find(|m| m.id() == "ground");
        node.set_material(ground_mat.as_ref());

        let (_, gate_id) = doc
            .assets()
            .into_iter()
            .find(|(k, _)| k == "gate")
            .expect("gate asset");

        let Ok(gate_doc) = load_hsd(&gate_id) else {
            eprintln!("error loading gate HSD");
            return Self;
        };

        gate_doc.set_translation(Vec3::new(0.0, 0.0, -GROUND_SIZE / 3.0));

        Self
    }

    fn tick(&self) {}

    fn render(&self) {}

    fn drop(&self) {}
}
