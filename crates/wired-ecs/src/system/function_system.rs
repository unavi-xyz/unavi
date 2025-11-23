use std::marker::PhantomData;

use super::System;

pub struct FunctionSystem<F, In> {
    f: F,
    _in: PhantomData<In>,
}

impl<F, In> FunctionSystem<F, In> {
    pub const fn new(f: F) -> Self {
        Self {
            f,
            _in: PhantomData,
        }
    }
}

impl<F> System for FunctionSystem<F, ()>
where
    F: Fn() + 'static,
{
    type In = ();

    fn run(&self, (): Self::In) {
        (self.f)();
    }
}

impl<F, A> System for FunctionSystem<F, (A,)>
where
    F: Fn(A) + 'static,
{
    type In = (A,);

    fn run(&self, data: Self::In) {
        (self.f)(data.0);
    }
}

impl<F, A, B> System for FunctionSystem<F, (A, B)>
where
    F: Fn(A, B) + 'static,
{
    type In = (A, B);

    fn run(&self, data: Self::In) {
        (self.f)(data.0, data.1);
    }
}

impl<F, A, B, C> System for FunctionSystem<F, (A, B, C)>
where
    F: Fn(A, B, C) + 'static,
{
    type In = (A, B, C);

    fn run(&self, data: Self::In) {
        (self.f)(data.0, data.1, data.2);
    }
}
