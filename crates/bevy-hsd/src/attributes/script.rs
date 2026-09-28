use bevy::prelude::*;
use hsd::{
    attributes::script::ScriptAttr,
    property::Payload,
};

use crate::attributes::ParseError;

#[derive(Component, Debug, Clone)]
pub struct HsdScript(pub Vec<u8>);

pub(crate) fn apply(
    commands: &mut Commands,
    prim: Entity,
    payload: Option<&[u8]>,
) -> Result<(), ParseError> {
    match payload {
        Some(payload) => {
            commands
                .entity(prim)
                .insert(HsdScript(ScriptAttr::decode(payload)?.0));
        }
        None => {
            commands.entity(prim).remove::<HsdScript>();
        }
    }
    Ok(())
}
