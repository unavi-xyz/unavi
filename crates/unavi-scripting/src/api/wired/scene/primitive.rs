use std::sync::{Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};

use bevy::{
    prelude::*,
    render::mesh::{Indices, VertexAttributeValues},
};
use wasm_bridge::component::Resource;

use crate::{
    api::{
        id::UniqueId,
        wired::scene::bindings::mesh::{HostPrimitive, Material},
    },
    data::ScriptData,
};

use super::{material::MaterialRes, mesh::MeshId, nodes::base::NodeData};

#[derive(Default, Debug, Clone)]
pub struct PrimitiveRes(Arc<RwLock<PrimitiveData>>);

#[derive(Debug, Default)]
pub struct PrimitiveData {
    pub id: UniqueId,
    pub handle: OnceLock<Handle<Mesh>>,
    pub material: Option<MaterialRes>,
    /// Contains all nodes using this primitive.
    pub node_primitives: Vec<NodePrimitive>,
}

#[derive(Debug)]
pub struct NodePrimitive {
    /// The entity spawned under the node, holding the primitive mesh data.
    pub entity: Entity,
    pub node: Weak<RwLock<NodeData>>,
}

impl PrimitiveRes {
    pub fn read(&self) -> RwLockReadGuard<PrimitiveData> {
        self.0.read().unwrap()
    }
    pub fn write(&self) -> RwLockWriteGuard<PrimitiveData> {
        self.0.write().unwrap()
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct PrimitiveId(pub u32);

#[derive(Bundle)]
pub struct GltfPrimitiveBundle {
    pub id: PrimitiveId,
    pub handle: Handle<Mesh>,
    pub mesh: MeshId,
}

impl HostPrimitive for ScriptData {
    fn id(&mut self, self_: Resource<PrimitiveRes>) -> wasm_bridge::Result<u32> {
        let data = self.table.get(&self_)?.read();
        Ok(data.id.into())
    }

    fn material(
        &mut self,
        self_: Resource<PrimitiveRes>,
    ) -> wasm_bridge::Result<Option<Resource<Material>>> {
        let material = self.table.get(&self_)?.read().material.clone();
        let res = match material.clone() {
            Some(m) => Some(self.table.push(m)?),
            None => None,
        };
        Ok(res)
    }
    fn set_material(
        &mut self,
        self_: Resource<PrimitiveRes>,
        value: Option<Resource<Material>>,
    ) -> wasm_bridge::Result<()> {
        let material = match &value {
            Some(m) => {
                let data = self.table.get(m)?;
                Some(data.clone())
            }
            None => None,
        };

        let primitive = self.table.get(&self_)?.clone();
        primitive.write().material = material.clone();

        let default_material = self
            .api
            .wired_scene
            .as_ref()
            .unwrap()
            .default_material
            .clone();

        self.command_send
            .try_send(Box::new(move |world: &mut World| {
                let handle = material
                    .map(|m| m.read().handle.get().unwrap().clone())
                    .unwrap_or_else(|| default_material);

                for np in primitive.read().node_primitives.iter() {
                    world.entity_mut(np.entity).insert(handle.clone());
                }
            }))
            .unwrap();

        Ok(())
    }

    fn set_indices(
        &mut self,
        self_: Resource<PrimitiveRes>,
        value: Vec<u32>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get(&self_)?.clone();

        self.command_send
            .try_send(Box::new(move |world: &mut World| {
                let mut assets = world.resource_mut::<Assets<Mesh>>();
                let mesh = assets.get_mut(data.read().handle.get().unwrap()).unwrap();
                mesh.insert_indices(Indices::U32(value));
            }))
            .unwrap();

        Ok(())
    }
    fn set_positions(
        &mut self,
        self_: Resource<PrimitiveRes>,
        value: Vec<f32>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get(&self_)?.clone();

        self.command_send
            .try_send(Box::new(move |world: &mut World| {
                let mut assets = world.resource_mut::<Assets<Mesh>>();
                let mesh = assets.get_mut(data.read().handle.get().unwrap()).unwrap();

                let value = value.chunks(3).map(|x| [x[0], x[1], x[2]]).collect();

                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    VertexAttributeValues::Float32x3(value),
                );
            }))
            .unwrap();

        Ok(())
    }
    fn set_normals(
        &mut self,
        self_: Resource<PrimitiveRes>,
        value: Vec<f32>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get(&self_)?.clone();

        self.command_send
            .try_send(Box::new(move |world: &mut World| {
                let mut assets = world.resource_mut::<Assets<Mesh>>();
                let mesh = assets.get_mut(data.read().handle.get().unwrap()).unwrap();

                let value = value.chunks(3).map(|x| [x[0], x[1], x[2]]).collect();

                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_NORMAL,
                    VertexAttributeValues::Float32x3(value),
                );
            }))
            .unwrap();

        Ok(())
    }
    fn set_uvs(
        &mut self,
        self_: Resource<PrimitiveRes>,
        value: Vec<f32>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get(&self_)?.clone();

        self.command_send
            .try_send(Box::new(move |world: &mut World| {
                let mut assets = world.resource_mut::<Assets<Mesh>>();
                let mesh = assets.get_mut(data.read().handle.get().unwrap()).unwrap();

                if value.len() % 2 != 0 {
                    warn!("UVs do not have an even length! Got: {}", value.len());
                } else {
                    let value = value.chunks(2).map(|x| [x[0], x[1]]).collect::<Vec<_>>();

                    mesh.insert_attribute(
                        Mesh::ATTRIBUTE_UV_0,
                        VertexAttributeValues::Float32x2(value),
                    );
                }
            }))
            .unwrap();

        Ok(())
    }

