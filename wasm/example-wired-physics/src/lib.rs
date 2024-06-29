use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::shapes::api::create_cuboid,
    wired::{
        gltf::node::create_node,
        math::types::Vec3,
        physics::types::{Collider, Cuboid, RigidBody, RigidBodyType, Shape},
    },
};

#[allow(warnings)]
mod bindings;
mod wired_math_impls;

struct Script {}

impl GuestScript for Script {
    fn new() -> Self {
        let node = create_node();
        let mesh = create_cuboid(Vec3::splat(1.0));
        node.set_mesh(Some(&mesh));

        let collider = Collider::new(Shape::Cuboid(Cuboid {
            x_len: 10.0,
            y_len: 0.5,
            z_len: 10.0,
        }));

        let rigid_body = RigidBody::new(RigidBodyType::Fixed);

        // node.set_collider(Some(&collider));
        // node.set_rigid_body(Some(&rigid_body));

        Script {}
    }

    fn update(&self, _delta: f32) {}
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
