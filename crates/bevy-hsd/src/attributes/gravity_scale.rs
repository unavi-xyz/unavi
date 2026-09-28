use avian3d::prelude::GravityScale;
use bevy::prelude::*;
use hsd::attributes::gravity_scale::GravityScaleAttr;

use crate::attributes::{
    ParseError,
    apply_simple,
};

pub fn apply(
    commands: &mut Commands,
    prim: Entity,
    payload: Option<&[u8]>,
) -> Result<(), ParseError> {
    apply_simple::<GravityScaleAttr, GravityScale>(commands, prim, payload, |attr| {
        let scale = attr.scale as f32;
        if scale.is_finite() {
            Some(GravityScale(scale))
        } else {
            warn!("gravity_scale: scale must be finite (got {scale})");
            None
        }
    })
}
