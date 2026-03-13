use std::time::SystemTime;

use crate::{
    exports::wired::script::guest_api::{Guest, GuestScript},
    unavi::shapes::api::Sphere,
    wired::{
        agent::{
            context::local_agent,
            types::{Agent, BoneName},
        },
        math::types::Vec3,
        scene::types::Node,
    },
};

wit_bindgen::generate!({
    generate_all,
});

struct World;

impl Guest for World {
    type Script = Script;
}

struct Script {
    node: Node,
    time: SystemTime,
}

impl GuestScript for Script {
    fn new() -> Self {
        let agent: Agent = local_agent();

        let mesh = Sphere::new(0.07).mesh();

        let mat = agent.document().create_material();
        mat.set_base_color(&[0.8, 0.1, 0.1, 1.0]);

        let node = agent.document().create_node();
        node.set_mesh(Some(mesh));
        node.set_material(Some(mat));

        // Attach cube to right hand bone.
        let hand = agent.bone(BoneName::RightHand).expect("right hand bone");
        hand.add_child(node);

        // Retrieve a handle to the cube from its parent.
        let cube = hand.children().into_iter().next().expect("cube child");

        Self {
            node: cube,
            time: SystemTime::now(),
        }
    }

    fn tick(&self) {
        let now = self.time.elapsed().expect("elapsed").as_secs_f32();

        let tr = self.node.global_transform().translation;
        println!("{}x {}y {}z", tr.x, tr.y, tr.z);

        self.node.set_translation(Vec3 {
            x: 0.0,
            y: now.sin() * 0.05,
            z: 0.0,
        });
    }

    fn render(&self) {}

    fn drop(&self) {}
}

export!(World);
