use std::cell::RefCell;

use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::shapes::api::create_cuboid,
    wired::{
        log::api::{log, LogLevel},
        math::types::Vec3,
        scene::{
            material::{create_material, Color, Material},
            node::{create_node, Node},
        },
    },
};

#[allow(warnings)]
mod bindings;
mod wired_math_impls;

struct Script {
    color_delta: RefCell<Color>,
    material: Material,
    node: Node,
}

impl GuestScript for Script {
    fn new() -> Self {
        log(LogLevel::Info, "Creating node");
        let node = create_node();

        log(LogLevel::Info, "Creating mesh");
        let mesh = create_cuboid(Vec3::splat(1.0));

        log(LogLevel::Info, "Setting mesh");
        node.set_mesh(Some(&mesh));

        log(LogLevel::Info, "Creating material");
        let material = create_material();

        log(LogLevel::Info, "Setting color");
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

        Script {
            color_delta: RefCell::new(Color {
                r: 0.01,
                g: 0.002,
                b: -0.003,
                a: 0.0,
            }),
            material,
            node,
        }
    }

    fn update(&self, delta: f32) {
        let mut color = self.material.color();
        let mut color_delta = self.color_delta.borrow_mut();

        color.r += color_delta.r;
        color.g += color_delta.g;
        color.b += color_delta.b;
        color.a += color_delta.a;

        color_delta.r = cycle(color_delta.r, color.r);
        color_delta.g = cycle(color_delta.g, color.g);
        color_delta.b = cycle(color_delta.b, color.b);
        color_delta.a = cycle(color_delta.a, color.a);

        self.material.set_color(color);

        let mut transform = self.node.transform();

        let mut quat = glam::Quat::from_xyzw(
            transform.rotation.x,
            transform.rotation.y,
            transform.rotation.z,
            transform.rotation.w,
        );

        quat *= glam::Quat::from_rotation_y(delta);

        transform.rotation.x = quat.x;
        transform.rotation.y = quat.y;
        transform.rotation.z = quat.z;
        transform.rotation.w = quat.w;

        self.node.set_transform(transform);
    }
}

fn cycle(delta: f32, value: f32) -> f32 {
    if value >= 1.0 || value <= 0.0 {
        delta * -1.0
    } else {
        delta
    }
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
