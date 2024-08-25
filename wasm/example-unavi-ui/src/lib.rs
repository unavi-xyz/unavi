use std::{cell::Cell, f32::consts::PI};

use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::{
        layout::container::{Alignment, Container},
        scene::api::{Root, Scene},
        shapes::api::Rectangle,
        ui::{button::Button, text::TextBox},
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
    clock: TextBox,
    elapsed: Cell<f32>,
}

const LENGTH: f32 = 5.0;
const HEIGHT: f32 = 3.0;

impl GuestScript for Script {
    fn new() -> Self {
        let scene = Scene::new();

        let container = Container::new(Vec3::new(LENGTH, HEIGHT, 0.1));
        container.set_align_z(Alignment::End);
        container.root().set_transform(Transform {
            translation: Vec3::new(0.0, 0.0, -8.0),
            rotation: Quat::from_rotation_y(PI),
            ..Default::default()
        });

        scene.add_node(&container.root());

        {
            let bg = Rectangle::new(Vec2::new(LENGTH, HEIGHT)).to_physics_node();
            container.inner().add_child(&bg);

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
        }

        let button = Button::new(Container::new(Vec3::new(1.0, 0.5, 0.2)));
        container.add_child(&button.root());

        let clock_container = Container::new(Vec3::new(2.0, 0.5, 0.1));
        clock_container.inner().set_transform(Transform {
            translation: Vec3::new(-1.0, 1.0, 0.0),
            rotation: Quat::from_rotation_y(PI),
            scale: Vec3::splat(0.2),
        });
        container.add_child(&clock_container);
        let clock = TextBox::new(clock_container);

        Root::add_scene(&scene);

        Script {
            button,
            clock,
            elapsed: Cell::default(),
        }
    }

    fn update(&self, delta: f32) {
        let elapsed = self.elapsed.get();
        self.elapsed.set(elapsed + delta);

        self.clock
            .text()
            .set_text(&format!("Elapsed: {:.2}", elapsed));

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
