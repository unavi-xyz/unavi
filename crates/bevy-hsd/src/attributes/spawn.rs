use bevy::prelude::*;
use hsd::{
    HSD_CONTAINER_ID,
    attributes::{
        Attribute,
        hydrate_attr,
        spawn::SpawnAttr,
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

#[derive(Component, Debug, Clone, Copy)]
pub struct SpawnPoint {
    pub radius: f32,
}

#[derive(Debug)]
pub enum SpawnEvent {
    Set(SpawnAttr),
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
        value: Option<ValueOrContainer>,
    ) -> Result<(), ParseError> {
        if value.is_none() {
            commands.entity(prim).remove::<SpawnPoint>();
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
        let attr: SpawnAttr = hydrate_attr(&meta)?;

        ctx.tx
            .send(HsdDiffEvent::AttrData {
                prim,
                data: AttrDataEvent::Spawn(SpawnEvent::Set(attr)),
            })
            .map_err(|_| ParseError::SendDiff)?;
        Ok(())
    }
}

pub fn apply_spawn(trigger: On<ApplyEvent<SpawnEvent>>, mut commands: Commands) {
    let SpawnEvent::Set(attr) = &trigger.value;
    commands.entity(trigger.entity).insert(SpawnPoint {
        radius: attr.radius.max(0.0) as f32,
    });
}
