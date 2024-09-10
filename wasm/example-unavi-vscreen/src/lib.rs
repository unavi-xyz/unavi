use std::f32::consts::FRAC_PI_2;

use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::{
        shapes::api::Rectangle,
        vscreen::screen::{Screen, ScreenShape},
    },
    wired::{
        math::types::{Quat, Transform, Vec2, Vec3},
        scene::material::{Color, Material},
    },
};

#[allow(warnings)]
mod bindings;
mod wired_math_impls;

struct Script {
    screens: Vec<Screen>,
}

impl GuestScript for Script {
    fn new() -> Self {
        let mut screens = Vec::default();

        // Background
        let bg = Rectangle::new(Vec2::new(8.0, 3.0)).to_physics_node();
        bg.set_transform(Transform {
            translation: Vec3::new(0.0, 0.0, -8.0),
            rotation: Quat::from_rotation_x(FRAC_PI_2),
            ..Default::default()
        });
        let material = Material::new();
        material.set_color(Color {
            r: 0.5,
            g: 0.1,
            b: 0.2,
            a: 1.0,
        });
        for primitive in bg.mesh().unwrap().list_primitives() {
            primitive.set_material(Some(&material))
        }

        // Circle screen
        let screen = Screen::new(ScreenShape::Circle(0.1));
        screen.set_visible(true);
        screen.root().root().set_transform(Transform {
            translation: Vec3::new(-2.0, 0.0, -7.99),
            rotation: Quat::from_rotation_x(FRAC_PI_2),
            ..Default::default()
        });
        screens.push(screen);

        // Rectangle screen
        let screen = Screen::new(ScreenShape::Rectangle(Vec2::new(0.4, 0.2)));
        screen.set_visible(true);
        screen.root().root().set_transform(Transform {
            translation: Vec3::new(0.0, 0.0, -7.99),
            rotation: Quat::from_rotation_x(FRAC_PI_2),
            ..Default::default()
        });
        screens.push(screen);

        Self { screens }
    }

    fn update(&self, delta: f32) {
        for screen in self.screens.iter() {
            screen.update(delta);
        }
    }
}

struct GuestImpl;

impl Guest for GuestImpl {
    type Script = Script;
}

bindings::export!(GuestImpl with_types_in bindings);
