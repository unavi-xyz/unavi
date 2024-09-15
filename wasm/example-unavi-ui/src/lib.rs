use std::cell::Cell;

use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::{
        layout::container::{Alignment, Container},
        scene::api::{Root, Scene},
        shapes::api::Rectangle,
        ui::{
            api::update_ui,
            button::Button,
            text::{Text, TextBox},
        },
    },
    wired::{
        log::api::{log, LogLevel},
        math::types::{Transform, Vec2, Vec3},
        scene::material::{Color, Material},
    },
};

#[allow(warnings)]
mod bindings;
mod wired_math_impls;

struct Script {
    button: Button,
    clock: Text,
    time: Cell<f32>,
}

const LENGTH: f32 = 10.0;
const HEIGHT: f32 = 3.0;

impl GuestScript for Script {
    fn new() -> Self {
        let scene = Scene::new();

        // Background container
        let bg = Container::new(Vec3::new(LENGTH, HEIGHT, 0.1));
        bg.root()
            .set_transform(Transform::from_translation(Vec3::new(0.0, 0.2, -8.0)));
        scene.add_node(&bg.root());

        {
            let rect = Rectangle::new(Vec2::new(LENGTH, HEIGHT)).to_physics_node();
            bg.root().add_child(&rect);

            let material = Material::new();
            material.set_color(Color {
                r: 0.1,
                g: 0.1,
                b: 0.2,
                a: 1.0,
            });
            rect.mesh()
                .unwrap()
                .list_primitives()
                .iter()
                .for_each(|p| p.set_material(Some(&material)));
        }

        // Button
        let button = Button::new(Container::new(Vec3::new(1.0, 0.5, 0.2)));
        bg.add_child(&button.root());

        // Text
        let clock = {
            let text = Text::new("");
            let node = scene.create_node();
            node.set_mesh(Some(&text.mesh()));
            node.set_transform(Transform::from_translation(Vec3::new(4.0, 1.0, 0.01)));
            bg.inner().add_child(&node);
            text
        };

        // TextBox
        {
            let size = Vec2::new(1.5, 1.0);

            let container = Container::new(Vec3::new(size.x, size.y, 0.01));
            container.set_align_y(Alignment::End);
            container
                .root()
                .set_transform(Transform::from_translation(Vec3::new(2.0, 0.0, 0.01)));
            bg.root().add_child(&container.root());

            let outline = Rectangle::new(size).to_physics_node();
            outline.set_mesh(None);
            container.root().add_child(&outline);

            let text = TextBox::new(container);
            text.text().set_alignment(Alignment::Start);
            text.set_text("The quick brown fox jumps over the lazy dog.");

            // TODO: Add buttons for controlling text settings
        }

        Root::add_scene(&scene);

        Script {
            button,
            clock,
            time: Cell::default(),
        }
    }

    fn update(&self, delta: f32) {
        update_ui(delta);

        let time = self.time.get();
        self.time.set(time + delta);

        self.clock.set_text(&format!("{:.0}", time));

        if self.button.pressed() {
            log(LogLevel::Info, "Button pressed!");
        }
    }
}

struct GuestImpl;

impl Guest for GuestImpl {
    type Script = Script;
}

bindings::export!(GuestImpl with_types_in bindings);
