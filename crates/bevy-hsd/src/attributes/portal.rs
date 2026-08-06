use bevy::prelude::*;
use hsd::attributes::{
    Attribute,
    portal::PortalAttr,
};

use crate::attributes::{
    AttributeParser,
    ParseError,
};

#[derive(Component, Debug, Clone, Copy)]
pub struct PortalData(pub PortalAttr);

/// Same payload as [`PortalData`], published a frame later by [`apply_portal`].
///
/// The stable component external crates (`unavi-space`, `unavi-script`)
/// depend on, decoupled from the raw parser-lifecycle type.
#[derive(Component, Debug, Clone, Copy)]
pub struct PortalConfig(pub PortalAttr);

pub struct PortalParser;

impl AttributeParser for PortalParser {
    fn key(&self) -> &'static str {
        PortalAttr::KEY
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
                    .insert(PortalData(PortalAttr::decode(payload)?));
            }
            None => {
                commands.entity(prim).remove::<(PortalData, PortalConfig)>();
            }
        }
        Ok(())
    }
}

pub fn apply_portal(
    changed: Query<(Entity, &PortalData), Changed<PortalData>>,
    mut commands: Commands,
) {
    for (entity, data) in &changed {
        commands.entity(entity).insert(PortalConfig(data.0));
    }
}
