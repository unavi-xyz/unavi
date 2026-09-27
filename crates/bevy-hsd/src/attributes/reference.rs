use bevy::prelude::*;
use hsd::{
    id::DocId,
    property::Payload,
    schema::reference::ReferenceAttr,
};

use crate::attributes::ParseError;

/// The document this prim stands for.
///
/// The child document exists because the prim carries this attribute, and
/// goes when the prim or the attribute does.
#[derive(Component, Debug, Clone, Copy)]
pub struct HsdRef(pub DocId);

pub fn apply(
    commands: &mut Commands,
    prim: Entity,
    payload: Option<&[u8]>,
) -> Result<(), ParseError> {
    match payload {
        Some(payload) => {
            commands
                .entity(prim)
                .insert(HsdRef(ReferenceAttr::decode(payload)?.0));
        }
        None => {
            commands.entity(prim).remove::<HsdRef>();
        }
    }
    Ok(())
}
