use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::shapes::api::create_cuboid,
    wired::{
        gltf::{
            material::{create_material, Color},
            node::{create_node, Node},
        },
        log::api::{log, LogLevel},
        math::types::Vec3,
    },
};
use glam::Quat;

#[allow(warnings)]
mod bindings;
mod impls;

struct Script {
    node: Node,
}

impl GuestScript for Script {
    fn new() -> Self {
        println!("println: Constructing script!");

        log(LogLevel::Info, "Creating node");
        let node = create_node();

        log(LogLevel::Info, "Creating mesh");
        let mesh = create_cuboid(Vec3::splat(1.0));

        log(LogLevel::Info, "Creating material");
        let material = create_material();
        material.set_color(Color {
            r: 1.0,
            g: 0.5,
            b: 0.9,
            a: 1.0,
        });

        log(LogLevel::Info, "Setting material");
        for primitive in mesh.list_primitives() {
            primitive.set_material(Some(&material));
        }

        log(LogLevel::Info, "Setting mesh");
        node.set_mesh(Some(&mesh));

        Script { node }
    }

    fn update(&self, delta: f32) {
        let mut transform = self.node.transform();

        let mut quat = Quat::from_xyzw(
            transform.rotation.x,
            transform.rotation.y,
            transform.rotation.z,
            transform.rotation.w,
        );

        quat *= Quat::from_rotation_y(delta);

        transform.rotation.x = quat.x;
        transform.rotation.y = quat.y;
        transform.rotation.z = quat.z;
        transform.rotation.w = quat.w;

        self.node.set_transform(transform);
    }
}

struct Api;

impl Guest for Api {
    type Script = Script;
}

bindings::export!(Api with_types_in bindings);
