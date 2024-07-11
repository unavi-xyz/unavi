use std::cell::Cell;

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
        render_asset::RenderAssetUsages,
    },
    utils::HashSet,
};
use wasm_bridge::component::Resource;

use crate::{
    api::utils::{RefCount, RefCountCell, RefResource},
    state::{MaterialState, PrimitiveState, StoreState},
};

use crate::api::wired_scene::wired::scene::mesh::{Host, HostMesh, HostPrimitive, Material};

use super::node::NodeMesh;

#[derive(Component, Clone, Copy, Debug)]
pub struct MeshId(pub u32);

#[derive(Bundle)]
pub struct WiredMeshBundle {
    pub id: MeshId,
}

impl WiredMeshBundle {
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
    pub nodes: HashSet<u32>,
    pub primitives: HashSet<u32>,
    ref_count: RefCountCell,
}

impl RefCount for MeshRes {
    fn ref_count(&self) -> &Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for MeshRes {}

#[derive(Default, Debug)]
pub struct PrimitiveRes {
    pub material: Option<u32>,
}

impl HostMesh for StoreState {
    fn new(&mut self) -> wasm_bridge::Result<Resource<MeshRes>> {
        let table_res = self.table.push(MeshRes::default())?;
        let res = MeshRes::from_res(&table_res, &self.table)?;
        Ok(res)
    }

    fn id(&mut self, self_: Resource<MeshRes>) -> wasm_bridge::Result<u32> {
        Ok(self_.rep())
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
        let mesh = self.table.get_mut(&self_)?;
        Ok(mesh
            .primitives
            .iter()
            .map(|rep| Resource::new_own(*rep))
            .collect())
    }
    fn create_primitive(
        &mut self,
        self_: Resource<MeshRes>,
    ) -> wasm_bridge::Result<Resource<PrimitiveRes>> {
        let resource = self.table.push(PrimitiveRes::default())?;
        let primitive_rep = resource.rep();

        let mesh = self.table.get_mut(&self_)?;
        mesh.primitives.insert(primitive_rep);

        let mesh_nodes = mesh.nodes.clone();
        let mesh_rep = self_.rep();
        let nodes = self.entities.nodes.clone();
        let primitives = self.entities.primitives.clone();
        self.commands.push(move |world: &mut World| {
            let mut assets = world.resource_mut::<Assets<Mesh>>();
            let handle = assets.add(Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::all(),
            ));

            let entity = world
                .spawn(GltfPrimitiveBundle {
                    id: PrimitiveId(primitive_rep),
                    mesh: MeshId(mesh_rep),
                    handle: handle.clone(),
                })
                .id();

            let mut primitives = primitives.write().unwrap();
            primitives.insert(
                primitive_rep,
                PrimitiveState {
                    entity,
                    handle: handle.clone(),
                },
            );

            // Create node primitives.
            let nodes = nodes.read().unwrap();
            for node_id in mesh_nodes {
                let entity = nodes.get(&node_id).unwrap();

                let mut p_ent = None;

                world.entity_mut(*entity).with_children(|world| {
                    p_ent = Some(world.spawn((SpatialBundle::default(), handle.clone())).id());
                });

                let mut node_mesh = world.get_mut::<NodeMesh>(*entity).unwrap();
                node_mesh
                    .node_primitives
                    .insert(primitive_rep, p_ent.take().unwrap());
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
        mesh.primitives.remove(&rep);

        let node_ids = mesh.nodes.clone();
        let nodes = self.entities.nodes.clone();
        let primitives = self.entities.primitives.clone();
        self.commands.push(move |world: &mut World| {
            let mut primitives = primitives.write().unwrap();
            let PrimitiveState { entity, .. } = primitives.get_mut(&rep).unwrap();
            world.despawn(*entity);

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

impl HostPrimitive for StoreState {
    fn id(&mut self, self_: Resource<PrimitiveRes>) -> wasm_bridge::Result<u32> {
        Ok(self_.rep())
    }

    fn material(
        &mut self,
        self_: Resource<PrimitiveRes>,
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
        self_: Resource<PrimitiveRes>,
        value: Option<Resource<Material>>,
    ) -> wasm_bridge::Result<()> {
        let primitive = self.table.get_mut(&self_)?;
        primitive.material = value.map(|v| v.rep());

        let material_rep = primitive.material;
        let materials = self.entities.materials.clone();
        let primitives = self.entities.primitives.clone();
        let rep = self_.rep();
        self.commands.push(move |world: &mut World| {
            let primitives = primitives.read().unwrap();
            let PrimitiveState { entity, .. } = primitives.get(&rep).unwrap();
            let mut entity = world.entity_mut(*entity);

            if let Some(material_rep) = material_rep {
                let materials = materials.read().unwrap();
                let MaterialState { handle, .. } = materials.get(&material_rep).unwrap();
                entity.insert(handle.clone());
            } else {
                // TODO: add default material
                entity.remove::<Handle<StandardMaterial>>();
            }
        });

        Ok(())
    }

    fn set_indices(
        &mut self,
        self_: Resource<PrimitiveRes>,
        value: Vec<u32>,
    ) -> wasm_bridge::Result<()> {
        let primitives = self.entities.primitives.clone();
        let rep = self_.rep();
        self.commands.push(move |world: &mut World| {
            let primitives = primitives.read().unwrap();
            let PrimitiveState { handle, .. } = primitives.get(&rep).unwrap();
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
        let primitives = self.entities.primitives.clone();
        let rep = self_.rep();
        self.commands.push(move |world: &mut World| {
            let primitives = primitives.read().unwrap();
            let PrimitiveState { handle, .. } = primitives.get(&rep).unwrap();
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
        let primitives = self.entities.primitives.clone();
        let rep = self_.rep();
        self.commands.push(move |world: &mut World| {
            let primitives = primitives.read().unwrap();
            let PrimitiveState { handle, .. } = primitives.get(&rep).unwrap();
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
        let primitives = self.entities.primitives.clone();
        let rep = self_.rep();
        self.commands.push(move |world: &mut World| {
            let primitives = primitives.read().unwrap();
            let PrimitiveState { handle, .. } = primitives.get(&rep).unwrap();
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

impl Host for StoreState {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let mut world = World::new();
        let root_ent = world.spawn_empty().id();
        let mut state = StoreState::new("test".to_string(), root_ent);

        let _ = HostMesh::new(&mut state).unwrap();

        world.commands().append(&mut state.commands);
        world.flush_commands();
    }
}
