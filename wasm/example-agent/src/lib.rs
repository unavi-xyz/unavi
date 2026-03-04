use std::cell::Cell;

use crate::{
    exports::wired::script::guest_api::{Guest, GuestScript},
    wired::{
        agent::{
            context::local_agent,
            types::{Agent, BoneName},
        },
        scene::types::{Node, Quat},
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
    upper_arm: Node,
    t: Cell<f32>,
}

impl GuestScript for Script {
    fn new() -> Self {
        let agent: Agent = local_agent();
        let doc = agent.document();

        // Build a small red cube mesh (unit cube).
        let mesh = doc.create_mesh();
        mesh.set_positions(Some(&CUBE_POSITIONS));
        mesh.set_indices(Some(&crate::wired::scene::types::Indices::Half(
            CUBE_INDICES.to_vec(),
        )));
        mesh.set_normals(Some(&CUBE_NORMALS));

        let mat = doc.create_material();
        mat.set_base_color(&[0.8, 0.1, 0.1, 1.0]);

        let cube = doc.create_node();
        cube.set_scale(crate::wired::math::types::Vec3 {
            x: 0.08,
            y: 0.08,
            z: 0.08,
        });
        cube.set_mesh(Some(mesh));
        cube.set_material(Some(mat));

        // Attach cube to right hand.
        let hand = agent.bone(BoneName::RightHand).expect("right hand bone");
        hand.add_child(cube);

        // Hold onto right upper arm for animation.
        let upper_arm = agent
            .bone(BoneName::RightUpperArm)
            .expect("right upper arm bone");

        Self {
            upper_arm,
            t: Cell::new(0.0),
        }
    }

    fn tick(&self) {
        let t = self.t.get() + 0.04;
        self.t.set(t);
        // Oscillate right upper arm forward/back.
        let angle = t.sin() * 0.6_f32;
        let (s, c) = (angle / 2.0).sin_cos();
        self.upper_arm.set_rotation(Quat {
            x: s,
            y: 0.0,
            z: 0.0,
            w: c,
        });
    }

    fn render(&self) {}

    fn drop(&self) {}
}

export!(World);

// Unit cube (24 unique vertices, 6 faces × 4 verts, flat normals).
#[rustfmt::skip]
const CUBE_POSITIONS: [f32; 72] = [
    // +Z
    -0.5,-0.5, 0.5,  0.5,-0.5, 0.5,  0.5, 0.5, 0.5, -0.5, 0.5, 0.5,
    // -Z
     0.5,-0.5,-0.5, -0.5,-0.5,-0.5, -0.5, 0.5,-0.5,  0.5, 0.5,-0.5,
    // +Y
    -0.5, 0.5, 0.5,  0.5, 0.5, 0.5,  0.5, 0.5,-0.5, -0.5, 0.5,-0.5,
    // -Y
    -0.5,-0.5,-0.5,  0.5,-0.5,-0.5,  0.5,-0.5, 0.5, -0.5,-0.5, 0.5,
    // +X
     0.5,-0.5, 0.5,  0.5,-0.5,-0.5,  0.5, 0.5,-0.5,  0.5, 0.5, 0.5,
    // -X
    -0.5,-0.5,-0.5, -0.5,-0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5,-0.5,
];

#[rustfmt::skip]
const CUBE_NORMALS: [f32; 72] = [
     0., 0., 1.,  0., 0., 1.,  0., 0., 1.,  0., 0., 1.,
     0., 0.,-1.,  0., 0.,-1.,  0., 0.,-1.,  0., 0.,-1.,
     0., 1., 0.,  0., 1., 0.,  0., 1., 0.,  0., 1., 0.,
     0.,-1., 0.,  0.,-1., 0.,  0.,-1., 0.,  0.,-1., 0.,
     1., 0., 0.,  1., 0., 0.,  1., 0., 0.,  1., 0., 0.,
    -1., 0., 0., -1., 0., 0., -1., 0., 0., -1., 0., 0.,
];

#[rustfmt::skip]
const CUBE_INDICES: [u16; 36] = [
     0, 1, 2,  0, 2, 3,   // +Z
     4, 5, 6,  4, 6, 7,   // -Z
     8, 9,10,  8,10,11,   // +Y
    12,13,14, 12,14,15,   // -Y
    16,17,18, 16,18,19,   // +X
    20,21,22, 20,22,23,   // -X
];
