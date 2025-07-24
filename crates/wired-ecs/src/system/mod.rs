use crate::{param::ParamGroup, types::ParamData};

pub mod function_system;

pub trait System: Send + Sync + 'static {
    type In;

    fn run(&self, data: Self::In);
}

pub trait BlindSystem {
    fn run_blind(&self, data: Vec<ParamData>);
}

impl<S, In> BlindSystem for S
where
    S: System<In = In>,
    In: ParamGroup + 'static,
{
    fn run_blind(&self, data: Vec<ParamData>) {
        let a = In::parse_params(data);
        self.run(a)
    }
}

pub trait IntoSystem<T> {
    fn into_system(this: Self) -> T;
}
