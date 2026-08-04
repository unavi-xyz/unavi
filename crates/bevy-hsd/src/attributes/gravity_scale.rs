use avian3d::prelude::GravityScale;
use bevy::prelude::*;
use hsd::attributes::{
    Attribute,
    gravity_scale::GravityScaleAttr,
};

use crate::attributes::{
    AttributeParser,
    ParseError,
};

#[derive(Component, Debug, Clone, Copy)]
pub struct GravityScaleData(pub GravityScaleAttr);

pub struct GravityScaleParser;

impl AttributeParser for GravityScaleParser {
    fn key(&self) -> &'static str {
        GravityScaleAttr::KEY
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
                    .insert(GravityScaleData(GravityScaleAttr::decode(payload)?));
            }
            None => {
                commands
                    .entity(prim)
                    .remove::<(GravityScaleData, GravityScale)>();
            }
        }
        Ok(())
    }
}

pub fn apply_gravity_scale(
    changed: Query<(Entity, &GravityScaleData), Changed<GravityScaleData>>,
    mut commands: Commands,
) {
    for (entity, data) in &changed {
        commands
            .entity(entity)
            .insert(GravityScale(data.0.scale as f32));
    }
}
