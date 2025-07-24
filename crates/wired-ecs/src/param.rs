use crate::{
    Component, Query,
    types::{Param, ParamData},
};

pub trait Params {
    fn from_param_data(data: Vec<ParamData>) -> Self;
    fn register_params() -> Vec<Param>;
}

impl Params for () {
    fn from_param_data(_: Vec<ParamData>) -> Self {}
    fn register_params() -> Vec<Param> {
        Vec::new()
    }
}

impl<A> Params for (Query<A>,)
where
    A: Component,
{
    fn from_param_data(data: Vec<ParamData>) -> Self {
        (Query::<A>::from_param_data(
            data.into_iter().next().unwrap(),
        ),)
    }
    fn register_params() -> Vec<Param> {
        vec![Param::Query(crate::types::Query {
            components: vec![A::register()],
            constraints: Vec::new(),
        })]
    }
}

impl<A, B> Params for (Query<A>, Query<B>)
where
    A: Component,
    B: Component,
{
    fn from_param_data(data: Vec<ParamData>) -> Self {
        let mut iter = data.into_iter();

        (
            Query::<A>::from_param_data(iter.next().unwrap()),
            Query::<B>::from_param_data(iter.next().unwrap()),
        )
    }
    fn register_params() -> Vec<Param> {
        Vec::new()
    }
}

impl<A, B, C> Params for (Query<A>, Query<B>, Query<C>)
where
    A: Component,
    B: Component,
    C: Component,
{
    fn from_param_data(data: Vec<ParamData>) -> Self {
        let mut iter = data.into_iter();

        (
            Query::<A>::from_param_data(iter.next().unwrap()),
            Query::<B>::from_param_data(iter.next().unwrap()),
            Query::<C>::from_param_data(iter.next().unwrap()),
        )
    }
    fn register_params() -> Vec<Param> {
        Vec::new()
    }
}
