use std::sync::{Arc, LazyLock};

use bevy::platform::collections::HashMap;
use loro::{ContainerID, Index, LoroDoc, TreeID, event::Diff};

pub mod xform;

pub static PARSERS: LazyLock<HashMap<&'static str, Box<dyn AttributeParser>>> =
    LazyLock::new(|| {
        let mut map = HashMap::<_, Box<dyn AttributeParser>>::default();
        for attr in [xform::XformParser] {
            map.insert(attr.key(), Box::new(attr));
        }
        map
    });

pub enum AttrDataEvent {
    Xform(xform::XformEvent),
}

pub trait AttributeParser: Send + Sync {
    fn key(&self) -> &'static str;

    fn parse(
        &self,
        doc: &Arc<LoroDoc>,
        prim: TreeID,
        path: &[(ContainerID, Index)],
        diff: Diff,
    ) -> anyhow::Result<Option<AttrDataEvent>>;
}
