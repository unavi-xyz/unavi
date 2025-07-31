use crate::types::{Param as WParam, ParamData};

mod commands;
mod query;
mod resource;

pub use commands::*;
pub use query::*;
pub use resource::*;

pub trait Param {
    fn parse_param(data: &mut std::vec::IntoIter<ParamData>) -> Self;
    /// Registers the param with the host, if it needs to be registered.
    fn register_param() -> Option<WParam>;
}

pub trait ParamGroup {
    fn parse_params(data: Vec<ParamData>) -> Self;
    fn register_params() -> Vec<Option<WParam>>;
}

impl ParamGroup for () {
    fn parse_params(_: Vec<ParamData>) -> Self {}
    fn register_params() -> Vec<Option<WParam>> {
        Vec::new()
    }
}

impl<A> ParamGroup for (A,)
where
    A: Param,
{
    fn parse_params(data: Vec<ParamData>) -> Self {
        let mut iter = data.into_iter();

        (A::parse_param(&mut iter),)
    }
    fn register_params() -> Vec<Option<WParam>> {
        vec![A::register_param()]
    }
}

impl<A, B> ParamGroup for (A, B)
where
    A: Param,
    B: Param,
{
    fn parse_params(data: Vec<ParamData>) -> Self {
        let mut iter = data.into_iter();

        (A::parse_param(&mut iter), B::parse_param(&mut iter))
    }
    fn register_params() -> Vec<Option<WParam>> {
        vec![A::register_param(), B::register_param()]
    }
}

impl<A, B, C> ParamGroup for (A, B, C)
where
    A: Param,
    B: Param,
    C: Param,
{
    fn parse_params(data: Vec<ParamData>) -> Self {
        let mut iter = data.into_iter();

        (
            A::parse_param(&mut iter),
            B::parse_param(&mut iter),
            C::parse_param(&mut iter),
        )
    }
    fn register_params() -> Vec<Option<WParam>> {
        vec![
            A::register_param(),
            B::register_param(),
            C::register_param(),
        ]
    }
}
