use crate::{
    Component, host_api,
    types::{EntityId, Param as WParam, ParamData},
};

use super::{Param, ParamMeta};

pub struct Commands;

impl Param for Commands {
    fn register_param() -> Option<WParam> {
        None
    }
    fn mutability() -> bool {
        false
    }
    fn meta() -> Option<ParamMeta> {
        None
    }
    fn parse_param(_: &mut std::slice::IterMut<ParamData>) -> Self {
        Commands
    }
}

impl Commands {
    pub fn spawn(&self) -> Entity {
        Entity(host_api::spawn().expect("spawn"))
    }
}

pub struct Entity(EntityId);

impl Entity {
    pub fn insert<T: Component>(&self, data: T) {
        let c_id = T::register();
        host_api::insert_component(self.0, c_id, &data.to_bytes()).expect("insert component");
    }
    pub fn remove<T: Component>(&self) {
        let c_id = T::register();
        host_api::remove_component(self.0, c_id).expect("remove component");
    }
}
