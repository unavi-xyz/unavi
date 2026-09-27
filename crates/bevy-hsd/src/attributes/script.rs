use bevy::prelude::*;
use hsd::{
    property::{
        Payload,
        name::PropName,
    },
    schema::script::{
        self,
        ScriptAttr,
    },
};

use crate::attributes::{
    AttributeParser,
    ParseError,
};

#[derive(Component, Debug, Clone)]
pub struct HsdScript(pub Vec<u8>);

pub struct ScriptParser;

impl AttributeParser for ScriptParser {
    fn group(&self) -> &'static str {
        script::GROUP
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
                    .insert(HsdScript(ScriptAttr::decode(payload)?.0));
            }
            None => {
                commands.entity(prim).remove::<HsdScript>();
            }
        }
        Ok(())
    }
}
