use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    wired::{
        log::api::{log, LogLevel},
        math::types::Vec3,
        physics::types::{Collider, RigidBody, RigidBodyType, Shape},
        scene::{node::Node, scene::Scene},
    },
};

#[allow(warnings)]
mod bindings;
mod wired_math_impls;

struct Script {
    spheres: Vec<Node>,
    _scene: Scene,
}

impl GuestScript for Script {
    fn new() -> Self {
        let scene = Scene::new();

        // Ground.
        {
            let size = Vec3::new(10.0, 0.5, 10.0);

            let ground = Node::new();
            let collider = Collider::new(Shape::Cuboid(size));
            let rigid_body = RigidBody::new(RigidBodyType::Fixed);

            ground.set_collider(Some(&collider));
            ground.set_rigid_body(Some(&rigid_body));

            scene.add_node(&ground);
        }

        // Spheres.
        let mut spheres = Vec::default();

        for i in 1..5 {
            let radius = 0.25;

            let ball = Node::new();

            let mut tr = ball.transform();
            tr.translation.x += (i as f32) / 4.0;
            tr.translation.y += i as f32;
            ball.set_transform(tr);

            let collider = Collider::new(Shape::Sphere(radius));
            let rigid_body = RigidBody::new(RigidBodyType::Dynamic);

            ball.set_collider(Some(&collider));
            ball.set_rigid_body(Some(&rigid_body));

            scene.add_node(&ball);
            spheres.push(ball);
        }

        // Collider only.
        {
            let node = Node::new();
            let collider = Collider::new(Shape::Sphere(1.0));
            node.set_collider(Some(&collider));

            scene.add_node(&node);
        }

        Script {
            spheres,
            _scene: scene,
        }
    }

    fn update(&self, _delta: f32) {
        let translation = self.spheres[0].transform().translation;
        log(
            LogLevel::Info,
            &format!("Ball translation: {}", translation),
        );
    }
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
