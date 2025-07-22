use crate::{param::Param, types::ParamType};

pub trait SystemFn: Send + Sync + 'static {
    fn param_types(&self) -> Vec<ParamType>;
    fn call(&self, raw_args: &[Vec<u8>]);
}

impl<F> SystemFn for F
where
    F: Fn() + Send + Sync + 'static,
{
    fn param_types(&self) -> Vec<ParamType> {
        Vec::new()
    }

    fn call(&self, _raw_args: &[Vec<u8>]) {
        self()
    }
}

// impl<A, F> SystemFn for F
// where
//     A: Param,
//     F: Fn(A) + Send + Sync + 'static,
// {
//     fn param_types(&self) -> Vec<ParamType> {
//         vec![A::param_type()]
//     }
//
//     fn call(&self, raw_args: &[Vec<u8>]) {
//         let a = A::from_bytes(&raw_args[0]);
//         (self)(a);
//     }
// }
