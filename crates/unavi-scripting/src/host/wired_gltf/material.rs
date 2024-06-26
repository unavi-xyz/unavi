use crate::StoreState;
use wasm_bridge::component::Resource;

use super::{
    bindgen::wired::gltf::material::{Color, Host, HostMaterial, Material},
    WiredGltfAction,
};

impl HostMaterial for StoreState {
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

        self.sender.send(WiredGltfAction::SetMaterialColor {
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

    fn drop(&mut self, _rep: Resource<Material>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}

impl Host for StoreState {
    fn list_materials(&mut self) -> wasm_bridge::Result<Vec<Resource<Material>>> {
        Ok(self
            .materials
            .iter()
            .map(|rep| Resource::new_own(*rep))
            .collect())
    }

    fn create_material(&mut self) -> wasm_bridge::Result<Resource<Material>> {
        let resource = self.table.push(Material::default())?;
        let material_rep = resource.rep();
        self.materials.push(material_rep);

        self.sender
            .send(WiredGltfAction::CreateMaterial { id: material_rep })?;

        Ok(Resource::new_own(material_rep))
    }

    fn remove_material(&mut self, value: Resource<Material>) -> wasm_bridge::Result<()> {
        let rep = value.rep();
        self.table.delete(value)?;

        let index =
            self.materials
                .iter()
                .enumerate()
                .find_map(|(i, item)| if *item == rep { Some(i) } else { None });
        if let Some(index) = index {
            self.materials.remove(index);
        }

        self.sender
            .send(WiredGltfAction::RemoveMaterial { id: rep })?;

        Ok(())
    }
}
