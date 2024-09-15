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
mod wired_scene_impls;

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

const SIZE: f32 = 0.1;

fn spawn_examples(transform: Transform, screens: &mut Vec<Screen>) {
    // Background
    let bg = Rectangle::new(Vec2::new(8.0, 3.0)).to_physics_node();
    bg.set_transform(transform);
    let material = Material::new();
    material.set_color(Color::RED);
    for primitive in bg.mesh().unwrap().list_primitives() {
        primitive.set_material(Some(&material))
    }

    // Circle screen
    let screen = Screen::new(ScreenShape::Circle(SIZE));
    screen.set_visible(true);
    screen.root().root().set_transform(Transform {
        translation: Vec3::new(-2.0, 0.0, 0.01),
        ..Default::default()
    });
    bg.add_child(&screen.root().root());

    spawn_children(&screen);

    screens.push(screen);

    // Rectangle screen
    let screen = Screen::new(ScreenShape::Rectangle(Vec2::new(SIZE * 4.0, SIZE * 2.0)));
    screen.set_visible(true);
    screen.root().root().set_transform(Transform {
        translation: Vec3::new(2.0, 0.0, 0.01),
        ..Default::default()
    });
    bg.add_child(&screen.root().root());

    spawn_children(&screen);

    screens.push(screen);
}

fn spawn_children(screen: &Screen) {
    for i in 0..6 {
        let shape = if i % 2 == 0 {
            ScreenShape::Circle(SIZE)
        } else {
            ScreenShape::Rectangle(Vec2::splat(SIZE * 2.0))
        };
        let child = Screen::new(shape);
        screen.add_child(&child);
    }
}

struct GuestImpl;

impl Guest for GuestImpl {
    type Script = Script;
}

bindings::export!(GuestImpl with_types_in bindings);
