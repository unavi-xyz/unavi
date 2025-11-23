use crate::{
    Component, ParamState, host_api,
    types::{EntityId, Param as WParam, ParamData},
};

use super::{
    Param, ParamMeta,
    component::QueriedComponent,
    owned_component::{HostWriteable, OwnedComponent},
};

pub struct Commands;

impl Param for Commands {
    fn register_param(_: &mut Vec<ParamState>) -> Option<WParam> {
        None
    }
    fn meta() -> Option<ParamMeta> {
        None
    }
    fn parse_param(
        _: &mut std::slice::IterMut<ParamState>,
        _: &mut std::vec::IntoIter<ParamData>,
    ) -> Self {
        Commands
    }
}

impl Commands {
    /// Spawn a new entity.
    ///
    /// # Panics
    ///
    /// Panics if entity spawning fails.
    #[must_use]
    pub fn spawn(&self) -> Entity {
        Entity(host_api::spawn().expect("spawn"))
    }
}

#[derive(Clone, Copy)]
pub struct Entity(EntityId);

impl Entity {
    #[must_use]
    pub fn id(&self) -> u64 {
        self.0
    }

    /// Insert a component on this entity.
    ///
    /// # Panics
    ///
    /// Panics if component insertion fails.
    pub fn insert<T: Component>(&self, data: T) {
        let c_id = T::register();
        host_api::insert_component(self.0, c_id, &data.to_bytes()).expect("insert component");
    }

    /// Remove a component from this entity.
    ///
    /// # Panics
    ///
    /// Panics if component removal fails.
    pub fn remove<T: Component>(&self) {
        let c_id = T::register();
        host_api::remove_component(self.0, c_id).expect("remove component");
    }
}

impl HostWriteable for Entity {
    fn write(&self, _: u64, _: u32) {}
}

impl QueriedComponent for Entity {
    type Owned = Self;
    type Ref = Self;
    type Mut = Self;

    fn register() -> Option<u32> {
        None
    }
    fn mutability() -> Option<bool> {
        None
    }

    fn from_bytes(
        entity: u64,
        _: &mut std::vec::IntoIter<Vec<u8>>,
    ) -> OwnedComponent<Self::Owned, Self::Ref, Self::Mut> {
        OwnedComponent::new(Entity(entity))
    }
}
