use bevy::prelude::{Name, *};
use hsd::{
    HSD_CONTAINER_ID,
    attributes::{Attribute, hydrate_attr, name::NameAttr},
};
use loro::{ContainerID, Index, TreeID, ValueOrContainer, event::Diff};

use crate::{
    attributes::{
        ApplyEvent, AttrDataEvent, AttributeParser, DocContext, ParseError,
        util::shallow_map_updated_keys,
    },
    diff::HsdDiffEvent,
};

#[derive(Debug)]
pub enum NameEvent {
    Name(String),
}

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
        if value.is_some() {
            commands.entity(prim).insert(Name::default());
        } else {
            commands.entity(prim).remove::<Name>();
        }
        Ok(())
    }

    fn parse(
        &self,
        ctx: &DocContext,
        prim: TreeID,
        path: &[(ContainerID, Index)],
        diff: Diff,
    ) -> Result<(), ParseError> {
        let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
        let meta = tree.get_meta(prim)?;

        let attr: NameAttr = hydrate_attr(&meta)?;

        let keys = shallow_map_updated_keys(path, diff)?;
        for key in keys {
            if key == "name" {
                ctx.tx.send(HsdDiffEvent::AttrData {
                    prim,
                    data: AttrDataEvent::Name(NameEvent::Name(attr.name)),
                })?;
                break;
            }
        }

        Ok(())
    }
}

pub fn apply_name(trigger: On<ApplyEvent<NameEvent>>, mut names: Query<&mut Name>) {
    let Ok(mut name) = names.get_mut(trigger.entity) else {
        warn!("Name not found");
        return;
    };

    match &trigger.value {
        NameEvent::Name(v) => name.set(v.clone()),
    }
}
