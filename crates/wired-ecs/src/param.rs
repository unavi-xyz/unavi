use crate::types::{Param as BParam, ParamData};

pub trait Param {
    fn parse_param(data: ParamData) -> Self;
    fn register_param() -> BParam;
}

pub trait ParamGroup {
    fn parse_params(data: Vec<ParamData>) -> Self;
    fn register_params() -> Vec<BParam>;
}

impl ParamGroup for () {
    fn parse_params(_: Vec<ParamData>) -> Self {}
    fn register_params() -> Vec<BParam> {
        Vec::new()
    }
}

impl<A> ParamGroup for (A,)
where
    A: Param,
{
    fn parse_params(data: Vec<ParamData>) -> Self {
        (A::parse_param(data.into_iter().next().unwrap()),)
    }
    fn register_params() -> Vec<BParam> {
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

        (
            A::parse_param(iter.next().unwrap()),
            B::parse_param(iter.next().unwrap()),
        )
    }
    fn register_params() -> Vec<BParam> {
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
            A::parse_param(iter.next().unwrap()),
            B::parse_param(iter.next().unwrap()),
            C::parse_param(iter.next().unwrap()),
        )
    }
    fn register_params() -> Vec<BParam> {
        vec![
            A::register_param(),
            B::register_param(),
            C::register_param(),
        ]
    }
}
