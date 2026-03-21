use wired_prelude::wired_math::types::Vec3;

use crate::{
    unavi::shapes::api::{Capsule, Cone, Cuboid, Cylinder, Sphere, Torus},
    wired::scene::context::self_document,
};

wired_prelude::generate_script!(Script);

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

        let mat = doc.create_material();
        mat.set_base_color(&[0.3, 0.4, 0.8, 1.0]);

        for (i, mesh) in meshes.into_iter().enumerate() {
            let node = doc.create_node();
            node.set_material(Some(&mat));
            node.set_translation(Vec3::new((i as f32).mul_add(spacing, start), 0.0, 0.0));
            node.set_mesh(Some(&mesh));
        }

        Self
    }

    fn tick(&self) {}

    fn render(&self) {}

    fn drop(&self) {}
}
