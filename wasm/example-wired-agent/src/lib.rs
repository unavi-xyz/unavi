use std::time::SystemTime;

use wired_prelude::prelude::*;

use crate::{
    unavi::shapes::api::Cuboid,
    wired::{
        agent::{api::local_agent, types::BoneName},
        scene::{api::self_document, types::Node},
    },
};

wired_prelude::generate_script!(Script);

struct Script {
    hand: Node,
    node: Node,
    time: SystemTime,
}

impl ScriptBehavior for Script {
    fn init() -> Self {
        let doc = self_document();

        let size = 0.1;
        let mesh = Cuboid::new(Vec3::splat(size)).mesh();

        let mat = doc.create_material();
        mat.set_base_color(Color::rgba(0.8, 0.1, 0.1, 1.0));

        let node = doc.create_node();
        node.set_mesh(Some(&mesh));
        node.set_material(Some(&mat));

        let agent = local_agent();
        let hand = agent.bone(BoneName::RightHand).expect("get bone");

        Self {
            hand,
            node,
            time: SystemTime::now(),
        }
    }

    fn tick(&mut self) {
        let now = self.time.elapsed().expect("elapsed").as_secs_f32();

        let offset = Vec3::new(0.0, now.sin() * 0.1, 0.0);

        let mut tr = self.hand.global_transform();
        println!("{}", tr.translation);
        tr.translation += offset;
        self.node.set_transform(tr);
    }
}
