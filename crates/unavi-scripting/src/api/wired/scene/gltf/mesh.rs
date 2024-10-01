use std::cell::Cell;

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
        render_asset::RenderAssetUsages,
    },
};
use wasm_bridge::component::Resource;

use crate::{
    api::{
        utils::{RefCount, RefCountCell, RefResource},
        wired::scene::bindings::mesh::{Host, HostMesh, HostPrimitive, Material},
    },
    data::ScriptData,
};

use super::{
    material::MaterialRes,
    node::{NodeMesh, NodeRes},
};

#[derive(Component, Clone, Copy, Debug)]
pub struct MeshId(pub u32);

#[derive(Bundle)]
pub struct GltfMeshBundle {
    pub id: MeshId,
}

impl GltfMeshBundle {
    pub fn new(id: u32) -> Self {
        Self { id: MeshId(id) }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct PrimitiveId(pub u32);

#[derive(Bundle)]
pub struct GltfPrimitiveBundle {
    pub handle: Handle<Mesh>,
    pub id: PrimitiveId,
    pub mesh: MeshId,
}

#[derive(Default, Debug)]
pub struct MeshRes {
    pub name: String,
    /// Nodes that are using this mesh.
    pub nodes: Vec<Resource<NodeRes>>,
    pub primitives: Vec<Resource<PrimitiveRes>>,
    ref_count: RefCountCell,
}

impl RefCount for MeshRes {
    fn ref_count(&self) -> &Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for MeshRes {}

#[derive(Debug)]
pub struct PrimitiveRes {
    pub material: Option<Resource<MaterialRes>>,
    mesh: u32,
    ref_count: RefCountCell,
}

impl PrimitiveRes {
    pub fn new(mesh: &Resource<MeshRes>) -> Self {
        Self {
            material: None,
            mesh: mesh.rep(),
            ref_count: RefCountCell::default(),
        }
    }
}

impl RefCount for PrimitiveRes {
    fn ref_count(&self) -> &Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for PrimitiveRes {}

impl HostMesh for ScriptData {
    fn new(&mut self) -> wasm_bridge::Result<Resource<MeshRes>> {
        let table_res = self.table.push(MeshRes::default())?;
        let res = self.clone_res(&table_res)?;
        Ok(res)
    }

    fn id(&mut self, self_: Resource<MeshRes>) -> wasm_bridge::Result<u32> {
        Ok(self_.rep())
    }
    fn ref_(&mut self, self_: Resource<MeshRes>) -> wasm_bridge::Result<Resource<MeshRes>> {
        let value = MeshRes::from_res(&self_, &self.table)?;
        Ok(value)
    }

    fn name(&mut self, self_: Resource<MeshRes>) -> wasm_bridge::Result<String> {
        let mesh = self.table.get(&self_)?;
        Ok(mesh.name.clone())
    }
    fn set_name(&mut self, self_: Resource<MeshRes>, value: String) -> wasm_bridge::Result<()> {
        let mesh = self.table.get_mut(&self_)?;
        mesh.name = value;
        Ok(())
    }

    fn list_primitives(
        &mut self,
        self_: Resource<MeshRes>,
    ) -> wasm_bridge::Result<Vec<Resource<PrimitiveRes>>> {
        let mesh = self.table.get(&self_)?;
        Ok(mesh
            .primitives
            .iter()
            .map(|res| self.clone_res(res))
            .collect::<Result<_, _>>()?)
    }
    fn create_primitive(
        &mut self,
        self_: Resource<MeshRes>,
    ) -> wasm_bridge::Result<Resource<PrimitiveRes>> {
        let resource = self.table.push(PrimitiveRes::new(&self_))?;
        let primitive_rep = resource.rep();

        let res = self.clone_res(&resource)?;
        let mesh = self.table.get_mut(&self_)?;
        mesh.primitives.push(res);

        let mesh_nodes = mesh.nodes.iter().map(|res| res.rep()).collect::<Vec<_>>();
        let nodes = self.api.wired_scene.as_ref().unwrap().nodes.clone();
        let primitives = self.api.wired_scene.as_ref().unwrap().primitives.clone();
        self.commands.push(move |world: &mut World| {
            let mut assets = world.resource_mut::<Assets<Mesh>>();
            let handle = assets.add(Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::all(),
            ));

            let mut primitives = primitives.write().unwrap();
            primitives.insert(primitive_rep, handle.clone());

            // Create node primitives.
            let nodes = nodes.read().unwrap();
            for node_id in mesh_nodes {
                let node_ent = nodes.get(&node_id).unwrap();

                let p_ent = world
                    .spawn(PbrBundle {
                        mesh: handle.clone(),
                        ..default()
                    })
                    .set_parent(*node_ent)
                    .id();

                let mut node_mesh = world.get_mut::<NodeMesh>(*node_ent).unwrap();
                node_mesh.node_primitives.insert(primitive_rep, p_ent);
            }
        });

        Ok(Resource::new_own(primitive_rep))
    }
    fn remove_primitive(
        &mut self,
        self_: Resource<MeshRes>,
        value: Resource<PrimitiveRes>,
    ) -> wasm_bridge::Result<()> {
        let rep = value.rep();
        self.table.delete(value)?;

        let mesh = self.table.get_mut(&self_)?;
        mesh.primitives
            .iter()
            .position(|r| r.rep() == rep)
            .map(|index| mesh.primitives.remove(index));

        let node_ids = mesh.nodes.iter().map(|res| res.rep()).collect::<Vec<_>>();
        let nodes = self.api.wired_scene.as_ref().unwrap().nodes.clone();
        self.commands.push(move |world: &mut World| {
            // Remove node primitives.
            let nodes = nodes.read().unwrap();
            for node_id in node_ids {
                let entity = nodes.get(&node_id).unwrap();
                let mut node_mesh = world.get_mut::<NodeMesh>(*entity).unwrap();
                if let Some(p_ent) = node_mesh.node_primitives.remove(&rep) {
                    world.despawn(p_ent);
                }
            }
        });

        Ok(())
    }

    fn drop(&mut self, rep: Resource<MeshRes>) -> wasm_bridge::Result<()> {
        let dropped = MeshRes::handle_drop(rep, &mut self.table)?;

        if dropped {
            // TODO: Remove primitives
        }

        Ok(())
    }
}

impl HostPrimitive for ScriptData {
    fn id(&mut self, self_: Resource<PrimitiveRes>) -> wasm_bridge::Result<u32> {
        Ok(self_.rep())
    }

    fn material(
        &mut self,
        self_: Resource<PrimitiveRes>,
    ) -> wasm_bridge::Result<Option<Resource<Material>>> {
        let primitive = self.table.get(&self_)?;
        let material = match &primitive.material {
            Some(m) => Some(self.clone_res(m)?),
            None => None,
        };
        Ok(material)
    }
    fn set_material(
        &mut self,
        self_: Resource<PrimitiveRes>,
        value: Option<Resource<Material>>,
    ) -> wasm_bridge::Result<()> {
        let material_rep = value.as_ref().map(|r| r.rep());

        let res = value.and_then(|v| self.clone_res(&v).ok());
        let primitive = self.table.get_mut(&self_)?;
        primitive.material = res;

        let mesh = Resource::<MeshRes>::new_own(primitive.mesh);
        let mesh = self.table.get(&mesh)?;

        let mesh_nodes = mesh.nodes.iter().map(|r| r.rep()).collect::<Vec<_>>();

        let default_material = self
            .api
            .wired_scene
            .as_ref()
            .unwrap()
            .default_material
            .clone();
        let materials = self.api.wired_scene.as_ref().unwrap().materials.clone();
        let nodes = self.api.wired_scene.as_ref().unwrap().nodes.clone();
        let rep = self_.rep();
        self.commands.push(move |world: &mut World| {
            let materials = materials.read().unwrap();
            let nodes = nodes.read().unwrap();

            let material = material_rep.map(|r| materials.get(&r).unwrap().handle.clone());

            for nid in mesh_nodes {
                let node_ent = nodes.get(&nid).unwrap(); // TODO: is this unwrap safe?
                let node_mesh = world.get::<NodeMesh>(*node_ent).unwrap();

                for (pid, p_ent) in node_mesh.node_primitives.clone() {
                    if pid == rep {
                        if let Some(material) = &material {
                            world.entity_mut(p_ent).insert(material.clone());
                        } else {
                            world.entity_mut(p_ent).insert(default_material.clone());
                        }
                    }
                }
            }
        });

        Ok(())
    }

    fn set_indices(
        &mut self,
        self_: Resource<PrimitiveRes>,
        value: Vec<u32>,
    ) -> wasm_bridge::Result<()> {
        let primitives = self.api.wired_scene.as_ref().unwrap().primitives.clone();
        let rep = self_.rep();
        self.commands.push(move |world: &mut World| {
            let primitives = primitives.read().unwrap();
            let handle = primitives.get(&rep).unwrap();
            let mut assets = world.resource_mut::<Assets<Mesh>>();
            let mesh = assets.get_mut(handle).unwrap();
            mesh.insert_indices(Indices::U32(value));
        });
        Ok(())
    }
    fn set_positions(
        &mut self,
        self_: Resource<PrimitiveRes>,
        value: Vec<f32>,
    ) -> wasm_bridge::Result<()> {
        let primitives = self.api.wired_scene.as_ref().unwrap().primitives.clone();
        let rep = self_.rep();
        self.commands.push(move |world: &mut World| {
            let primitives = primitives.read().unwrap();
            let handle = primitives.get(&rep).unwrap();
            let mut assets = world.resource_mut::<Assets<Mesh>>();
            let mesh = assets.get_mut(handle).unwrap();

            let value = value.chunks(3).map(|x| [x[0], x[1], x[2]]).collect();

            mesh.insert_attribute(
                Mesh::ATTRIBUTE_POSITION,
                VertexAttributeValues::Float32x3(value),
            );
        });
        Ok(())
    }
    fn set_normals(
        &mut self,
        self_: Resource<PrimitiveRes>,
        value: Vec<f32>,
    ) -> wasm_bridge::Result<()> {
        let primitives = self.api.wired_scene.as_ref().unwrap().primitives.clone();
        let rep = self_.rep();
        self.commands.push(move |world: &mut World| {
            let primitives = primitives.read().unwrap();
            let handle = primitives.get(&rep).unwrap();
            let mut assets = world.resource_mut::<Assets<Mesh>>();
            let mesh = assets.get_mut(handle).unwrap();

            let value = value.chunks(3).map(|x| [x[0], x[1], x[2]]).collect();

            mesh.insert_attribute(
                Mesh::ATTRIBUTE_NORMAL,
                VertexAttributeValues::Float32x3(value),
            );
        });
        Ok(())
    }
    fn set_uvs(
        &mut self,
        self_: Resource<PrimitiveRes>,
        value: Vec<f32>,
    ) -> wasm_bridge::Result<()> {
        let primitives = self.api.wired_scene.as_ref().unwrap().primitives.clone();
        let rep = self_.rep();
        self.commands.push(move |world: &mut World| {
            let primitives = primitives.read().unwrap();
            let handle = primitives.get(&rep).unwrap();

            let mut assets = world.resource_mut::<Assets<Mesh>>();
            let mesh = assets.get_mut(handle).unwrap();

            if value.len() % 2 != 0 {
                warn!("UVs do not have an even length! Got: {}", value.len());
            } else {
                let value = value.chunks(2).map(|x| [x[0], x[1]]).collect::<Vec<_>>();

                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_UV_0,
                    VertexAttributeValues::Float32x2(value),
                );
            }
        });
        Ok(())
    }

    fn drop(&mut self, _rep: Resource<PrimitiveRes>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}

impl Host for ScriptData {}

#[cfg(test)]
mod tests {
    use crate::api::utils::tests::init_test_data;

    use super::*;

    #[test]
    fn test_new() {
        let (mut world, mut data) = init_test_data();

        let _ = HostMesh::new(&mut data).unwrap();

        world.commands().append(&mut data.commands);
        world.flush_commands();
    }
}
