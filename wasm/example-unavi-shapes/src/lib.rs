use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::{
        scene::api::{add_scene, Node, Scene},
        shapes::api::{create_cuboid, create_sphere, Vec3},
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
        add_scene(&scene);

        let cuboid_node = Node::new();
        cuboid_node.set_transform(Transform {
            translation: Vec3::new(3.0, 0.0, 0.0),
            ..Default::default()
        });
        scene.add_node(&cuboid_node);
        let cuboid_mesh = create_cuboid(Vec3::splat(1.0));
        cuboid_node.set_mesh(Some(&cuboid_mesh));

        let sphere_node = Node::new();
        scene.add_node(&sphere_node);
        sphere_node.set_transform(Transform {
            translation: Vec3::new(1.5, 0.0, 0.0),
            ..Default::default()
        });
        let sphere_mesh = create_sphere(0.5, 32, 18);
        sphere_node.set_mesh(Some(&sphere_mesh));

        Script
    }

    fn update(&self, _delta: f32) {}
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
