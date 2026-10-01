use bevy::prelude::*;
use hsd::attributes::name::NameAttr;

use crate::attributes::apply_simple;

pub fn apply(
    commands: &mut Commands,
    prim: Entity,
    payload: Option<&[u8]>,
) -> Result<(), postcard::Error> {
    apply_simple::<NameAttr, Name>(commands, prim, payload, |attr| Some(Name::new(attr.0)))
}
