use crate::{
    exports::wired::script::guest_api::{Guest, GuestScript},
    unavi::shapes::api::{Capsule, Cone, Cuboid, Cylinder, Sphere, Torus},
    wired::{math::types::Vec3, scene::context::self_document},
};

wit_bindgen::generate!({
    generate_all,
});

struct World;

impl Guest for World {
    type Script = Script;
}

struct Script;

impl GuestScript for Script {
    fn new() -> Self {
        let doc = self_document();
        let spacing = 1.5_f32;

        let meshes = [
            Capsule::new(0.3, 0.8).mesh(),
            Cone::new(0.4, 0.8).mesh(),
            Cuboid::new(0.7, 0.7, 0.7).mesh(),
            Cylinder::new(0.3, 0.8).mesh(),
            Sphere::new(0.4).mesh(),
            Torus::new(0.15, 0.4).mesh(),
        ];

        let count = meshes.len() as f32;
        let start = -(count - 1.0) * spacing / 2.0;

        for (i, mesh) in meshes.into_iter().enumerate() {
            let mat = doc.create_material();
            mat.set_base_color(&[0.3, 0.4, 0.8, 1.0]);
            let node = doc.create_node();
            node.set_material(Some(mat));
            node.set_translation(Vec3 {
                x: (i as f32).mul_add(spacing, start),
                y: 0.0,
                z: 0.0,
            });
            node.set_mesh(Some(mesh));
        }

        Self
    }

    fn tick(&self) {}

    fn render(&self) {}

    fn drop(&self) {}
}

export!(World);
