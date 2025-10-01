use std::collections::BTreeMap;

use crate::{
    host_api::{register_system, write_component},
    param::{ParamGroup, ParamMeta},
    system::{BlindSystem, IntoSystem, System, function_system::FunctionSystem},
    types::{Param as WParam, ParamData, Query as WQuery, Schedule, System as WSystem, SystemId},
};

struct SystemCache {
    immutable: bool,
    mutability: Vec<bool>,
    metas: Vec<Option<ParamMeta>>,
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
        let metas = In::meta();
        let params: Vec<WParam> = In::register_params().into_iter().flatten().collect();

        // Validate parameters.
        let mut seen = Vec::<WQuery>::new();
        for (param, meta) in params.iter().zip(metas.iter()) {
            match param {
                // Multiple queries within a system cannot hold mutable references to the same
                // component.
                WParam::Query(q) => {
                    let Some(ParamMeta::Query { component_mut, .. }) = meta else {
                        panic!("Invalid param meta")
                    };

                    let components = q
                        .components
                        .iter()
                        .zip(component_mut.iter())
                        .filter_map(|(c, is_mut)| if *is_mut { Some(*c) } else { None })
                        .collect();

                    for c in &components {
                        if let Some(q1) = seen.iter().find(|x| x.components.contains(c)) {
                            panic!(
                                "Conflicting Query parameters (component {c}):\n- {q1:?}\n- {q:?}"
                            )
                        }
                    }

                    let mut q1 = q.clone();
                    q1.components = components;
                    seen.push(q1);
                }
            }
        }

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
                metas,
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
