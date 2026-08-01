use avian3d::prelude::GravityScale;
use bevy::prelude::*;
use hsd::{
    HSD_CONTAINER_ID,
    attributes::{
        Attribute,
        gravity_scale::GravityScaleAttr,
        hydrate_attr,
    },
};
use loro::{
    ContainerID,
    Index,
    TreeID,
    ValueOrContainer,
    event::Diff,
};

use crate::{
    attributes::{
        ApplyEvent,
        AttrDataEvent,
        AttributeParser,
        DocContext,
        ParseError,
    },
    diff::HsdDiffEvent,
};

#[derive(Debug)]
pub enum GravityScaleEvent {
    Set(GravityScaleAttr),
}

pub struct GravityScaleParser;

impl AttributeParser for GravityScaleParser {
    fn key(&self) -> &'static str {
        GravityScaleAttr::KEY
    }

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        value: Option<ValueOrContainer>,
    ) -> Result<(), ParseError> {
        if value.is_none() {
            commands.entity(prim).remove::<GravityScale>();
        }
        Ok(())
    }

    fn parse(
        &self,
        ctx: &DocContext,
        prim: TreeID,
        _path: &[(ContainerID, Index)],
        _diff: Diff,
    ) -> Result<(), ParseError> {
        let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
        let meta = tree.get_meta(prim)?;
        let attr: GravityScaleAttr = hydrate_attr(&meta)?;

        ctx.tx
            .send(HsdDiffEvent::AttrData {
                prim,
                data: AttrDataEvent::GravityScale(GravityScaleEvent::Set(attr)),
            })
            .map_err(|_| ParseError::SendDiff)?;
        Ok(())
    }
}

pub fn apply_gravity_scale(trigger: On<ApplyEvent<GravityScaleEvent>>, mut commands: Commands) {
    let GravityScaleEvent::Set(attr) = &trigger.value;
    commands
        .entity(trigger.entity)
        .insert(GravityScale(attr.scale as f32));
}
