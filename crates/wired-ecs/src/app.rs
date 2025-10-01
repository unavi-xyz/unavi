use std::collections::BTreeMap;

use crate::{
    host_api::{register_system, write_component},
    param::{ConcreteConstraint, ParamGroup, ParamMeta},
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
        for (i, (p1, m1)) in params.iter().zip(metas.iter()).enumerate() {
            match p1 {
                WParam::Query(q1) => {
                    let Some(ParamMeta::Query {
                        component_mut: c_mut_1,
                        constraints: cons_1,
                        ..
                    }) = m1
                    else {
                        panic!("Invalid param meta")
                    };

                    // A query cannot have conflicting constraints.
                    for (a, con1) in cons_1.iter().enumerate() {
                        for (b, con2) in cons_1.iter().enumerate() {
                            if a == b {
                                continue;
                            }

                            if con1 == con2 {
                                println!("Query has duplicate constraint:\n- {q1:?}")
                            }
                        }

                        match con1 {
                            ConcreteConstraint::With { component } => {
                                if q1.components.contains(component) {
                                    println!("Query has redundent constraint:\n- {q1:?}");
                                }
                            }
                            ConcreteConstraint::Without { component } => {
                                if q1.components.contains(component) {
                                    panic!("Query has conflicting constraint:\n- {q1:?}")
                                }

                                for con2 in cons_1 {
                                    if let ConcreteConstraint::With { component: c2 } = con2
                                        && *c2 == *component
                                    {
                                        panic!("Query has conflicting constraint:\n- {q1:?}")
                                    }
                                }
                            }
                        };
                    }

                    // If a query contains a mutable reference to a component, it must hold exclusive
                    // access to that component for each entity. Any queries referencing that
                    // same component must be mutually exculsive in their constraints.
                    for (j, p2) in params.iter().enumerate() {
                        if i == j {
                            continue;
                        }

                        let WParam::Query(q2) = p2;

                        let Some(ParamMeta::Query {
                            constraints: cons_2,
                            ..
                        }) = &metas[j]
                        else {
                            panic!("Invalid param meta")
                        };

                        for (c1, is_mut) in q1.components.iter().zip(c_mut_1.iter()) {
                            if !is_mut {
                                continue;
                            }

                            if !q2.components.contains(c1) {
                                continue;
                            }

                            // Both queries contain c1.
                            // Ensure exclusivity.
                            let mut with_1 = q1.components.clone();
                            let mut without_1 = Vec::new();
                            for con1 in cons_1 {
                                match con1 {
                                    ConcreteConstraint::With { component } => {
                                        with_1.push(*component)
                                    }
                                    ConcreteConstraint::Without { component } => {
                                        without_1.push(*component)
                                    }
                                }
                            }

                            let mut with_2 = q2.components.clone();
                            let mut without_2 = Vec::new();
                            for con2 in cons_2 {
                                match con2 {
                                    ConcreteConstraint::With { component } => {
                                        with_2.push(*component)
                                    }
                                    ConcreteConstraint::Without { component } => {
                                        without_2.push(*component)
                                    }
                                }
                            }

                            let is_exclusive = with_1.iter().any(|w1| without_2.contains(w1))
                                || with_2.iter().any(|w2| without_1.contains(w2));

                            if !is_exclusive {
                                panic!(
                                    "Query has mutable reference to component {c1}, but not exclusive access:\n- {q1:?}\n- {q2:?}"
                                )
                            }
                        }
                    }
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
