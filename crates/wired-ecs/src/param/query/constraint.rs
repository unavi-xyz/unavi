use std::marker::PhantomData;

use crate::{Component, param::ConcreteConstraint, types::Constraint as WConstraint};

pub trait Constraint {
    fn build_constraints() -> Vec<WConstraint>;
    fn concrete_constraints() -> Vec<ConcreteConstraint>;
}

pub struct With<T: Component> {
    _t: PhantomData<T>,
}
impl<T: Component> Constraint for With<T> {
    fn build_constraints() -> Vec<WConstraint> {
        let c = T::register();
        vec![WConstraint::WithComponent(c)]
    }
    fn concrete_constraints() -> Vec<ConcreteConstraint> {
        let component = T::register();
        vec![ConcreteConstraint::With { component }]
    }
}

pub struct Without<T: Component> {
    _t: PhantomData<T>,
}
impl<T: Component> Constraint for Without<T> {
    fn build_constraints() -> Vec<WConstraint> {
        let c = T::register();
        vec![WConstraint::WithoutComponent(c)]
    }
    fn concrete_constraints() -> Vec<ConcreteConstraint> {
        let component = T::register();
        vec![ConcreteConstraint::Without { component }]
    }
}

impl Constraint for () {
    fn build_constraints() -> Vec<WConstraint> {
        Vec::new()
    }
    fn concrete_constraints() -> Vec<ConcreteConstraint> {
        Vec::new()
    }
}
impl<A> Constraint for (A,)
where
    A: Constraint,
{
    fn build_constraints() -> Vec<WConstraint> {
        A::build_constraints()
    }
    fn concrete_constraints() -> Vec<ConcreteConstraint> {
        A::concrete_constraints()
    }
}
impl<A, B> Constraint for (A, B)
where
    A: Constraint,
    B: Constraint,
{
    fn build_constraints() -> Vec<WConstraint> {
        let mut out = A::build_constraints();
        out.append(&mut B::build_constraints());
        out
    }
    fn concrete_constraints() -> Vec<ConcreteConstraint> {
        let mut out = A::concrete_constraints();
        out.append(&mut B::concrete_constraints());
        out
    }
}
