use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::{
        scene::api::Scene,
        shapes::api::{Cuboid, Sphere, Vec3},
    },
    wired::math::types::Transform,
};

#[allow(warnings)]
mod bindings;
mod wired_math_impls;

struct Script;

impl GuestScript for Script {
    fn new() -> Self {
        let scene = Scene::new();

        let cuboid = Cuboid::new(Vec3::new(1.0, 0.5, 1.5)).to_physics_node();
        cuboid.set_transform(Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)));
        scene.add_node(&cuboid);

        let sphere = Sphere::new_ico(0.5).to_physics_node();
        sphere.set_transform(Transform::from_translation(Vec3::new(1.5, 0.0, 2.0)));
        scene.add_node(&sphere);

        let sphere = Sphere::new_uv(0.5).to_physics_node();
        sphere.set_transform(Transform::from_translation(Vec3::new(1.5, 0.0, 0.0)));
        scene.add_node(&sphere);

        Script
    }

    fn update(&self, _delta: f32) {}
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
