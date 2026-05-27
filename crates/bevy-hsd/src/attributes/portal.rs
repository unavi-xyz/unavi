use bevy::prelude::*;
use hsd::{
    HSD_CONTAINER_ID,
    attributes::{
        Attribute,
        hydrate_attr,
        portal::PortalAttr,
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

#[derive(Component, Debug, Clone)]
pub struct Portal(pub PortalAttr);

#[derive(Debug)]
pub enum PortalEvent {
    Set(PortalAttr),
}

pub struct PortalParser;

impl AttributeParser for PortalParser {
    fn key(&self) -> &'static str {
        PortalAttr::KEY
    }

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        value: Option<ValueOrContainer>,
    ) -> Result<(), ParseError> {
        if value.is_none() {
            commands.entity(prim).remove::<Portal>();
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
        let attr: PortalAttr = hydrate_attr(&meta)?;

        ctx.tx
            .send(HsdDiffEvent::AttrData {
                prim,
                data: AttrDataEvent::Portal(PortalEvent::Set(attr)),
            })
            .map_err(|_| ParseError::SendDiff)?;
        Ok(())
    }
}

pub fn apply_portal(trigger: On<ApplyEvent<PortalEvent>>, mut commands: Commands) {
    let PortalEvent::Set(attr) = &trigger.value;
    commands.entity(trigger.entity).insert(Portal(attr.clone()));
}
