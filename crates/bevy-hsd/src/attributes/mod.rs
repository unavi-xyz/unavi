use std::sync::{Arc, LazyLock};

use bevy::{platform::collections::HashMap, prelude::*};
use loro::{ContainerID, Index, LoroDoc, TreeID, ValueOrContainer, event::Diff};

pub mod name;
pub mod xform;

pub static PARSERS: LazyLock<HashMap<&'static str, Box<dyn AttributeParser>>> =
    LazyLock::new(|| {
        let parsers: [Box<dyn AttributeParser>; _] =
            [Box::new(name::NameParser), Box::new(xform::XformParser)];
        let mut map = HashMap::default();
        for attr in parsers {
            map.insert(attr.key(), attr);
        }
        map
    });

pub enum AttrDataEvent {
    Name(name::NameEvent),
    Xform(xform::XformEvent),
}

pub trait AttributeParser: Send + Sync {
    fn key(&self) -> &'static str;

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        value: Option<ValueOrContainer>,
    ) -> anyhow::Result<()>;

    fn parse(
        &self,
        doc: &Arc<LoroDoc>,
        prim: TreeID,
        path: &[(ContainerID, Index)],
        diff: Diff,
    ) -> anyhow::Result<Option<AttrDataEvent>>;
}

#[derive(EntityEvent)]
pub struct ApplyEvent<T> {
    pub entity: Entity,
    pub value: T,
}
