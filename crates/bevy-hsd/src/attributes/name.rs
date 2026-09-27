use bevy::prelude::*;
use hsd::{
    property::{
        Payload,
        name::PropName,
    },
    schema::name::{
        self,
        NameAttr,
    },
};

use crate::attributes::{
    AttributeParser,
    ParseError,
};

#[derive(Component, Debug, Clone)]
pub struct NameData(pub NameAttr);

pub struct NameParser;

impl AttributeParser for NameParser {
    fn group(&self) -> &'static str {
        name::GROUP
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
                    .insert(NameData(NameAttr::decode(payload)?));
            }
            None => {
                commands.entity(prim).remove::<(NameData, Name)>();
            }
        }
        Ok(())
    }
}

pub fn apply_name(changed: Query<(Entity, &NameData), Changed<NameData>>, mut commands: Commands) {
    for (entity, data) in &changed {
        commands.entity(entity).insert(Name::new(data.0.0.clone()));
    }
}
