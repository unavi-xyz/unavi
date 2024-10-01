use std::cell::Cell;

use bevy::prelude::*;
use wasm_bridge::component::Resource;

use crate::{
    api::{
        utils::{RefCount, RefCountCell, RefResource},
        wired::scene::{
            bindings::material::{Color, Host, HostMaterial},
            MaterialState,
        },
    },
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

#[derive(Default, Debug)]
pub struct MaterialRes {
    pub name: String,
    pub color: Color,
    ref_count: RefCountCell,
}

impl RefCount for MaterialRes {
    fn ref_count(&self) -> &Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for MaterialRes {}

impl HostMaterial for ScriptData {
    fn new(&mut self) -> wasm_bridge::Result<wasm_bridge::component::Resource<MaterialRes>> {
        let table_res = self.table.push(MaterialRes::default())?;
        let res = MaterialRes::from_res(&table_res, &self.table)?;

        let materials = self.api.wired_scene.as_ref().unwrap().materials.clone();
        let rep = res.rep();
        self.commands.push(move |world: &mut World| {
            let mut assets = world.resource_mut::<Assets<StandardMaterial>>();
            let handle = assets.add(StandardMaterial::default());
            let entity = world
                .spawn(WiredMaterialBundle::new(rep, handle.clone()))
                .id();

            let mut materials = materials.write().unwrap();
            materials.insert(rep, MaterialState { entity, handle });
        });

        Ok(res)
    }

    fn id(&mut self, self_: Resource<MaterialRes>) -> wasm_bridge::Result<u32> {
        Ok(self_.rep())
    }
    fn ref_(&mut self, self_: Resource<MaterialRes>) -> wasm_bridge::Result<Resource<MaterialRes>> {
        let value = MaterialRes::from_res(&self_, &self.table)?;
        Ok(value)
    }

    fn name(&mut self, self_: Resource<MaterialRes>) -> wasm_bridge::Result<String> {
        let material = self.table.get(&self_)?;
        Ok(material.name.clone())
    }
    fn set_name(&mut self, self_: Resource<MaterialRes>, value: String) -> wasm_bridge::Result<()> {
        let material = self.table.get_mut(&self_)?;
        material.name = value;
        Ok(())
    }

    fn color(&mut self, self_: Resource<MaterialRes>) -> wasm_bridge::Result<Color> {
        let material = self.table.get(&self_)?;
        Ok(material.color)
    }
    fn set_color(&mut self, self_: Resource<MaterialRes>, value: Color) -> wasm_bridge::Result<()> {
        let material = self.table.get_mut(&self_)?;
        material.color = value;

        let color = bevy::prelude::Color::linear_rgba(
            material.color.r,
            material.color.g,
            material.color.b,
            material.color.a,
        );

        let materials = self.api.wired_scene.as_ref().unwrap().materials.clone();
        let rep = self_.rep();
        self.commands.push(move |world: &mut World| {
            let materials = materials.read().unwrap();
            let MaterialState { handle, .. } = materials.get(&rep).unwrap();
            let mut assets = world.resource_mut::<Assets<StandardMaterial>>();
            let material = assets.get_mut(handle).unwrap();
            material.base_color = color;
        });

        Ok(())
    }

    fn drop(&mut self, rep: Resource<MaterialRes>) -> wasm_bridge::Result<()> {
        let id = rep.rep();
        let dropped = MaterialRes::handle_drop(rep, &mut self.table)?;

        if dropped {
            let materials = self.api.wired_scene.as_ref().unwrap().materials.clone();
            self.commands.push(move |world: &mut World| {
                let mut materials = materials.write().unwrap();
                let MaterialState { entity, .. } = materials.remove(&id).unwrap();
                world.despawn(entity);
            });
        }

        Ok(())
    }
}

impl Host for ScriptData {}
