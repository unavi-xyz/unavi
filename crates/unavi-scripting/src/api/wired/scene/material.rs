use std::sync::{Arc, OnceLock, RwLock};

use bevy::prelude::*;
use wasm_bridge::component::Resource;

use crate::{
    api::wired::scene::bindings::material::{Color, Host, HostMaterial},
    data::ScriptData,
};

#[derive(Component, Clone, Copy, Debug)]
pub struct MaterialId(pub u32);

#[derive(Bundle)]
pub struct WiredMaterialBundle {
    pub id: MaterialId,
    pub handle: Handle<StandardMaterial>,
}

impl WiredMaterialBundle {
    pub fn new(id: u32, handle: Handle<StandardMaterial>) -> Self {
        Self {
            id: MaterialId(id),
            handle,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct MaterialRes(pub Arc<RwLock<MaterialData>>);

#[derive(Default, Debug)]
pub struct MaterialData {
    pub id: u32,
    pub color: Color,
    pub handle: OnceLock<Handle<StandardMaterial>>,
    pub name: String,
}

impl HostMaterial for ScriptData {
    fn new(&mut self) -> wasm_bridge::Result<wasm_bridge::component::Resource<MaterialRes>> {
        let res = MaterialRes::default();
        let data = res.0.clone();

        let res = self.table.push(res)?;

        self.commands.push(move |world: &mut World| {
            let mut assets = world.resource_mut::<Assets<StandardMaterial>>();
            let handle = assets.add(StandardMaterial::default());
            data.write().unwrap().handle.get_or_init(|| handle);
        });

        Ok(res)
    }

    fn id(&mut self, self_: Resource<MaterialRes>) -> wasm_bridge::Result<u32> {
        let data = self.table.get(&self_)?.0.read().unwrap();
        Ok(data.id)
    }
    fn ref_(&mut self, self_: Resource<MaterialRes>) -> wasm_bridge::Result<Resource<MaterialRes>> {
        let data = self.table.get(&self_)?.clone();
        let res = self.table.push(data)?;
        Ok(res)
    }

    fn name(&mut self, self_: Resource<MaterialRes>) -> wasm_bridge::Result<String> {
        let data = self.table.get(&self_)?.0.read().unwrap();
        Ok(data.name.clone())
    }
    fn set_name(&mut self, self_: Resource<MaterialRes>, value: String) -> wasm_bridge::Result<()> {
        let mut data = self.table.get_mut(&self_)?.0.write().unwrap();
        data.name = value;
        Ok(())
    }

    fn color(&mut self, self_: Resource<MaterialRes>) -> wasm_bridge::Result<Color> {
        let data = self.table.get(&self_)?.0.read().unwrap();
        Ok(data.color)
    }
    fn set_color(&mut self, self_: Resource<MaterialRes>, value: Color) -> wasm_bridge::Result<()> {
        let res = self.table.get_mut(&self_)?;

        let mut data = res.0.write().unwrap();
        data.color = value;

        let color = bevy::prelude::Color::linear_rgba(
            data.color.r,
            data.color.g,
            data.color.b,
            data.color.a,
        );

        let res = res.0.clone();

        self.commands.push(move |world: &mut World| {
            let handle = &res.read().unwrap().handle;
            let mut assets = world.resource_mut::<Assets<StandardMaterial>>();
            let material = assets.get_mut(handle.get().unwrap()).unwrap();
            material.base_color = color;
        });

        Ok(())
    }

    fn drop(&mut self, rep: Resource<MaterialRes>) -> wasm_bridge::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}

impl Host for ScriptData {}

#[cfg(test)]
mod tests {
    use crate::api::tests::init_test_data;

    use super::*;

    #[test]
    fn test_resource_drop() {
        let (_, mut data) = init_test_data();

        let res = HostMaterial::new(&mut data).unwrap();
        let res_clone = Resource::<MaterialRes>::new_own(res.rep());

        assert!(data.table.get(&res).is_ok());

        HostMaterial::drop(&mut data, res).unwrap();

        assert!(data.table.get(&res_clone).is_err());
    }
}
