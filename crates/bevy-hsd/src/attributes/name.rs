use std::sync::Arc;

use anyhow::bail;
use bevy::prelude::{Name as BevyName, *};
use hsd::{
    HSD_CONTAINER_ID,
    attributes::{Attribute, name::Name},
};
use loro::{ContainerID, Index, LoroDoc, TreeID, ValueOrContainer, event::Diff};

use crate::attributes::{ApplyEvent, AttrDataEvent, AttributeParser};

pub enum NameEvent {
    Name(String),
}

pub struct NameParser;

impl AttributeParser for NameParser {
    fn key(&self) -> &'static str {
        Name::KEY
    }

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        value: Option<ValueOrContainer>,
    ) -> anyhow::Result<()> {
        if value.is_some() {
            commands.entity(prim).insert(BevyName::default());
        } else {
            commands.entity(prim).remove::<BevyName>();
        }
        Ok(())
    }

    fn parse(
        &self,
        doc: &Arc<LoroDoc>,
        prim: TreeID,
        path: &[(ContainerID, Index)],
        _diff: Diff,
    ) -> anyhow::Result<Option<AttrDataEvent>> {
        if path.is_empty() {
            return Ok(None);
        }

        let key = path[0]
            .1
            .as_key()
            .ok_or_else(|| anyhow::anyhow!("invalid index type"))?;

        let tree = doc.get_tree(&*HSD_CONTAINER_ID);
        let meta = tree.get_meta(prim)?;

        let name = Name::attr_hydrate(&meta)?;

        match key.as_str() {
            "name" => Ok(Some(AttrDataEvent::Name(NameEvent::Name(name.name)))),
            _ => bail!("unknown key"),
        }
    }
}

pub fn apply_name(trigger: On<ApplyEvent<NameEvent>>, mut names: Query<&mut BevyName>) {
    let Ok(mut name) = names.get_mut(trigger.entity) else {
        warn!("Name not found");
        return;
    };

    match &trigger.value {
        NameEvent::Name(v) => name.set(v.clone()),
    }
}
