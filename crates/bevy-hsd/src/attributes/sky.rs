use bevy::prelude::*;
use hsd::attributes::sky::SkyAttr;

use crate::attributes::{
    apply_simple,
    values::from_color_vec,
};

/// The colour a space's sky takes.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Sky(pub Color);

pub fn apply(
    commands: &mut Commands,
    prim: Entity,
    payload: Option<&[u8]>,
) -> Result<(), postcard::Error> {
    apply_simple::<SkyAttr, Sky>(commands, prim, payload, |attr| {
        if attr.color.0.iter().all(|channel| channel.is_finite()) {
            Some(Sky(from_color_vec(Some(&attr.color), Color::BLACK)))
        } else {
            warn!("sky: color must be finite");
            None
        }
    })
}
