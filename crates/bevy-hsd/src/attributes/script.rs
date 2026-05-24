use bevy::prelude::*;
use hsd::attributes::{Attribute, script::ScriptAttr};
use loro::{LoroValue, ValueOrContainer};

use crate::attributes::{AttributeParser, ParseError};

#[derive(Component, Debug, Clone)]
pub struct HsdScript(pub blake3::Hash);

pub struct ScriptParser;

impl AttributeParser for ScriptParser {
    fn key(&self) -> &'static str {
        ScriptAttr::KEY
    }

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        value: Option<ValueOrContainer>,
    ) -> Result<(), ParseError> {
        match value {
            Some(ValueOrContainer::Value(LoroValue::Binary(bytes))) => {
                if let Ok(arr) = <[u8; 32]>::try_from(bytes.as_slice()) {
                    commands
                        .entity(prim)
                        .insert(HsdScript(blake3::Hash::from_bytes(arr)));
                } else {
                    warn!("script attribute: expected 32-byte blob id");
                }
            }
            Some(other) => {
                warn!(?other, "script attribute must be a binary blob id");
            }
            None => {
                commands.entity(prim).remove::<HsdScript>();
            }
        }
        Ok(())
    }
}
