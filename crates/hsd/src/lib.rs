use std::sync::LazyLock;

use loro::{ContainerID, ContainerType};

pub mod attributes;

pub static HSD_CONTAINER_ID: LazyLock<ContainerID> = LazyLock::new(|| ContainerID::Root {
    name: "hsd".into(),
    container_type: ContainerType::Tree,
});
