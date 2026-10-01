use bevy::prelude::*;
use hsd::attributes::script::ScriptAttr;

use crate::attributes::apply_simple;

#[derive(Component, Debug, Clone)]
pub struct HsdScript(pub Vec<u8>);

pub(crate) fn apply(
    commands: &mut Commands,
    prim: Entity,
    payload: Option<&[u8]>,
) -> Result<(), postcard::Error> {
    apply_simple::<ScriptAttr, HsdScript>(commands, prim, payload, |attr| Some(HsdScript(attr.0)))
}
