use std::cell::Cell;

use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    wired::{
        physics::types::{Collider, RigidBody, RigidBodyType, Shape},
        scene::scene::Node,
    },
};

use crate::bindings::wired::log::api::{log, LogLevel};

#[allow(warnings)]
mod bindings;
mod wired_math_impls;
mod wired_scene_impls;

struct Script {
    is_first_update: Cell<bool>,
    node: Node,
}

impl GuestScript for Script {
    fn new() -> Self {
        log(LogLevel::Info, "Called script construct!");

        let node = Node::new();
        let collider = Collider::new(Shape::Sphere(0.5));
        let rigid_body = RigidBody::new(RigidBodyType::Dynamic);

        {
            node.set_collider(Some(&collider));
            assert!(node.collider().is_some());

            node.set_rigid_body(Some(&rigid_body));
            assert!(node.rigid_body().is_some());

            node.set_collider(None);
            assert!(node.collider().is_none());

            node.set_rigid_body(None);
            assert!(node.rigid_body().is_none());
        }

        node.set_collider(Some(&collider));
        node.set_rigid_body(Some(&rigid_body));

        Script {
            is_first_update: Cell::new(true),
            node,
        }
    }

    fn update(&self, _delta: f32) {
        log(LogLevel::Info, "Called script update!");

        if self.is_first_update.get() {
            self.is_first_update.set(false);
            return;
        }

        // Ensure transform is updated from physics behaviors.
        if self.node.transform().translation.y >= 0.0 {
            panic_log("Dynamic node translation not updated");
        }
    }
}

fn panic_log(err: &str) {
    log(LogLevel::Error, err);
    panic!("{}", err);
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
