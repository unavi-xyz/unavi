use bevy::prelude::*;
use hsd::{
    id::DocId,
    property::{
        Payload,
        name::PropName,
    },
    schema::reference::{
        self,
        ReferenceAttr,
    },
};

use crate::attributes::{
    AttributeParser,
    ParseError,
};

/// The document this prim stands for.
///
/// Realizing is declarative, as instancing was: the child document exists
/// because the prim carries the attribute, and goes when the prim or the
/// attribute does.
#[derive(Component, Debug, Clone, Copy)]
pub struct HsdRef(pub DocId);

pub struct ReferenceParser;

impl AttributeParser for ReferenceParser {
    fn group(&self) -> &'static str {
        reference::GROUP
    }

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        _name: &PropName,
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
}
