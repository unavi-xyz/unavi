use bevy::prelude::{
    Name,
    *,
};
use hsd::attributes::{
    Attribute,
    name::NameAttr,
};
use loro::{
    LoroValue,
    ValueOrContainer,
};

use crate::attributes::{
    AttributeParser,
    ParseError,
};

pub struct NameParser;

impl AttributeParser for NameParser {
    fn key(&self) -> &'static str {
        NameAttr::KEY
    }

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        value: Option<ValueOrContainer>,
    ) -> Result<(), ParseError> {
        match value {
            Some(ValueOrContainer::Value(LoroValue::String(s))) => {
                commands.entity(prim).insert(Name::new(s.to_string()));
            }
            Some(other) => {
                warn!(?other, "name attribute must be a string");
            }
            None => {
                commands.entity(prim).remove::<Name>();
            }
        }
        Ok(())
    }
}
