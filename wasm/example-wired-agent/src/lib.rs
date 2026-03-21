use std::time::SystemTime;

use wired_prelude::wired_math::types::Vec3;

use crate::{
    unavi::shapes::api::Cuboid,
    wired::{
        agent::{context::local_agent, types::BoneName},
        scene::{context::self_document, types::Node},
    },
};

wired_prelude::generate_script!(Script);

struct Script {
    node: Node,
    time: SystemTime,
}

impl GuestScript for Script {
    fn new() -> Self {
        let doc = self_document();

        let size = 0.1;
        let mesh = Cuboid::new(size, size, size).mesh();

        let mat = doc.create_material();
        mat.set_base_color(&[0.8, 0.1, 0.1, 1.0]);

        let node = doc.create_node();
        node.set_mesh(Some(&mesh));
        node.set_material(Some(&mat));

        // Attach to bone.
        let agent = local_agent();
        let hand = agent.bone(BoneName::RightHand).expect("get bone");
        hand.add_child(&node);

        Self {
            node,
            time: SystemTime::now(),
        }
    }

    fn tick(&self) {
        let now = self.time.elapsed().expect("elapsed").as_secs_f32();

        let tr = self.node.global_transform().translation;
        println!("{}x {}y {}z", tr.x, tr.y, tr.z);

        self.node
            .set_translation(Vec3::new(0.0, now.sin() * 0.1, 0.0));
    }

    fn render(&self) {}

    fn drop(&self) {}
}
