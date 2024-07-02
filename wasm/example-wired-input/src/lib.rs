use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::shapes::api::create_cuboid,
    wired::{
        input::handler::InputHandler,
        log::api::{log, LogLevel},
        math::types::Vec3,
        physics::types::{Collider, Shape},
        scene::{
            material::{create_material, Color, Material},
            node::create_node,
        },
    },
};
use rand::Rng;

#[allow(warnings)]
mod bindings;
mod wired_math_impls;

struct Script {
    handler: InputHandler,
    material: Material,
}

impl GuestScript for Script {
    fn new() -> Self {
        let node = create_node();

        let size = Vec3::splat(1.0);
        let collider = Collider::new(Shape::Cuboid(size));
        node.set_collider(Some(&collider));

        let mesh = create_cuboid(size);
        node.set_mesh(Some(&mesh));

        let material = create_material();
        for primitive in mesh.list_primitives() {
            primitive.set_material(Some(&material));
        }

        log(LogLevel::Info, "Registering input handler");
        let handler = InputHandler::new();
        node.set_input_handler(Some(&handler));

        Script { handler, material }
    }

    fn update(&self, _delta: f32) {
        while let Some(event) = self.handler.handle_input() {
            log(LogLevel::Info, &format!("Got input: {:?}", event));

            let mut rng = rand::thread_rng();
            let r = rng.gen_range(0.0..1.0);
            let g = rng.gen_range(0.0..1.0);
            let b = rng.gen_range(0.0..1.0);
            let a = rng.gen_range(0.2..1.0);

            self.material.set_color(Color { r, g, b, a });
        }
    }
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
