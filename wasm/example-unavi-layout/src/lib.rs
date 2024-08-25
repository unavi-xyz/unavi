use std::f32::consts::PI;

use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::{
        layout::container::Container,
        scene::api::{Root, Scene},
        shapes::api::Rectangle,
    },
    wired::math::types::{Quat, Transform, Vec2, Vec3},
};

#[allow(warnings)]
mod bindings;
mod wired_math_impls;

struct Script {}

impl GuestScript for Script {
    fn new() -> Self {
        let scene = Scene::new();

        let bg = Rectangle::new(Vec2::new(5.0, 3.0)).to_physics_node();
        bg.set_transform(Transform {
            translation: Vec3::new(0.0, 0.0, -8.0),
            rotation: Quat::from_rotation_y(PI),
            ..Default::default()
        });
        scene.add_node(&bg);

        let _a = Container::new(Vec3::splat(1.0));

        Root::add_scene(&scene);

        Script {}
    }

    fn update(&self, _delta: f32) {}
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
