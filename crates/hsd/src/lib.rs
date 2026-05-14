use std::{collections::HashMap, sync::LazyLock};

use loro::{ContainerID, ContainerType, LoroValue};
use lorosurgeon::{Hydrate, Reconcile};

pub mod attributes;

pub static HSD_CONTAINER_ID: LazyLock<ContainerID> = LazyLock::new(|| ContainerID::Root {
    name: "hsd".into(),
    container_type: ContainerType::Tree,
});

#[derive(Reconcile, Hydrate)]
pub struct Prim(pub HashMap<String, AttrValue>);

#[derive(Reconcile, Hydrate)]
pub enum AttrValue {
    Relationship(String),
    Value(LoroValue),
}

// #[derive(Reconcile, Hydrate)]
// #[loro(root = "hsd")]
// pub struct Hsd {
//     pub stage: lorosurgeon::MaybeMissing<Vec<String>>,
//     #[loro(movable)]
//     pub layers: Vec<Layer>,
// }
//
// #[derive(Reconcile, Hydrate)]
// pub struct Layer {
//     #[key]
//     id: String,
//     // value: Vec<>,
// }
