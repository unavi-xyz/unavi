use std::cell::Cell;

use wasm_bridge::component::Resource;

use crate::{
    actions::ScriptAction,
    api::utils::{RefCount, RefCountCell, RefResource},
    state::StoreState,
};

use crate::api::wired_scene::wired::scene::material::{Color, Host, HostMaterial};

#[derive(Default, Debug)]
pub struct Material {
    pub name: String,
    pub color: Color,
    ref_count: RefCountCell,
}

impl RefCount for Material {
    fn ref_count(&self) -> &Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for Material {}

impl HostMaterial for StoreState {
    fn new(&mut self) -> wasm_bridge::Result<wasm_bridge::component::Resource<Material>> {
        let table_res = self.table.push(Material::default())?;
        let res = Material::from_res(&table_res, &self.table)?;
        self.materials.push(table_res);

        self.sender
            .send(ScriptAction::CreateMaterial { id: res.rep() })?;

        Ok(res)
    }

    fn id(&mut self, self_: Resource<Material>) -> wasm_bridge::Result<u32> {
        Ok(self_.rep())
    }

    fn name(&mut self, self_: Resource<Material>) -> wasm_bridge::Result<String> {
        let material = self.table.get(&self_)?;
        Ok(material.name.clone())
    }
    fn set_name(&mut self, self_: Resource<Material>, value: String) -> wasm_bridge::Result<()> {
        let material = self.table.get_mut(&self_)?;
        material.name = value;
        Ok(())
    }

    fn color(&mut self, self_: Resource<Material>) -> wasm_bridge::Result<Color> {
        let material = self.table.get(&self_)?;
        Ok(material.color)
    }
    fn set_color(&mut self, self_: Resource<Material>, value: Color) -> wasm_bridge::Result<()> {
        let material = self.table.get_mut(&self_)?;
        material.color = value;

        self.sender.send(ScriptAction::SetMaterialColor {
            id: self_.rep(),
            color: bevy::prelude::Color::rgba(
                material.color.r,
                material.color.g,
                material.color.b,
                material.color.a,
            ),
        })?;

        Ok(())
    }

    fn drop(&mut self, rep: Resource<Material>) -> wasm_bridge::Result<()> {
        let id = rep.rep();
        let dropped = Material::handle_drop(rep, &mut self.table)?;
        if dropped {
            bevy::log::info!("Dropping mat: {}", id);
            let index = self.materials.iter().enumerate().find_map(|(i, item)| {
                if item.rep() == id {
                    Some(i)
                } else {
                    None
                }
            });
            if let Some(index) = index {
                self.materials.remove(index);
            }

            self.sender.send(ScriptAction::RemoveMaterial { id })?;
        }
        Ok(())
    }
}

impl Host for StoreState {}

#[cfg(test)]
mod tests {
    crate::generate_resource_tests!(Material);
}
