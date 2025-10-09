use std::marker::PhantomData;

use crate::Component;

use super::component_ref::{AsComponentMut, AsComponentRef};

pub struct OwnedComponent<T, R, M> {
    owned: T,
    _r: PhantomData<R>,
    _m: PhantomData<M>,
}

impl<T, R, M> OwnedComponent<T, R, M> {
    pub fn new(owned: T) -> Self {
        OwnedComponent {
            owned,
            _r: PhantomData,
            _m: PhantomData,
        }
    }
}

impl<T, M> AsRef<T> for OwnedComponent<T, &T, M> {
    fn as_ref(&self) -> &T {
        &self.owned
    }
}
impl<T, R> AsMut<T> for OwnedComponent<T, R, &mut T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.owned
    }
}

impl<T> AsComponentRef<T> for OwnedComponent<T, T, T>
where
    T: Copy,
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
    T: Copy,
{
    type CMut<'a>
        = T
    where
        Self: 'a;
    fn as_component_mut<'a>(&'a mut self) -> Self::CMut<'a> {
        self.owned
    }
}

impl<T, M> AsComponentRef<T> for OwnedComponent<T, &T, M> {
    type CRef<'a>
        = &'a T
    where
        Self: 'a;
    fn as_component_ref<'a>(&'a self) -> Self::CRef<'a> {
        &self.owned
    }
}
impl<T, M> AsComponentRef<&T> for OwnedComponent<T, &T, M> {
    type CRef<'a>
        = &'a T
    where
        Self: 'a;
    fn as_component_ref<'a>(&'a self) -> Self::CRef<'a> {
        &self.owned
    }
}
impl<T, R> AsComponentMut<T> for OwnedComponent<T, R, &mut T> {
    type CMut<'a>
        = &'a mut T
    where
        Self: 'a;
    fn as_component_mut<'a>(&'a mut self) -> Self::CMut<'a> {
        &mut self.owned
    }
}
impl<T, R> AsComponentMut<&T> for OwnedComponent<T, R, &T> {
    type CMut<'a>
        = &'a T
    where
        Self: 'a;
    fn as_component_mut<'a>(&'a mut self) -> Self::CMut<'a> {
        &self.owned
    }
}
impl<T, R> AsComponentMut<&mut T> for OwnedComponent<T, R, &mut T> {
    type CMut<'a>
        = &'a mut T
    where
        Self: 'a;
    fn as_component_mut<'a>(&'a mut self) -> Self::CMut<'a> {
        &mut self.owned
    }
}

pub trait QueriedComponent: Sized {
    type Owned: AsComponentRef<Self::Ref> + AsComponentMut<Self::Mut>;
    type Ref;
    type Mut;

    fn register() -> Option<u32>;
    fn mutability() -> Option<bool>;

    fn from_bytes(entity: u64, bytes: Vec<u8>) -> OwnedComponent<Self::Owned, Self::Ref, Self::Mut>
    where
        OwnedComponent<Self::Owned, Self::Ref, Self::Mut>:
            AsComponentRef<Self::Ref> + AsComponentMut<Self::Mut>;
    fn to_bytes(&self) -> Vec<u8>;
}

impl<'t, T: Component> QueriedComponent for &'t T {
    type Owned = T;
    type Ref = &'t T;
    type Mut = &'t T;

    fn register() -> Option<u32> {
        Some(T::register())
    }
    fn mutability() -> Option<bool> {
        Some(false)
    }

    fn from_bytes(_: u64, bytes: Vec<u8>) -> OwnedComponent<Self::Owned, Self::Ref, Self::Mut> {
        let owned = T::from_bytes(bytes);
        OwnedComponent::new(owned)
    }
    fn to_bytes(&self) -> Vec<u8> {
        T::to_bytes(self)
    }
}

impl<'t, T: Component> QueriedComponent for &'t mut T {
    type Owned = T;
    type Ref = &'t T;
    type Mut = &'t mut T;

    fn register() -> Option<u32> {
        Some(T::register())
    }
    fn mutability() -> Option<bool> {
        Some(false)
    }

    fn from_bytes(_: u64, bytes: Vec<u8>) -> OwnedComponent<Self::Owned, Self::Ref, Self::Mut> {
        let owned = T::from_bytes(bytes);
        OwnedComponent::new(owned)
    }
    fn to_bytes(&self) -> Vec<u8> {
        T::to_bytes(self)
    }
}
