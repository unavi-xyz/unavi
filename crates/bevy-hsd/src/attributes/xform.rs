use bevy::{
    math::{Quat, Vec3},
    transform::components::Transform,
};
use loro::LoroValue;
use loro_surgeon::Hydrate;

use crate::attributes::Attribute;

pub struct Xform(pub Transform);

#[derive(Debug, Clone, Default, Hydrate)]
struct XformRaw {
    #[loro(with = "wired_schemas::conv::float_slice")]
    rotate: [f64; 4],
    #[loro(with = "wired_schemas::conv::float_slice")]
    scale: [f64; 3],
    #[loro(with = "wired_schemas::conv::float_slice")]
    translate: [f64; 3],
}

impl Hydrate for Xform {
    #[expect(clippy::cast_possible_truncation)]
    fn hydrate(value: &LoroValue) -> Result<Self, loro_surgeon::HydrateError> {
        let raw = XformRaw::hydrate(value)?;

        let tr = Transform::default()
            .with_rotation(Quat::from_xyzw(
                raw.rotate[0] as f32,
                raw.rotate[1] as f32,
                raw.rotate[2] as f32,
                raw.rotate[3] as f32,
            ))
            .with_scale(Vec3::new(
                raw.scale[0] as f32,
                raw.scale[1] as f32,
                raw.scale[2] as f32,
            ))
            .with_translation(Vec3::new(
                raw.translate[0] as f32,
                raw.translate[1] as f32,
                raw.translate[2] as f32,
            ));

        Ok(Self(tr))
    }
}

impl Attribute for Xform {
    fn merge(self, next: Self) -> Self {
        next
    }
}
