use std::f32::consts::PI;

use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::{
        scene::api::{Root, Scene},
        shapes::api::{Axes, Circle, Cuboid, Cylinder, Ellipse, Rectangle, Sphere},
    },
    wired::math::types::{Quat, Transform, Vec2, Vec3},
};

#[allow(warnings)]
mod bindings;
mod wired_math_impls;

struct Script;

impl GuestScript for Script {
    fn new() -> Self {
        let scene = Scene::new();

        {
            let translation = Vec3::new(3.0, 0.0, 0.0);
            let node = Axes::new().to_node();
            node.set_transform(Transform::from_translation(translation));

            let cuboid = Cuboid::new(Vec3::new(1.0, 0.5, 1.5)).to_physics_node();
            cuboid.set_transform(Transform::from_translation(translation));
            scene.add_node(&cuboid);
        }

        {
            let translation = Vec3::new(1.5, 0.0, 2.0);
            let node = Axes::new().to_node();
            node.set_transform(Transform::from_translation(translation));

            let sphere = Sphere::new_ico(0.5).to_physics_node();
            sphere.set_transform(Transform::from_translation(translation));
            scene.add_node(&sphere);
        }

        {
            let translation = Vec3::new(1.5, 0.0, 0.0);
            let node = Axes::new().to_node();
            node.set_transform(Transform::from_translation(translation));

            let sphere = Sphere::new_uv(0.5).to_physics_node();
            sphere.set_transform(Transform::from_translation(translation));
            scene.add_node(&sphere);
        }

        {
            let translation = Vec3::default();
            let node = Axes::new().to_node();
            node.set_transform(Transform::from_translation(translation));

            let cylinder = Cylinder::new(0.5, 1.0).to_physics_node();
            scene.add_node(&cylinder);
        }

        {
            let translation = Vec3::new(-1.5, 0.0, 0.0);
            let node = Axes::new().to_node();
            node.set_transform(Transform::from_translation(translation));

            let rectangle = Rectangle::new(Vec2::splat(1.0)).to_physics_node();
            rectangle.set_transform(Transform {
                translation,
                rotation: Quat::from_rotation_y(PI),
                ..Default::default()
            });
            scene.add_node(&rectangle);
        }

        {
            let translation = Vec3::new(-3.0, 0.0, 0.0);
            let node = Axes::new().to_node();
            node.set_transform(Transform::from_translation(translation));

            let circle = Circle::new(0.5).to_physics_node();
            circle.set_transform(Transform::from_translation(translation));
            scene.add_node(&circle);
        }

        {
            let translation = Vec3::new(-4.5, 0.0, 0.0);
            let node = Axes::new().to_node();
            node.set_transform(Transform::from_translation(translation));

            let ellipse = Ellipse::new(Vec2::new(0.5, 0.75)).to_node();
            ellipse.set_transform(Transform::from_translation(translation));
            scene.add_node(&ellipse);
        }

        Root::add_scene(&scene);

        Script
    }

    fn update(&self, _delta: f32) {}
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
