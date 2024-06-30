use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::shapes::api::create_cuboid,
    wired::{
        input::handler::SpatialHandler,
        log::api::{log, LogLevel},
        math::types::Vec3,
        scene::{material::create_material, node::create_node},
    },
};

#[allow(warnings)]
mod bindings;
mod wired_math_impls;

struct Script {
    handler: SpatialHandler,
}

impl GuestScript for Script {
    fn new() -> Self {
        let node = create_node();
        let mesh = create_cuboid(Vec3::splat(1.0));
        node.set_mesh(Some(&mesh));

        let material = create_material();
        for primitive in mesh.list_primitives() {
            primitive.set_material(Some(&material));
        }

        log(LogLevel::Info, "Registering spatial handler");

        let handler = SpatialHandler::new(&node);

        Script { handler }
    }

    fn update(&self, _delta: f32) {
        while let Some(event) = self.handler.handle_input() {
            log(
                LogLevel::Info,
                &format!("Handling input event: {:?}", event),
            );
        }
    }
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
