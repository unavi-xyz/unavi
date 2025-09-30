use crate::types::{Param as WParam, ParamData};

mod commands;
mod local;
mod query;
mod resource;

pub use commands::*;
pub use local::*;
pub use query::*;
pub use resource::*;

pub trait Param {
    /// Registers the param with the host, if it needs to be registered.
    fn register_param() -> Option<WParam>;
    fn is_mutable() -> bool;
    fn parse_param(data: &mut std::slice::IterMut<ParamData>) -> Self;
}

pub trait ParamGroup {
    fn register_params() -> Vec<Option<WParam>>;
    fn mutability() -> Vec<bool>;
    fn parse_params(data: &mut Vec<ParamData>) -> Self;
}

impl ParamGroup for () {
    fn register_params() -> Vec<Option<WParam>> {
        Vec::new()
    }
    fn mutability() -> Vec<bool> {
        Vec::new()
    }
    fn parse_params(_: &mut Vec<ParamData>) -> Self {}
}

impl<A> ParamGroup for (A,)
where
    A: Param,
{
    fn register_params() -> Vec<Option<WParam>> {
        vec![A::register_param()]
    }
    fn mutability() -> Vec<bool> {
        vec![A::is_mutable()]
    }
    fn parse_params(data: &mut Vec<ParamData>) -> Self {
        let mut iter = data.iter_mut();
        (A::parse_param(&mut iter),)
    }
}

impl<A, B> ParamGroup for (A, B)
where
    A: Param,
    B: Param,
{
    fn register_params() -> Vec<Option<WParam>> {
        vec![A::register_param(), B::register_param()]
    }
    fn mutability() -> Vec<bool> {
        vec![A::is_mutable(), B::is_mutable()]
    }
    fn parse_params(data: &mut Vec<ParamData>) -> Self {
        let mut iter = data.iter_mut();
        (A::parse_param(&mut iter), B::parse_param(&mut iter))
    }
}

impl<A, B, C> ParamGroup for (A, B, C)
where
    A: Param,
    B: Param,
    C: Param,
{
    fn register_params() -> Vec<Option<WParam>> {
        vec![
            A::register_param(),
            B::register_param(),
            C::register_param(),
        ]
    }
    fn mutability() -> Vec<bool> {
        vec![A::is_mutable(), B::is_mutable(), C::is_mutable()]
    }
    fn parse_params(data: &mut Vec<ParamData>) -> Self {
        let mut iter = data.iter_mut();
        (
            A::parse_param(&mut iter),
            B::parse_param(&mut iter),
            C::parse_param(&mut iter),
        )
    }
}
