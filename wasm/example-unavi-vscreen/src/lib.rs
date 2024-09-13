use std::f32::consts::{FRAC_PI_2, PI};

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

const DIST: f32 = 8.0;

impl GuestScript for Script {
    fn new() -> Self {
        let mut screens = Vec::default();

        spawn_examples(
            Transform {
                translation: Vec3::new(DIST, 0.0, 0.0),
                rotation: Quat::from_rotation_y(-FRAC_PI_2),
                ..Default::default()
            },
            &mut screens,
        );
        spawn_examples(
            Transform {
                translation: Vec3::new(-DIST, 0.0, 0.0),
                rotation: Quat::from_rotation_y(FRAC_PI_2),
                ..Default::default()
            },
            &mut screens,
        );
        spawn_examples(
            Transform {
                translation: Vec3::new(0.0, 0.0, DIST),
                rotation: Quat::from_rotation_y(PI),
                ..Default::default()
            },
            &mut screens,
        );
        spawn_examples(
            Transform {
                translation: Vec3::new(0.0, 0.0, -DIST),
                ..Default::default()
            },
            &mut screens,
        );

        Self { screens }
    }

    fn update(&self, delta: f32) {
        for screen in self.screens.iter() {
            screen.update(delta);
        }
    }
}

fn spawn_examples(transform: Transform, screens: &mut Vec<Screen>) {
    // Background
    let bg = Rectangle::new(Vec2::new(8.0, 3.0)).to_physics_node();
    bg.set_transform(transform);
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
        translation: Vec3::new(-2.0, 0.0, 0.01),
        ..Default::default()
    });
    bg.add_child(&screen.root().root());

    for _ in 0..6 {
        let child = Screen::new(ScreenShape::Circle(0.08));
        screen.add_child(&child);
    }

    screens.push(screen);

    // Rectangle screen
    let screen = Screen::new(ScreenShape::Rectangle(Vec2::new(0.4, 0.2)));
    screen.set_visible(true);
    screen.root().root().set_transform(Transform {
        translation: Vec3::new(2.0, 0.0, 0.01),
        ..Default::default()
    });
    bg.add_child(&screen.root().root());

    for _ in 0..2 {
        let child = Screen::new(ScreenShape::Circle(0.08));
        screen.add_child(&child);
    }

    screens.push(screen);
}

struct GuestImpl;

impl Guest for GuestImpl {
    type Script = Script;
}

bindings::export!(GuestImpl with_types_in bindings);
