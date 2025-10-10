use crate::{
    AppState, SystemState,
    host_api::register_system,
    param::{ConcreteConstraint, ParamGroup, ParamMeta},
    types::{Param as WParam, Schedule, System as WSystem, SystemId},
};

use super::{BlindSystem, System, function_system::FunctionSystem};

pub struct SystemCache {
    pub system: Box<dyn BlindSystem>,
}

pub trait RegisterSystem {
    fn register_system(self, state: &mut AppState, schedule: Schedule) -> (SystemId, SystemCache);
}

impl<F, In> RegisterSystem for FunctionSystem<F, In>
where
    F: 'static,
    In: ParamGroup + 'static,
    FunctionSystem<F, In>: System<In = In>,
{
    fn register_system(self, state: &mut AppState, schedule: Schedule) -> (SystemId, SystemCache) {
        let metas = In::meta();

        let mut sys_state = SystemState::default();
        let params: Vec<WParam> = In::register_params(&mut sys_state)
            .into_iter()
            .flatten()
            .collect();

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

        state.system_state.insert(id, sys_state);

        (
            id,
            SystemCache {
                system: Box::new(self),
            },
        )
    }
}
