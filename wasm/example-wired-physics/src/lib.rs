use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::scene::api::Scene,
    wired::{
        math::types::Vec3,
        physics::types::{Collider, RigidBody, RigidBodyType, Shape},
    },
};

#[allow(warnings)]
mod bindings;
mod wired_math_impls;

struct Script {}

impl GuestScript for Script {
    fn new() -> Self {
        let scene = Scene::new();

        // Ground.
        let size = Vec3::new(10.0, 0.5, 10.0);

        let node = scene.create_node();
        let collider = Collider::new(Shape::Cuboid(size));
        let rigid_body = RigidBody::new(RigidBodyType::Fixed);

        node.set_collider(Some(&collider));
        node.set_rigid_body(Some(&rigid_body));

        // Spheres.
        for i in 1..5 {
            let radius = 0.25;

            let node = scene.create_node();
            let mut tr = node.transform();
            tr.translation.x += (i as f32) / 4.0;
            tr.translation.y += i as f32;

            node.set_transform(tr);

            let collider = Collider::new(Shape::Sphere(radius));
            let rigid_body = RigidBody::new(RigidBodyType::Dynamic);

            node.set_collider(Some(&collider));
            node.set_rigid_body(Some(&rigid_body));
        }

        Script {}
    }

    fn update(&self, _delta: f32) {}
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
