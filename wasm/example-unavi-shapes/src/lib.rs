use std::f32::consts::{FRAC_PI_2, PI};

use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::shapes::api::{Axes, Circle, Cuboid, Cylinder, Ellipse, Rectangle, Sphere},
    wired::{
        math::types::{Quat, Transform, Vec2, Vec3},
        scene::scene::Scene,
    },
};

#[allow(warnings)]
mod bindings;
mod wired_math_impls;

struct Script {
    _scene: Scene,
}

impl GuestScript for Script {
    fn new() -> Self {
        let scene = Scene::new();

        {
            let translation = Vec3::new(3.0, 0.0, 0.0);
            let axes = Axes::new().to_node();
            axes.set_transform(Transform::from_translation(translation));
            scene.add_node(&axes);

            let cuboid = Cuboid::new(Vec3::new(1.0, 0.5, 1.5)).to_physics_node();
            axes.add_child(&cuboid);
        }

        {
            let translation = Vec3::new(1.5, 0.0, 2.0);
            let axes = Axes::new().to_node();
            axes.set_transform(Transform::from_translation(translation));
            scene.add_node(&axes);

            let sphere = Sphere::new_ico(0.5).to_physics_node();
            axes.add_child(&sphere);
        }

        {
            let translation = Vec3::new(1.5, 0.0, 0.0);
            let axes = Axes::new().to_node();
            axes.set_transform(Transform::from_translation(translation));
            scene.add_node(&axes);

            let sphere = Sphere::new_uv(0.5).to_physics_node();
            axes.add_child(&sphere);
        }

        {
            let translation = Vec3::default();
            let axes = Axes::new().to_node();
            axes.set_transform(Transform::from_translation(translation));
            scene.add_node(&axes);

            let cylinder = Cylinder::new(0.5, 1.0).to_physics_node();
            axes.add_child(&cylinder);
        }

        {
            let translation = Vec3::new(-1.5, 0.0, 0.0);
            let axes = Axes::new().to_node();
            axes.set_transform(Transform::from_translation(translation));
            scene.add_node(&axes);

            let rectangle = Rectangle::new(Vec2::splat(1.0)).to_physics_node();
            rectangle.set_transform(Transform::from_rotation(Quat::from_rotation_y(PI)));
            axes.add_child(&rectangle);
        }

        {
            let translation = Vec3::new(-3.0, 0.0, 0.0);
            let axes = Axes::new().to_node();
            axes.set_transform(Transform::from_translation(translation));
            scene.add_node(&axes);

            let circle = Circle::new(0.5).to_physics_node();
            circle.set_transform(Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)));
            axes.add_child(&circle);
        }

        {
            let translation = Vec3::new(-4.5, 0.0, 0.0);
            let axes = Axes::new().to_node();
            axes.set_transform(Transform::from_translation(translation));
            scene.add_node(&axes);

            let ellipse = Ellipse::new(Vec2::new(0.5, 0.75)).to_node();
            ellipse.set_transform(Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)));
            axes.add_child(&ellipse);
        }

        Script { _scene: scene }
    }

    fn update(&self, _delta: f32) {}
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
