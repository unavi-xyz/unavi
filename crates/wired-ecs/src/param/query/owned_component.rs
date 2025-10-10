use std::marker::PhantomData;

use crate::{Component, host_api::write_component};

use super::component_ref::{AsComponentMut, AsComponentRef};

pub enum OnDrop {
    WriteComponent { entity: u64, component: u32 },
}

pub trait HostWriteable {
    fn write(&self, entity: u64, component: u32);
}

impl<T> HostWriteable for T
where
    T: Component,
{
    fn write(&self, entity: u64, component: u32) {
        let data = self.to_bytes();
        if let Err(e) = write_component(entity, component, &data) {
            panic!("Error writing component: {e}")
        }
    }
}

pub struct OwnedComponent<T, R, M>
where
    T: HostWriteable,
{
    owned: T,
    on_drop: Option<OnDrop>,
    _r: PhantomData<R>,
    _m: PhantomData<M>,
}

impl<T, R, M> OwnedComponent<T, R, M>
where
    T: HostWriteable,
{
    pub fn new(owned: T) -> Self {
        OwnedComponent {
            owned,
            on_drop: None,
            _r: PhantomData,
            _m: PhantomData,
        }
    }

    pub fn on_drop(&mut self, value: OnDrop) {
        self.on_drop = Some(value);
    }
}

impl<T, R, M> Drop for OwnedComponent<T, R, M>
where
    T: HostWriteable,
{
    fn drop(&mut self) {
        match self.on_drop {
            Some(OnDrop::WriteComponent { entity, component }) => {
                self.owned.write(entity, component);
            }
            None => {}
        }
    }
}

impl<T, M> AsRef<T> for OwnedComponent<T, &T, M>
where
    T: HostWriteable,
{
    fn as_ref(&self) -> &T {
        &self.owned
    }
}
impl<T, R> AsMut<T> for OwnedComponent<T, R, &mut T>
where
    T: HostWriteable,
{
    fn as_mut(&mut self) -> &mut T {
        &mut self.owned
    }
}

impl<T> AsComponentRef<T> for OwnedComponent<T, T, T>
where
    T: Copy + HostWriteable,
{
    type CRef<'a>
        = T
    where
        Self: 'a;
    fn as_component_ref<'a>(&'a self) -> Self::CRef<'a> {
        self.owned
    }
}
impl<T> AsComponentMut<T> for OwnedComponent<T, T, T>
where
    T: Copy + HostWriteable,
{
    type CMut<'a>
        = T
    where
        Self: 'a;
    fn as_component_mut<'a>(&'a mut self) -> Self::CMut<'a> {
        self.owned
    }
}

impl<T, M> AsComponentRef<T> for OwnedComponent<T, &T, M>
where
    T: HostWriteable,
{
    type CRef<'a>
        = &'a T
    where
        Self: 'a;
    fn as_component_ref<'a>(&'a self) -> Self::CRef<'a> {
        &self.owned
    }
}
impl<T, M> AsComponentRef<&T> for OwnedComponent<T, &T, M>
where
    T: HostWriteable,
{
    type CRef<'a>
        = &'a T
    where
        Self: 'a;
    fn as_component_ref<'a>(&'a self) -> Self::CRef<'a> {
        &self.owned
    }
}
impl<T, R> AsComponentMut<T> for OwnedComponent<T, R, &mut T>
where
    T: HostWriteable,
{
    type CMut<'a>
        = &'a mut T
    where
        Self: 'a;
    fn as_component_mut<'a>(&'a mut self) -> Self::CMut<'a> {
        &mut self.owned
    }
}
impl<T, R> AsComponentMut<&T> for OwnedComponent<T, R, &T>
where
    T: HostWriteable,
{
    type CMut<'a>
        = &'a T
    where
        Self: 'a;
    fn as_component_mut<'a>(&'a mut self) -> Self::CMut<'a> {
        &self.owned
    }
}
impl<T, R> AsComponentMut<&mut T> for OwnedComponent<T, R, &mut T>
where
    T: HostWriteable,
{
    type CMut<'a>
        = &'a mut T
    where
        Self: 'a;
    fn as_component_mut<'a>(&'a mut self) -> Self::CMut<'a> {
        &mut self.owned
    }
}
