use avian3d::prelude::GravityScale;
use bevy::prelude::*;
use hsd::{
    property::{
        Payload,
        name::PropName,
    },
    schema::gravity_scale::{
        self,
        GravityScaleAttr,
    },
};

use crate::attributes::{
    AttributeParser,
    ParseError,
};

#[derive(Component, Debug, Clone, Copy)]
pub struct GravityScaleData(pub GravityScaleAttr);

pub struct GravityScaleParser;

impl AttributeParser for GravityScaleParser {
    fn group(&self) -> &'static str {
        gravity_scale::GROUP
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
