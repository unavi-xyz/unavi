use super::component_ref::{AsComponentMut, AsComponentRef};

pub trait AsTupleRef<T> {
    type TRef<'a>
    where
        Self: 'a;
    fn as_tuple_ref<'a>(&'a self) -> Self::TRef<'a>;
}
pub trait AsTupleMut<T> {
    type TMut<'a>
    where
        Self: 'a;
    fn as_tuple_mut<'a>(&'a mut self) -> Self::TMut<'a>;
}

impl<A, ARef> AsTupleRef<ARef> for A
where
    A: AsComponentRef<ARef>,
{
    type TRef<'a>
        = A::CRef<'a>
    where
        Self: 'a;

    fn as_tuple_ref<'a>(&'a self) -> Self::TRef<'a> {
        self.as_component_ref()
    }
}
impl<A, AMut> AsTupleMut<AMut> for A
where
    A: AsComponentMut<AMut>,
{
    type TMut<'a>
        = A::CMut<'a>
    where
        Self: 'a;

    fn as_tuple_mut<'a>(&'a mut self) -> Self::TMut<'a> {
        self.as_component_mut()
    }
}

impl<A, ARef> AsTupleRef<(ARef,)> for (A,)
where
    A: AsComponentRef<ARef>,
{
    type TRef<'a>
        = (A::CRef<'a>,)
    where
        Self: 'a;

    fn as_tuple_ref<'a>(&'a self) -> Self::TRef<'a> {
        (self.0.as_component_ref(),)
    }
}
impl<A, AMut> AsTupleMut<(AMut,)> for (A,)
where
    A: AsComponentMut<AMut>,
{
    type TMut<'a>
        = (A::CMut<'a>,)
    where
        Self: 'a;

    fn as_tuple_mut<'a>(&'a mut self) -> Self::TMut<'a> {
        (self.0.as_component_mut(),)
    }
}