    fn drop(&mut self, rep: Resource<PrimitiveRes>) -> wasm_bridge::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::api::{
        tests::init_test_data,
        wired::scene::{
            bindings::{material::HostMaterial, mesh::HostMesh, node::HostNode},
            nodes::base::NodeRef,
        },
    };

    use super::*;

    #[test]
    fn test_cleanup_resource() {
        let (_, mut data) = init_test_data();

        let mesh = HostMesh::new(&mut data).unwrap();
        let res = HostMesh::create_primitive(&mut data, Resource::new_own(mesh.rep())).unwrap();
        let res_weak = Resource::<PrimitiveRes>::new_own(res.rep());
        assert!(data.table.get(&res_weak).is_ok());

        HostMesh::remove_primitive(&mut data, mesh, Resource::new_own(res.rep())).unwrap();
        assert!(data.table.get(&res_weak).is_ok());

        HostPrimitive::drop(&mut data, res).unwrap();
        assert!(data.table.get(&res_weak).is_err());
    }

    #[test]
    fn test_set_material() {
        let (mut app, mut data) = init_test_data();
        let default_material = data
            .api
            .wired_scene
            .as_ref()
            .unwrap()
            .default_material
            .clone();

        let material = HostMaterial::new(&mut data).unwrap();
        let mesh = HostMesh::new(&mut data).unwrap();
        let primitive =
            HostMesh::create_primitive(&mut data, Resource::new_own(mesh.rep())).unwrap();

        let node = HostNode::new(&mut data).unwrap();
        HostNode::set_mesh(&mut data, node, Some(mesh)).unwrap();

        let world = app.world_mut();
        data.push_commands(&mut world.commands());
        world.flush_commands();

        let entity = data.table.get(&primitive).unwrap().read().node_primitives[0].entity;

        // Set new material.
        HostPrimitive::set_material(
            &mut data,
            Resource::new_own(primitive.rep()),
            Some(Resource::new_own(material.rep())),
        )
        .unwrap();

        let world = app.world_mut();
        data.push_commands(&mut world.commands());
        world.flush_commands();

        let material_handle = data
            .table
            .get(&material)
            .unwrap()
            .read()
            .handle
            .get()
            .unwrap()
            .clone();
        assert_eq!(
            *app.world().get::<Handle<StandardMaterial>>(entity).unwrap(),
            material_handle
        );

        // Remove material.
        HostPrimitive::set_material(&mut data, Resource::new_own(primitive.rep()), None).unwrap();

        let world = app.world_mut();
        data.push_commands(&mut world.commands());
        world.flush_commands();

        assert_eq!(
            *app.world().get::<Handle<StandardMaterial>>(entity).unwrap(),
            default_material
        );
    }

    #[test]
    fn test_set_positions() {
        let (mut app, mut data) = init_test_data();

        let mesh = HostMesh::new(&mut data).unwrap();
        let primitive =
            HostMesh::create_primitive(&mut data, Resource::new_own(mesh.rep())).unwrap();

        let node = HostNode::new(&mut data).unwrap();
        HostNode::set_mesh(&mut data, node, Some(mesh)).unwrap();

        let positions = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6];
        HostPrimitive::set_positions(&mut data, primitive, positions.clone()).unwrap();

        let world = app.world_mut();
        data.push_commands(&mut world.commands());
        world.flush_commands();

        let world = app.world_mut();

        let node_children = world
            .query_filtered::<&Children, With<NodeRef>>()
            .single(world);
        let primitive_ent = node_children[0];

        let mesh_handle = world.get::<Handle<Mesh>>(primitive_ent).unwrap();

        let assets = world.resource::<Assets<Mesh>>();
        let mesh = assets.get(mesh_handle).unwrap();

        let value = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap()
            .to_vec()
            .into_flattened();
        assert_eq!(value, positions);
    }
}
