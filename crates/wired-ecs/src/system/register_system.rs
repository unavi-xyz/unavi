use crate::{
    AppState, SystemState,
    host_api::register_system,
    param::{ConcreteConstraint, ParamGroup, ParamMeta},
    types::{Param as WParam, Query as WQuery, Schedule, System as WSystem, SystemId},
};

use super::{BlindSystem, System, function_system::FunctionSystem};

pub struct SystemCache {
    pub system: Box<dyn BlindSystem>,
}

pub trait RegisterSystem {
    fn register_system(self, state: &mut AppState, schedule: Schedule) -> (SystemId, SystemCache);
}

/// Validates a single query's constraints for conflicts.
fn validate_query_constraints(query: &WQuery, constraints: &[ConcreteConstraint]) {
    // Check for duplicate constraints.
    let mut a = 0;
    while a < constraints.len() {
        let mut b = 0;
        while b < constraints.len() {
            if a != b
                && (matches!((& constraints[a], &constraints[b]),
                    (ConcreteConstraint::With { component: c1 }, ConcreteConstraint::With { component: c2 }) if c1 == c2)
                    || matches!((&constraints[a], &constraints[b]),
                        (ConcreteConstraint::Without { component: c1 }, ConcreteConstraint::Without { component: c2 }) if c1 == c2))
            {
                println!("Query has duplicate constraint:\n- {query:?}");
            }
            b += 1;
        }
        a += 1;
    }

    // Check constraint validity.
    let mut i = 0;
    while i < constraints.len() {
        match &constraints[i] {
            ConcreteConstraint::With { component } => {
                if query.components.contains(component) {
                    println!("Query has redundent constraint:\n- {query:?}");
                }
            }
            ConcreteConstraint::Without { component } => {
                assert!(
                    !query.components.contains(component),
                    "Query has conflicting constraint:\n- {query:?}"
                );

                // Check for conflicting With/Without.
                let mut j = 0;
                while j < constraints.len() {
                    if let ConcreteConstraint::With { component: c2 } = &constraints[j] {
                        assert!(
                            c2 != component,
                            "Query has conflicting constraint:\n- {query:?}"
                        );
                    }
                    j += 1;
                }
            }
        }
        i += 1;
    }
}

/// Builds with/without component lists from constraints.
fn build_constraint_lists(
    components: &[u32],
    constraints: &[ConcreteConstraint],
) -> (Vec<u32>, Vec<u32>) {
    let mut with = components.to_vec();
    let mut without = Vec::new();

    for con in constraints {
        match con {
            ConcreteConstraint::With { component } => {
                with.push(*component);
            }
            ConcreteConstraint::Without { component } => {
                without.push(*component);
            }
        }
    }

    (with, without)
}

/// Checks if two queries are mutually exclusive.
fn are_queries_exclusive(
    with_1: &[u32],
    without_1: &[u32],
    with_2: &[u32],
    without_2: &[u32],
) -> bool {
    // Check if any component in with_1 is in without_2.
    let mut i = 0;
    while i < with_1.len() {
        let mut j = 0;
        while j < without_2.len() {
            if with_1[i] == without_2[j] {
                return true;
            }
            j += 1;
        }
        i += 1;
    }

    // Check if any component in with_2 is in without_1.
    let mut i = 0;
    while i < with_2.len() {
        let mut j = 0;
        while j < without_1.len() {
            if with_2[i] == without_1[j] {
                return true;
            }
            j += 1;
        }
        i += 1;
    }

    false
}

/// Validates mutual exclusivity between two queries when one has mutable access.
fn validate_query_exclusivity(
    i: usize,
    q1: &WQuery,
    c_mut_1: &[bool],
    cons_1: &[ConcreteConstraint],
    params: &[WParam],
    metas: &[Option<ParamMeta>],
) {
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
            if !is_mut || !q2.components.contains(c1) {
                continue;
            }

            // Both queries contain c1 and q1 has mutable access.
            // Ensure exclusivity.
            let (with_1, without_1) = build_constraint_lists(&q1.components, cons_1);
            let (with_2, without_2) = build_constraint_lists(&q2.components, cons_2);

            let is_exclusive = are_queries_exclusive(&with_1, &without_1, &with_2, &without_2);

            assert!(
                is_exclusive,
                "Query has mutable reference to component {c1}, but not exclusive access:\n- {q1:?}\n- {q2:?}"
            );
        }
    }
}

impl<F, In> RegisterSystem for FunctionSystem<F, In>
where
    F: 'static,
    In: ParamGroup + 'static,
    Self: System<In = In>,
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
            let WParam::Query(q1) = p1;
            let Some(ParamMeta::Query {
                component_mut: c_mut_1,
                constraints: cons_1,
                ..
            }) = m1
            else {
                panic!("Invalid param meta")
            };

            // Validate this query's constraints.
            validate_query_constraints(q1, cons_1);

            // Validate exclusivity with other queries.
            validate_query_exclusivity(i, q1, c_mut_1, cons_1, &params, &metas);
        }

        let id = match register_system(&WSystem { schedule, params }) {
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
