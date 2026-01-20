use bevy::{
    math::{Quat, Vec3},
    transform::components::Transform,
};
use loro::LoroValue;
use loro_surgeon::Hydrate;

use crate::attributes::Attribute;

#[derive(Clone)]
pub struct Xform {
    pub rotate: Option<Quat>,
    pub scale: Option<Vec3>,
    pub translate: Option<Vec3>,
}

impl Xform {
    pub fn into_transform(self) -> Transform {
        let mut tr = Transform::default();
        if let Some(v) = self.rotate {
            tr.rotation = v;
        }
        if let Some(v) = self.scale {
            tr.scale = v;
        }
        if let Some(v) = self.translate {
            tr.translation = v;
        }
        tr
    }
}

#[derive(Debug, Clone, Default, Hydrate)]
struct XformRaw {
    #[loro(with = "wired_schemas::conv::float_slice::optional")]
    rotate: Option<[f64; 4]>,
    #[loro(with = "wired_schemas::conv::float_slice::optional")]
    scale: Option<[f64; 3]>,
    #[loro(with = "wired_schemas::conv::float_slice::optional")]
    translate: Option<[f64; 3]>,
}

impl Hydrate for Xform {
    #[expect(clippy::cast_possible_truncation)]
    fn hydrate(value: &LoroValue) -> Result<Self, loro_surgeon::HydrateError> {
        let raw = XformRaw::hydrate(value)?;

        Ok(Self {
            rotate: raw
                .rotate
                .map(|v| Quat::from_xyzw(v[0] as f32, v[1] as f32, v[2] as f32, v[3] as f32)),
            scale: raw
                .scale
                .map(|v| Vec3::new(v[0] as f32, v[1] as f32, v[2] as f32)),
            translate: raw
                .translate
                .map(|v| Vec3::new(v[0] as f32, v[1] as f32, v[2] as f32)),
        })
    }
}

impl Attribute for Xform {
    fn merge(mut self, next: &Self) -> Self {
        if let Some(v) = next.rotate {
            self.rotate = Some(v);
        }
        if let Some(v) = next.scale {
            self.scale = Some(v);
        }
        if let Some(v) = next.translate {
            self.translate = Some(v);
        }
        self
    }
}
