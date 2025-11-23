use super::component_ref::{AsComponentMut, AsComponentRef};

pub trait AsTupleRef<T> {
    type TRef<'a>
    where
        Self: 'a;
    fn as_tuple_ref(&self) -> Self::TRef<'_>;
}
pub trait AsTupleMut<T> {
    type TMut<'a>
    where
        Self: 'a;
    fn as_tuple_mut(&mut self) -> Self::TMut<'_>;
}

impl<A, ARef> AsTupleRef<ARef> for A
where
    A: AsComponentRef<ARef>,
{
    type TRef<'a>
        = A::CRef<'a>
    where
        Self: 'a;

    fn as_tuple_ref(&self) -> Self::TRef<'_> {
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

    fn as_tuple_mut(&mut self) -> Self::TMut<'_> {
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

    fn as_tuple_ref(&self) -> Self::TRef<'_> {
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

    fn as_tuple_mut(&mut self) -> Self::TMut<'_> {
        (self.0.as_component_mut(),)
    }
}

impl<A, ARef, B, BRef> AsTupleRef<(ARef, BRef)> for (A, B)
where
    A: AsComponentRef<ARef>,
    B: AsComponentRef<BRef>,
{
    type TRef<'a>
        = (A::CRef<'a>, B::CRef<'a>)
    where
        Self: 'a;

    fn as_tuple_ref(&self) -> Self::TRef<'_> {
        (self.0.as_component_ref(), self.1.as_component_ref())
    }
}
impl<A, AMut, B, BMut> AsTupleMut<(AMut, BMut)> for (A, B)
where
    A: AsComponentMut<AMut>,
    B: AsComponentMut<BMut>,
{
    type TMut<'a>
        = (A::CMut<'a>, B::CMut<'a>)
    where
        Self: 'a;

    fn as_tuple_mut(&mut self) -> Self::TMut<'_> {
        (self.0.as_component_mut(), self.1.as_component_mut())
    }
}
