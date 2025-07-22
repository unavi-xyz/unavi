use std::collections::BTreeMap;

mod param;
mod system;

pub use bytemuck;
pub use param::*;
pub use wired_ecs_derive;

mod bindings {
    wit_bindgen::generate!({
        world: "guest",
        path: "../../protocol/wit/wired-ecs",
    });
}

pub use bindings::wired::ecs::*;
use bindings::wired::ecs::{
    host_api::register_system,
    types::{ParamType, Schedule, System, SystemId},
};
use system::SystemFn;

#[derive(Default)]
pub struct App {
    systems: BTreeMap<SystemId, SystemImpl>,
}

struct SystemImpl {
    params: Vec<ParamType>,
    f: Box<dyn SystemFn>,
}

impl App {
    pub fn add_system(&mut self, schedule: Schedule, system: impl SystemFn) {
        let id = register_system(&System {
            schedule,
            queries: Vec::new(),
        });

        self.systems.insert(
            id,
            SystemImpl {
                params: Vec::new(),
                f: Box::new(system),
            },
        );
    }

    pub fn exec_system(&self, id: u64, params: Vec<Vec<u8>>) {
        let sys = &self.systems[&id];
        sys.f.call(&params);
    }
}
