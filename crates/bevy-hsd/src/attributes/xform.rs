use bevy::{
    math::{Quat, Vec3},
    transform::components::Transform,
};
use loro::LoroValue;
use loro_surgeon::Hydrate;

#[derive(Debug, Clone)]
pub struct XformAttr {
    pub rotate: Option<Quat>,
    pub scale: Option<Vec3>,
    pub translate: Option<Vec3>,
}

impl XformAttr {
    #[must_use]
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
    rotate: Option<[f64; 4]>,
    scale: Option<[f64; 3]>,
    translate: Option<[f64; 3]>,
}

impl Hydrate for XformAttr {
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
