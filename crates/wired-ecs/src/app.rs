use std::collections::BTreeMap;

use crate::{
    host_api::write_component,
    param::{ParamGroup, ParamMeta},
    system::{
        System,
        function_system::FunctionSystem,
        register_system::{RegisterSystem, SystemCache},
    },
    types::{Param as WParam, ParamData, Schedule, SystemId},
};

#[derive(Default)]
pub struct App {
    systems: BTreeMap<SystemId, SystemCache>,
}

impl App {
    pub fn add_system<F, In>(&mut self, schedule: Schedule, f: F) -> &mut Self
    where
        F: 'static,
        In: ParamGroup + 'static,
        FunctionSystem<F, In>: System<In = In>,
    {
        let f = FunctionSystem::new(f);
        for (id, sys) in f.register_system(schedule) {
            self.systems.insert(id, sys);
        }
        self
    }

    pub fn exec_system(&self, id: u32, mut data: Vec<ParamData>) {
        let cache = &self.systems[&id];

        if cache.immutable {
            cache.system.run_blind(&mut data);
        } else {
            let mut before = Vec::new();

            for (i, p) in data
                .iter()
                .enumerate()
                .zip(cache.mutability.iter())
                .filter_map(|(x, is_mut)| if *is_mut { Some(x) } else { None })
            {
                match p {
                    ParamData::Query(q) => {
                        before.push((i, q.data.clone()));
                    }
                }
            }

            // println!("exec {id}: {data:?}");
            cache.system.run_blind(&mut data);

            for (i, bef) in before {
                match &data[i] {
                    ParamData::Query(aft) => {
                        if bef == aft.data {
                            continue;
                        }

                        // Data was modified, write to host.
                        let WParam::Query(q) = &cache.params[i];
                        let Some(ParamMeta::Query {
                            component_mut,
                            component_sizes,
                            ..
                        }) = &cache.metas[i]
                        else {
                            panic!("Param meta not found")
                        };

                        for (entity, d) in aft.ents.iter().zip(aft.data.iter()) {
                            let mut i = 0;

                            for ((component, size), is_mut) in q
                                .components
                                .iter()
                                .zip(component_sizes.iter())
                                .zip(component_mut.iter())
                            {
                                let i_end = i + size;
                                let d = &d[i..i_end];
                                i = i_end;

                                if !is_mut {
                                    continue;
                                }

                                if let Err(e) = write_component(*entity, *component, d) {
                                    panic!("Failed to write component data: {e}")
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
