use std::{cell::Cell, f32::consts::FRAC_PI_2};

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
    time: Cell<f32>,
}

const LENGTH: f32 = 5.0;
const HEIGHT: f32 = 3.0;

impl GuestScript for Script {
    fn new() -> Self {
        let scene = Scene::new();

        // Background container
        let container = Container::new(Vec3::new(LENGTH, HEIGHT, 0.1));
        container.set_align_z(Alignment::End);
        container.root().set_transform(Transform {
            translation: Vec3::new(0.0, 0.2, -8.0),
            rotation: Quat::from_rotation_x(FRAC_PI_2),
            ..Default::default()
        });

        scene.add_node(&container.root());

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

        // Button
        let button = Button::new(Container::new(Vec3::new(1.0, 0.5, 0.2)));
        button
            .root()
            .inner()
            .set_transform(Transform::from_rotation(Quat::from_rotation_x(-FRAC_PI_2)));
        container.add_child(&button.root());

        // TextBox
        let clock_container = Container::new(Vec3::new(2.0, 0.5, 0.01));
        clock_container.inner().set_transform(Transform {
            translation: Vec3::new(1.5, 0.01, -1.0),
            rotation: Quat::from_rotation_x(-FRAC_PI_2),
            ..Default::default()
        });
        container.add_child(&clock_container);

        let clock = TextBox::new(clock_container);

        // Text
        let label_text = Text::new("Elapsed:");
        let label = scene.create_node();
        clock.root().root().add_child(&label);
        label.set_mesh(Some(&label_text.mesh()));
        label.set_transform(Transform {
            translation: Vec3::new(0.5, 0.01, -1.0),
            rotation: Quat::from_rotation_x(-FRAC_PI_2),
            ..Default::default()
        });

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

        self.clock.text().set_text(&format!("{:.0}", time));

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
