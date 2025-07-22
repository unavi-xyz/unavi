use std::collections::{BTreeMap, HashMap};

use anyhow::Context;
use wasmtime::{
    AsContextMut,
    component::{HasData, ResourceAny},
};
use wired::ecs::types::{ParamId, ParamType, Schedule, System, SystemId};

use crate::{execute::RuntimeCtx, load::bindings::Guest};

wasmtime::component::bindgen!({
    world: "host",
    path: "../../protocol/wit/wired-ecs",
    additional_derives: [Hash],
});

pub struct HasWiredEcsData;

impl HasData for HasWiredEcsData {
    type Data<'a> = &'a mut WiredEcsData;
}

#[derive(Default)]
pub struct WiredEcsData {
    pub initialized: bool,
    pub params: Vec<ParamType>,
    pub systems: Vec<System>,
    pub schedules: HashMap<Schedule, Vec<SystemId>>,
}

impl wired::ecs::host_api::Host for WiredEcsData {
    fn register_param(&mut self, param_type: ParamType) -> wired::ecs::host_api::ParamId {
        let id = self.systems.len() as u64;
        self.params.push(param_type);
        id
    }
    fn register_system(&mut self, system: System) -> wired::ecs::host_api::SystemId {
        let id = self.systems.len() as u64;
        self.schedules.entry(system.schedule).or_default().push(id);
        self.systems.push(system);
        id
    }
    fn write_query(
        &mut self,
        system: wired::ecs::host_api::SystemId,
        query_index: u32,
        data: wasmtime::component::__internal::Vec<wired::ecs::host_api::ParamData>,
    ) {
        todo!()
    }
}
