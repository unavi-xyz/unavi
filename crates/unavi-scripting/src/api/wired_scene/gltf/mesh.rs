use std::cell::Cell;

use bevy::utils::HashSet;
use wasm_bridge::component::Resource;

use crate::{
    actions::ScriptAction,
    api::utils::{RefCount, RefCountCell, RefResource},
    state::StoreState,
};

use crate::api::wired_scene::wired::scene::mesh::{Host, HostMesh, HostPrimitive, Material};

#[derive(Default, Debug)]
pub struct Mesh {
    pub name: String,
    pub primitives: HashSet<u32>,
    ref_count: RefCountCell,
}

impl RefCount for Mesh {
    fn ref_count(&self) -> &Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for Mesh {}

#[derive(Default, Debug)]
pub struct Primitive {
    pub material: Option<u32>,
}

impl HostPrimitive for StoreState {
    fn id(&mut self, self_: Resource<Primitive>) -> wasm_bridge::Result<u32> {
        Ok(self_.rep())
    }

    fn material(
        &mut self,
        self_: Resource<Primitive>,
    ) -> wasm_bridge::Result<Option<Resource<Material>>> {
        let primitive = self.table.get(&self_)?;
        let material = match primitive.material {
            Some(m) => Some(Material::from_rep(m, &self.table)?),
            None => None,
        };
        Ok(material)
    }
    fn set_material(
        &mut self,
        self_: Resource<Primitive>,
        value: Option<Resource<Material>>,
    ) -> wasm_bridge::Result<()> {
        let primitive = self.table.get_mut(&self_)?;
        primitive.material = value.map(|v| v.rep());

        self.sender.send(ScriptAction::SetPrimitiveMaterial {
            id: self_.rep(),
            material: primitive.material,
        })?;

        Ok(())
    }

    fn set_indices(
        &mut self,
        self_: Resource<Primitive>,
        value: Vec<u32>,
    ) -> wasm_bridge::Result<()> {
        self.sender.send(ScriptAction::SetPrimitiveIndices {
            id: self_.rep(),
            value,
        })?;
        Ok(())
    }
    fn set_positions(
        &mut self,
        self_: Resource<Primitive>,
        value: Vec<f32>,
    ) -> wasm_bridge::Result<()> {
        self.sender.send(ScriptAction::SetPrimitivePositions {
            id: self_.rep(),
            value,
        })?;
        Ok(())
    }
    fn set_normals(
        &mut self,
        self_: Resource<Primitive>,
        value: Vec<f32>,
    ) -> wasm_bridge::Result<()> {
        self.sender.send(ScriptAction::SetPrimitiveNormals {
            id: self_.rep(),
            value,
        })?;
        Ok(())
    }
    fn set_uvs(&mut self, self_: Resource<Primitive>, value: Vec<f32>) -> wasm_bridge::Result<()> {
        self.sender.send(ScriptAction::SetPrimitiveUvs {
            id: self_.rep(),
            value,
        })?;
        Ok(())
    }

    fn drop(&mut self, _rep: Resource<Primitive>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}

impl HostMesh for StoreState {
    fn new(&mut self) -> wasm_bridge::Result<Resource<Mesh>> {
        let table_res = self.table.push(Mesh::default())?;
        let res = Mesh::from_res(&table_res, &self.table)?;
        self.meshes.push(table_res);

        self.sender
            .send(ScriptAction::CreateMesh { id: res.rep() })?;

        Ok(res)
    }

    fn id(&mut self, self_: Resource<Mesh>) -> wasm_bridge::Result<u32> {
        Ok(self_.rep())
    }

    fn name(&mut self, self_: Resource<Mesh>) -> wasm_bridge::Result<String> {
        let mesh = self.table.get(&self_)?;
        Ok(mesh.name.clone())
    }
    fn set_name(&mut self, self_: Resource<Mesh>, value: String) -> wasm_bridge::Result<()> {
        let mesh = self.table.get_mut(&self_)?;
        mesh.name = value;

        Ok(())
    }

    fn list_primitives(
        &mut self,
        self_: Resource<Mesh>,
    ) -> wasm_bridge::Result<Vec<Resource<Primitive>>> {
        let mesh = self.table.get_mut(&self_)?;
        Ok(mesh
            .primitives
            .iter()
            .map(|rep| Resource::new_own(*rep))
            .collect())
    }
    fn create_primitive(
        &mut self,
        self_: Resource<Mesh>,
    ) -> wasm_bridge::Result<Resource<Primitive>> {
        let resource = self.table.push(Primitive::default())?;
        let primitive_rep = resource.rep();
        self.primitives.push(resource);

        let mesh = self.table.get_mut(&self_)?;
        mesh.primitives.insert(primitive_rep);

        self.sender.send(ScriptAction::CreatePrimitive {
            id: primitive_rep,
            mesh: self_.rep(),
        })?;

        Ok(Resource::new_own(primitive_rep))
    }
    fn remove_primitive(
        &mut self,
        self_: Resource<Mesh>,
        value: Resource<Primitive>,
    ) -> wasm_bridge::Result<()> {
        let rep = value.rep();
        self.table.delete(Resource::new_own(rep))?;

        let mesh = self.table.get_mut(&self_)?;
        mesh.primitives.remove(&rep);

        self.sender.send(ScriptAction::RemovePrimitive {
            id: rep,
            mesh: self_.rep(),
        })?;

        Ok(())
    }

    fn drop(&mut self, rep: Resource<Mesh>) -> wasm_bridge::Result<()> {
        let id = rep.rep();
        let dropped = Mesh::handle_drop(rep, &mut self.table)?;

        if dropped {
            let index = self.meshes.iter().enumerate().find_map(|(i, item)| {
                if item.rep() == id {
                    Some(i)
                } else {
                    None
                }
            });
            if let Some(index) = index {
                self.meshes.remove(index);
            }

            self.sender.send(ScriptAction::RemoveMesh { id })?;
        }

        Ok(())
    }
}

impl Host for StoreState {}

#[cfg(test)]
mod tests {
    crate::generate_resource_tests!(Mesh);
}
