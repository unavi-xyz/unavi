use std::collections::BTreeMap;

use crate::{
    host_api::{register_system, write_component},
    param::ParamGroup,
    system::{BlindSystem, IntoSystem, System, function_system::FunctionSystem},
    types::{Param as WParam, ParamData, Schedule, System as WSystem, SystemId},
};

struct SystemCache {
    immutable: bool,
    mutability: Vec<bool>,
    params: Vec<WParam>,
    system: Box<dyn BlindSystem>,
}

#[derive(Default)]
pub struct App {
    systems: BTreeMap<SystemId, SystemCache>,
}

impl App {
    pub fn add_system<F, In>(&mut self, schedule: Schedule, f: F) -> &mut Self
    where
        In: ParamGroup + 'static,
        F: IntoSystem<FunctionSystem<F, In>> + 'static,
        FunctionSystem<F, In>: System<In = In>,
    {
        let system = F::into_system(f);

        let mutability = In::mutability();
        let params: Vec<WParam> = In::register_params().into_iter().flatten().collect();

        let id = match register_system(&WSystem {
            schedule,
            params: params.clone(),
        }) {
            Ok(id) => id,
            Err(e) => panic!("Failed to register system: {e}"),
        };

        self.systems.insert(
            id,
            SystemCache {
                immutable: !mutability.iter().any(|x| *x),
                mutability,
                params,
                system: Box::new(system),
            },
        );

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

            // println!("exec: {data:?}");
            cache.system.run_blind(&mut data);

            for (i, bef) in before {
                match &data[i] {
                    ParamData::Query(aft) => {
                        if bef == aft.data {
                            continue;
                        }

                        // Data was modified, write to host.
                        // println!("modified: {aft:?}");
                        let WParam::Query(q) = &cache.params[i];

                        for (entity, d) in aft.ents.iter().zip(aft.data.iter()) {
                            for component in &q.components {
                                // TODO: slice d to match component size
                                // println!("writing {entity} > {component}: {d:?}");
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
