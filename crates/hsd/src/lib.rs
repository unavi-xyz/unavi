use std::{
    collections::BTreeMap,
    sync::LazyLock,
};

use loro::{
    ContainerID,
    ContainerType,
};
use loro_surgeon::{
    Hydrate,
    Reconcile,
};
pub mod attributes;
pub mod file;

pub static HSD_CONTAINER_ID: LazyLock<ContainerID> = LazyLock::new(|| ContainerID::Root {
    name:           "hsd".into(),
    container_type: ContainerType::Tree,
});

#[derive(Reconcile, Hydrate, Default)]
#[loro(default)]
pub struct PrimMeta {
    pub attributes:    Option<attributes::Attributes>,
    pub relationships: Option<BTreeMap<String, String>>,
}
