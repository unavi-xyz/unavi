use std::collections::BTreeMap;

use crate::{
    host_api::register_system,
    param::ParamGroup,
    system::{BlindSystem, IntoSystem, System, function_system::FunctionSystem},
    types::{ParamData, Schedule, SystemId},
};

#[derive(Default)]
pub struct App {
    systems: BTreeMap<SystemId, Box<dyn BlindSystem>>,
}

impl App {
    pub fn add_system<F, In>(&mut self, schedule: Schedule, f: F)
    where
        F: IntoSystem<FunctionSystem<F, In>>,
        FunctionSystem<F, In>: System<In = In>,
        In: ParamGroup + Send + Sync + 'static,
    {
        let params = In::register_params();
        let id = match register_system(&crate::types::System { schedule, params }) {
            Ok(id) => id,
            Err(e) => panic!("Failed to register system: {e}"),
        };

        let system = F::into_system(f);
        self.systems.insert(id, Box::new(system));
    }

    pub fn exec_system(&self, id: u64, data: Vec<ParamData>) {
        let sys = &self.systems[&id];
        sys.run_blind(data);
    }
}
