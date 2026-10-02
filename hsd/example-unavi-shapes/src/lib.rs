use wired_guest::math::{
    Color,
    Quat,
    Transform,
    Vec3,
};

use crate::{
    unavi::shapes::api::{
        Capsule,
        Cone,
        Cuboid,
        Cylinder,
        Sphere,
        Torus,
    },
    wired::scene::{
        document::script_document,
        properties::{
            Material,
            Property,
        },
    },
};

wired_guest::generate_script!(Script);

struct Script;

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let doc = script_document()?;

        let spacing = 1.5_f32;
        let prims = [
            Capsule::new(0.3, 0.8).mesh()?,
            Cone::new(0.4, 0.8).mesh()?,
            Cuboid::new(Vec3::splat(0.7)).mesh()?,
            Cylinder::new(0.3, 0.8).mesh()?,
            Sphere::new(0.4).mesh()?,
            Torus::new(0.15, 0.4).mesh()?,
        ];

        let count = prims.len() as f32;
        let start = -(count - 1.0) * spacing / 2.0;

        let mut batch = doc.local();
        for (i, prim) in prims.into_iter().enumerate() {
            let translation = Vec3::new((i as f32).mul_add(spacing, start), 0.0, 0.0);
            batch = batch
                .set(
                    prim,
                    Property::Material(Material::solid(Color::rgb(0.3, 0.4, 0.8))),
                )
                .set(
                    prim,
                    Property::Transform(Transform::new(translation, Quat::IDENTITY, Vec3::ONE)),
                );
        }
        batch.flush()?;

        Ok(Self)
    }
}
