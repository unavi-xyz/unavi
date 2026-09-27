use bevy::prelude::*;
use hsd::schema::name::NameAttr;

use crate::attributes::{
    ParseError,
    apply_simple,
};

pub fn apply(
    commands: &mut Commands,
    prim: Entity,
    payload: Option<&[u8]>,
) -> Result<(), ParseError> {
    apply_simple::<NameAttr, Name>(commands, prim, payload, |attr| Some(Name::new(attr.0)))
}
