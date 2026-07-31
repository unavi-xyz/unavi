use wired_prelude::prelude::*;

use crate::unavi::shapes::api::{
    Capsule,
    Cone,
    Cuboid,
    Cylinder,
    Sphere,
    Torus,
};

wired_prelude::generate_script!(Script);

struct Script;

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let spacing = 1.5_f32;
        let prims = [
            Capsule::new(0.3, 0.8).mesh(),
            Cone::new(0.4, 0.8).mesh(),
            Cuboid::new(Vec3::splat(0.7)).mesh(),
            Cylinder::new(0.3, 0.8).mesh(),
            Sphere::new(0.4).mesh(),
            Torus::new(0.15, 0.4).mesh(),
        ];

        let count = prims.len() as f32;
        let start = -(count - 1.0) * spacing / 2.0;

        let material = wired::scene::types::Material {
            alpha_cutoff:               None,
            alpha_mode:                 None,
            base_color:                 Some(Color::rgb(0.3, 0.4, 0.8)),
            base_color_texture:         None,
            double_sided:               None,
            emissive:                   None,
            emissive_texture:           None,
            metallic:                   None,
            metallic_roughness_texture: None,
            normal_texture:             None,
            occlusion_texture:          None,
            roughness:                  None,
        };

        for (i, prim) in prims.into_iter().enumerate() {
            prim.set_material(Some(&material))?;
            prim.set_xform(Some(wired::scene::types::Xform {
                translation: Vec3::new((i as f32).mul_add(spacing, start), 0.0, 0.0),
                rotation:    Quat {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
                scale:       Vec3::splat(1.0),
            }))?;
        }

        Ok(Self)
    }
}
