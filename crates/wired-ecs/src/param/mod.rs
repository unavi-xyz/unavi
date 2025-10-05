use crate::{
    ParamState, SystemState,
    types::{Param as WParam, ParamData},
};

mod commands;
mod local;
mod query;
mod resource;

pub use commands::*;
pub use local::*;
pub use query::*;
pub use resource::*;

pub enum ParamMeta {
    Query {
        component_mut: Vec<bool>,
        component_sizes: Vec<usize>,
        constraints: Vec<ConcreteConstraint>,
    },
}

#[derive(PartialEq)]
pub enum ConcreteConstraint {
    With { component: u32 },
    Without { component: u32 },
}

pub trait Param {
    /// Registers the param with the host, if it needs to be registered.
    fn register_param(state: &mut Vec<ParamState>) -> Option<WParam>;
    fn mutability() -> bool;
    fn meta() -> Option<ParamMeta>;
    fn parse_param(
        state: &mut std::slice::IterMut<ParamState>,
        data: &mut std::slice::IterMut<ParamData>,
    ) -> Self;
}

pub trait ParamGroup {
    fn register_params(state: &mut SystemState) -> Vec<Option<WParam>>;
    fn mutability() -> Vec<bool>;
    fn meta() -> Vec<Option<ParamMeta>>;
    fn parse_params(state: &mut SystemState, data: &mut Vec<ParamData>) -> Self;
}

impl ParamGroup for () {
    fn register_params(_: &mut SystemState) -> Vec<Option<WParam>> {
        Vec::new()
    }
    fn mutability() -> Vec<bool> {
        Vec::new()
    }
    fn meta() -> Vec<Option<ParamMeta>> {
        Vec::new()
    }
    fn parse_params(_: &mut SystemState, _: &mut Vec<ParamData>) -> Self {}
}

impl<A> ParamGroup for (A,)
where
    A: Param,
{
    fn register_params(state: &mut SystemState) -> Vec<Option<WParam>> {
        vec![A::register_param(&mut state.param_state)]
    }
    fn mutability() -> Vec<bool> {
        vec![A::mutability()]
    }
    fn meta() -> Vec<Option<ParamMeta>> {
        vec![A::meta()]
    }
    fn parse_params(state: &mut SystemState, data: &mut Vec<ParamData>) -> Self {
        let mut s_iter = state.param_state.iter_mut();
        let mut d_iter = data.iter_mut();
        (A::parse_param(&mut s_iter, &mut d_iter),)
    }
}

impl<A, B> ParamGroup for (A, B)
where
    A: Param,
    B: Param,
{
    fn register_params(state: &mut SystemState) -> Vec<Option<WParam>> {
        vec![
            A::register_param(&mut state.param_state),
            B::register_param(&mut state.param_state),
        ]
    }
    fn mutability() -> Vec<bool> {
        vec![A::mutability(), B::mutability()]
    }
    fn meta() -> Vec<Option<ParamMeta>> {
        vec![A::meta(), B::meta()]
    }
    fn parse_params(state: &mut SystemState, data: &mut Vec<ParamData>) -> Self {
        let mut s_iter = state.param_state.iter_mut();
        let mut d_iter = data.iter_mut();
        (
            A::parse_param(&mut s_iter, &mut d_iter),
            B::parse_param(&mut s_iter, &mut d_iter),
        )
    }
}

impl<A, B, C> ParamGroup for (A, B, C)
where
    A: Param,
    B: Param,
    C: Param,
{
    fn register_params(state: &mut SystemState) -> Vec<Option<WParam>> {
        vec![
            A::register_param(&mut state.param_state),
            B::register_param(&mut state.param_state),
            C::register_param(&mut state.param_state),
        ]
    }
    fn mutability() -> Vec<bool> {
        vec![A::mutability(), B::mutability(), C::mutability()]
    }
    fn meta() -> Vec<Option<ParamMeta>> {
        vec![A::meta(), B::meta(), C::meta()]
    }
    fn parse_params(state: &mut SystemState, data: &mut Vec<ParamData>) -> Self {
        let mut s_iter = state.param_state.iter_mut();
        let mut d_iter = data.iter_mut();
        (
            A::parse_param(&mut s_iter, &mut d_iter),
            B::parse_param(&mut s_iter, &mut d_iter),
            C::parse_param(&mut s_iter, &mut d_iter),
        )
    }
}
