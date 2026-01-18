use bevy::transform::components::Transform;
use loro::LoroValue;

use crate::attributes::Attribute;

pub struct Xform(Transform);

impl Attribute for Xform {
    fn parse(_value: LoroValue) -> Self {
        Self(Transform::default())
    }
}
