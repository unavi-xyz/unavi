use std::f32::consts::PI;

use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::{
        layout::container::Container,
        scene::api::{Root, Scene},
        shapes::api::Rectangle,
        ui::{button::Button, text::Text},
    },
    wired::{
        log::api::{log, LogLevel},
        math::types::{Quat, Transform, Vec2, Vec3},
        scene::material::{Color, Material},
    },
};

#[allow(warnings)]
mod bindings;
mod wired_math_impls;

struct Script {
    button: Button,
}

const LENGTH: f32 = 5.0;
const HEIGHT: f32 = 3.0;

impl GuestScript for Script {
    fn new() -> Self {
        let scene = Scene::new();

        let bg = Rectangle::new(Vec2::new(LENGTH, HEIGHT)).to_physics_node();
        bg.set_transform(Transform {
            translation: Vec3::new(0.0, 0.0, -8.0),
            rotation: Quat::from_rotation_y(PI),
            ..Default::default()
        });

        let material = Material::new();
        material.set_color(Color {
            r: 0.1,
            g: 0.1,
            b: 0.2,
            a: 1.0,
        });
        bg.mesh()
            .unwrap()
            .list_primitives()
            .iter()
            .for_each(|p| p.set_material(Some(&material)));

        scene.add_node(&bg);

        let container = Container::new(Vec3::new(LENGTH, HEIGHT, 0.1));
        bg.add_child(&container.root());

        let button = Button::new(Container::new(Vec3::new(1.0, 0.5, 0.2)));
        container.inner().add_child(&button.root().root());

        let text = Text::new("Hello world!").to_node().unwrap();
        text.set_transform(Transform {
            translation: Vec3::new(0.0, 0.5, -0.3),
            rotation: Quat::from_rotation_y(PI),
            scale: Vec3::splat(0.5),
        });
        button.root().inner().add_child(&text);

        Root::add_scene(&scene);

        Script { button }
    }

    fn update(&self, _delta: f32) {
        if self.button.pressed() {
            log(LogLevel::Info, "Button pressed!");
        }
    }
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
