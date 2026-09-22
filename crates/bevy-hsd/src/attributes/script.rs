use bevy::prelude::*;
use hsd::attributes::{
    Attribute,
    script::ScriptAttr,
};

use crate::attributes::{
    AttributeParser,
    ParseError,
};

#[derive(Component, Debug, Clone)]
pub struct HsdScript(pub Vec<u8>);

pub struct ScriptParser;

impl AttributeParser for ScriptParser {
    fn key(&self) -> &'static str {
        ScriptAttr::KEY
    }

    fn lifecycle(
        &self,
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
}
