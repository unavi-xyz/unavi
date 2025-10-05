use crate::{SystemState, param::ParamGroup, types::ParamData};

pub mod function_system;
pub mod register_system;

pub trait System {
    type In;
    fn run(&self, data: Self::In);
}

pub trait BlindSystem {
    fn run_blind(&self, state: &mut SystemState, data: &mut Vec<ParamData>);
}

impl<S, P> BlindSystem for S
where
    P: ParamGroup,
    S: System<In = P>,
{
    fn run_blind(&self, state: &mut SystemState, data: &mut Vec<ParamData>) {
        let a = P::parse_params(state, data);
        self.run(a);
    }
}
