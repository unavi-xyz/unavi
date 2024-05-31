use anyhow::Result;
use bevy::utils::HashMap;
use wasm_component_layer::{ResourceOwn, ResourceType, StoreContextMut};

use super::{load::EngineBackend, StoreData};

#[derive(Default)]
pub struct ResourceTable {
    next_id: u32,
    resources: HashMap<u32, ResourceOwn>,
}

impl ResourceTable {
    fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn get(&self, id: &u32) -> Option<&ResourceOwn> {
        self.resources.get(id)
    }

    pub fn get_mut(&mut self, id: &u32) -> Option<&mut ResourceOwn> {
        self.resources.get_mut(id)
    }

    pub fn remove(&mut self, id: &u32) -> Option<ResourceOwn> {
        self.resources.remove(id)
    }

    pub fn push<T: 'static + Send + Sync>(
        &mut self,
        ctx: StoreContextMut<StoreData, EngineBackend>,
        resource_type: ResourceType,
        create_value: impl Fn(u32) -> T,
    ) -> Result<(u32, ResourceOwn)> {
        let id = self.next_id();
        let value = create_value(id);
        let resource = ResourceOwn::new(ctx, value, resource_type)?;
        self.resources.insert(id, resource.clone());
        Ok((id, resource))
    }
}
