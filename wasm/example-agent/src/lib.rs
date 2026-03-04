use std::cell::Cell;

use crate::{
    exports::wired::script::guest_api::{Guest, GuestScript},
    unavi::shapes::api::Cuboid,
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
    cube: Node,
    t: Cell<f32>,
}

impl GuestScript for Script {
    fn new() -> Self {
        let agent: Agent = local_agent();

        let mesh = Cuboid::new(0.08, 0.08, 0.08).mesh();

        let mat = agent.document().create_material();
        mat.set_base_color(&[0.8, 0.1, 0.1, 1.0]);

        let cube = agent.document().create_node();
        cube.set_mesh(Some(mesh));
        cube.set_material(Some(mat));

        // Attach cube to right hand bone.
        let hand = agent.bone(BoneName::RightHand).expect("right hand bone");
        hand.add_child(cube);

        // Retrieve a handle to the cube from its parent.
        let cube = hand.children().into_iter().next().expect("cube child");

        Self {
            cube,
            t: Cell::new(0.0),
        }
    }

    fn tick(&self) {
        let t = self.t.get() + 0.04;
        self.t.set(t);

        let tr = self.cube.global_transform().translation;
        println!("{}x {}y {}z", tr.x, tr.y, tr.z);

        self.cube.set_translation(Vec3 {
            x: 0.0,
            y: t.sin() * 0.05,
            z: 0.0,
        });
    }

    fn render(&self) {}

    fn drop(&self) {}
}

export!(World);
