use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

use crate::Component;

use super::{
    component_ref::{AsComponentMut, AsComponentRef},
    owned_component::{HostWriteable, OnDrop, OwnedComponent},
};

pub trait QueriedComponent: Sized {
    type Owned: HostWriteable + AsComponentRef<Self::Ref> + AsComponentMut<Self::Mut>;
    type Ref;
    type Mut;

    fn register() -> Option<u32>;
    fn mutability() -> Option<bool>;

    fn from_bytes(
        entity: u64,
        bytes: &mut std::vec::IntoIter<Vec<u8>>,
    ) -> OwnedComponent<Self::Owned, Self::Ref, Self::Mut>
    where
        OwnedComponent<Self::Owned, Self::Ref, Self::Mut>:
            AsComponentRef<Self::Ref> + AsComponentMut<Self::Mut>;
}

static COMPONENT_IDS: LazyLock<Arc<Mutex<HashMap<String, u32>>>> = LazyLock::new(Arc::default);

impl<'t, T: Component> QueriedComponent for &'t T {
    type Owned = T;
    type Ref = &'t T;
    type Mut = &'t T;

    fn register() -> Option<u32> {
        let id = T::register();
        COMPONENT_IDS.lock().unwrap().insert(T::key(), id);
        Some(id)
    }
    fn mutability() -> Option<bool> {
        Some(false)
    }

    fn from_bytes(
        _: u64,
        bytes: &mut std::vec::IntoIter<Vec<u8>>,
    ) -> OwnedComponent<Self::Owned, Self::Ref, Self::Mut> {
        let owned = T::from_bytes(bytes.next().unwrap());
        OwnedComponent::new(owned)
    }
}

impl<'t, T: Component> QueriedComponent for &'t mut T {
    type Owned = T;
    type Ref = &'t T;
    type Mut = &'t mut T;

    fn register() -> Option<u32> {
        let id = T::register();
        COMPONENT_IDS.lock().unwrap().insert(T::key(), id);
        Some(id)
    }
    fn mutability() -> Option<bool> {
        Some(true)
    }

    fn from_bytes(
        entity: u64,
        bytes: &mut std::vec::IntoIter<Vec<u8>>,
    ) -> OwnedComponent<Self::Owned, Self::Ref, Self::Mut> {
        let owned = T::from_bytes(bytes.next().unwrap());
        let mut c = OwnedComponent::new(owned);

        let component = *COMPONENT_IDS
            .lock()
            .unwrap()
            .get(&T::key())
            .expect("component not registered");
        c.on_drop(OnDrop::WriteComponent { entity, component });

        c
    }
}
