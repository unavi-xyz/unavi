use bevy::prelude::*;
use hsd::attributes::{
    Attribute,
    spawn::SpawnAttr,
};

use crate::attributes::{
    AttributeParser,
    ParseError,
};

#[derive(Component, Debug, Clone, Copy)]
pub struct SpawnData(pub SpawnAttr);

#[derive(Component, Debug, Clone, Copy)]
pub struct SpawnPoint {
    pub radius: f32,
}

pub struct SpawnParser;

impl AttributeParser for SpawnParser {
    fn key(&self) -> &'static str {
        SpawnAttr::KEY
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
                    .insert(SpawnData(SpawnAttr::decode(payload)?));
            }
            None => {
                commands.entity(prim).remove::<(SpawnData, SpawnPoint)>();
            }
        }
        Ok(())
    }
}

pub fn apply_spawn(
    changed: Query<(Entity, &SpawnData), Changed<SpawnData>>,
    mut commands: Commands,
) {
    for (entity, data) in &changed {
        commands.entity(entity).insert(SpawnPoint {
            radius: data.0.radius.max(0.0) as f32,
        });
    }
}
