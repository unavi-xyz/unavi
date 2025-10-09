use crate::Component;

use super::component::QueriedComponent;

pub trait AsComponentRef<T> {
    type CRef<'a>
    where
        Self: 'a;
    fn as_component_ref<'a>(&'a self) -> Self::CRef<'a>;
}
pub trait AsComponentMut<T> {
    type CMut<'a>
    where
        Self: 'a;
    fn as_component_mut<'a>(&'a mut self) -> Self::CMut<'a>;
}

impl<T> AsComponentRef<&T> for T
where
    T: Component,
{
    type CRef<'a>
        = &'a T
    where
        Self: 'a;
    fn as_component_ref<'a>(&'a self) -> Self::CRef<'a> {
        self
    }
}
impl<T> AsComponentMut<&T> for T
where
    T: Component,
{
    type CMut<'a>
        = &'a T
    where
        Self: 'a;
    fn as_component_mut<'a>(&'a mut self) -> Self::CMut<'a> {
        self
    }
}
impl<T> AsComponentMut<&mut T> for T
where
    T: Component,
{
    type CMut<'a>
        = &'a mut T
    where
        Self: 'a;
    fn as_component_mut<'a>(&'a mut self) -> Self::CMut<'a> {
        self
    }
}

impl<T> AsComponentRef<T> for T
where
    T: QueriedComponent + Copy,
{
    type CRef<'a>
        = T
    where
        Self: 'a;
    fn as_component_ref<'a>(&'a self) -> Self::CRef<'a> {
        *self
    }
}
impl<T> AsComponentMut<T> for T
where
    T: QueriedComponent + Copy,
{
    type CMut<'a>
        = T
    where
        Self: 'a;
    fn as_component_mut<'a>(&'a mut self) -> Self::CMut<'a> {
        *self
    }
}
