use std::f32::consts::FRAC_PI_2;

use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::{
        scene::api::{Root, Scene},
        shapes::api::{Cuboid, Cylinder, Rectangle, Sphere},
    },
    wired::{
        math::types::{Quat, Transform, Vec2, Vec3},
        scene::{
            material::{Color, Material},
            node::Node,
        },
    },
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
            spawn_axis(translation);

            let cuboid = Cuboid::new(Vec3::new(1.0, 0.5, 1.5)).to_physics_node();
            cuboid.set_transform(Transform::from_translation(translation));
            scene.add_node(&cuboid);
        }

        {
            let translation = Vec3::new(1.5, 0.0, 2.0);
            spawn_axis(translation);

            let sphere = Sphere::new_ico(0.5).to_physics_node();
            sphere.set_transform(Transform::from_translation(translation));
            scene.add_node(&sphere);
        }

        {
            let translation = Vec3::new(1.5, 0.0, 0.0);
            spawn_axis(translation);

            let sphere = Sphere::new_uv(0.5).to_physics_node();
            sphere.set_transform(Transform::from_translation(translation));
            scene.add_node(&sphere);
        }

        {
            let translation = Vec3::default();
            spawn_axis(translation);

            let cylinder = Cylinder::new(0.5, 1.0).to_physics_node();
            scene.add_node(&cylinder);
        }

        {
            let translation = Vec3::new(-1.5, 0.0, 0.0);
            spawn_axis(translation);

            let rectangle = Rectangle::new(Vec2::splat(1.0)).to_physics_node();
            rectangle.set_transform(Transform::from_translation(translation));
            scene.add_node(&rectangle);
        }

        Root::add_scene(&scene);

        Script
    }

    fn update(&self, _delta: f32) {}
}

fn spawn_axis(translation: Vec3) {
    let shape = Cylinder::new(0.01, 2.0);
    shape.set_resolution(8);

    let root = Node::new();
    root.set_transform(Transform::from_translation(translation));

    let red = Material::new();
    red.set_color(Color {
        r: 1.0,
        g: 0.1,
        b: 0.1,
        a: 0.9,
    });
    let green = Material::new();
    green.set_color(Color {
        r: 0.1,
        g: 1.0,
        b: 0.1,
        a: 0.9,
    });
    let blue = Material::new();
    blue.set_color(Color {
        r: 0.1,
        g: 0.1,
        b: 1.0,
        a: 0.9,
    });

    let x = shape.to_node();
    root.add_child(&x);
    x.mesh().unwrap().list_primitives()[0].set_material(Some(&red));
    x.set_transform(Transform::from_rotation(Quat::from_rotation_z(FRAC_PI_2)));

    let y = shape.to_node();
    root.add_child(&y);
    y.mesh().unwrap().list_primitives()[0].set_material(Some(&green));

    let z = shape.to_node();
    root.add_child(&z);
    z.mesh().unwrap().list_primitives()[0].set_material(Some(&blue));
    z.set_transform(Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)));
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
