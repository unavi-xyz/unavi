use crate::{param::ParamGroup, types::ParamData};

pub mod function_system;

pub trait System {
    type In;
    fn run(&self, data: Self::In);
}

pub trait BlindSystem {
    fn run_blind(&self, data: &mut Vec<ParamData>);
}

impl<S, P> BlindSystem for S
where
    P: ParamGroup,
    S: System<In = P>,
{
    fn run_blind(&self, data: &mut Vec<ParamData>) {
        let a = P::parse_params(data);
        self.run(a);
    }
}

pub trait IntoSystem<T>: Sized {
    fn into_system(self) -> T;
}
