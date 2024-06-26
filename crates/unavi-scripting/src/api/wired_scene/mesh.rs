use bevy::utils::HashSet;
use wasm_bridge::component::Resource;

use crate::{actions::ScriptAction, state::StoreState};

use super::wired::scene::mesh::{Host, HostMesh, HostPrimitive, Material};

#[derive(Default)]
pub struct Mesh {
    pub name: String,
    pub primitives: HashSet<u32>,
}

#[derive(Default)]
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
        Ok(primitive.material.map(Resource::new_own))
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
        self.table.delete(value)?;

        let index = self.primitives.iter().enumerate().find_map(|(i, item)| {
            if item.rep() == rep {
                Some(i)
            } else {
                None
            }
        });
        if let Some(index) = index {
            self.primitives.remove(index);
        }

        let mesh = self.table.get_mut(&self_)?;
        mesh.primitives.remove(&rep);

        self.sender.send(ScriptAction::RemovePrimitive {
            id: rep,
            mesh: self_.rep(),
        })?;

        Ok(())
    }

    fn drop(&mut self, _rep: Resource<Mesh>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}

impl Host for StoreState {
    fn list_meshes(&mut self) -> wasm_bridge::Result<Vec<Resource<Mesh>>> {
        Ok(self
            .meshes
            .iter()
            .map(|res| Resource::new_own(res.rep()))
            .collect())
    }

    fn create_mesh(&mut self) -> wasm_bridge::Result<Resource<Mesh>> {
        let resource = self.table.push(Mesh::default())?;
        let mesh_rep = resource.rep();
        self.meshes.push(resource);

        self.sender
            .send(ScriptAction::CreateMesh { id: mesh_rep })?;

        Ok(Resource::new_own(mesh_rep))
    }

    fn remove_mesh(&mut self, value: Resource<Mesh>) -> wasm_bridge::Result<()> {
        let rep = value.rep();
        self.table.delete(value)?;

        let index =
            self.meshes
                .iter()
                .enumerate()
                .find_map(|(i, item)| if item.rep() == rep { Some(i) } else { None });
        if let Some(index) = index {
            self.meshes.remove(index);
        }

        self.sender.send(ScriptAction::RemoveMesh { id: rep })?;

        Ok(())
    }
}
