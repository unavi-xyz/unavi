use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};

use bevy::{
    prelude::*,
    render::{mesh::PrimitiveTopology, render_asset::RenderAssetUsages},
};
use wasm_bridge::component::Resource;

use crate::{
    api::{
        id::ResourceId,
        wired::scene::bindings::mesh::{Host, HostMesh, HostPrimitive},
    },
    data::ScriptData,
};

use super::{
    nodes::base::{NodeData, NodeRes},
    primitive::{NodePrimitive, PrimitiveData, PrimitiveRes},
};

#[derive(Component, Clone, Copy, Debug)]
pub struct MeshId(pub u32);

#[derive(Default, Debug, Clone)]
pub struct MeshRes(Arc<RwLock<MeshData>>);

#[derive(Default, Debug)]
pub struct MeshData {
    pub id: ResourceId,
    pub name: String,
    /// Nodes that are using this mesh.
    pub nodes: Vec<Weak<RwLock<NodeData>>>,
    pub primitives: Vec<PrimitiveRes>,
}

impl MeshRes {
    pub fn read(&self) -> RwLockReadGuard<MeshData> {
        self.0.read().unwrap()
    }
    pub fn write(&self) -> RwLockWriteGuard<MeshData> {
        self.0.write().unwrap()
    }
}

impl HostMesh for ScriptData {
    fn new(&mut self) -> wasm_bridge::Result<Resource<MeshRes>> {
        let res = self.table.push(MeshRes::default())?;
        Ok(res)
    }

    fn id(&mut self, self_: Resource<MeshRes>) -> wasm_bridge::Result<u32> {
        let data = self.table.get(&self_)?;
        Ok(data.read().id.into())
    }
    fn ref_(&mut self, self_: Resource<MeshRes>) -> wasm_bridge::Result<Resource<MeshRes>> {
        let data = self.table.get(&self_)?;
        let res = self.table.push(data.clone())?;
        Ok(res)
    }

    fn name(&mut self, self_: Resource<MeshRes>) -> wasm_bridge::Result<String> {
        let data = self.table.get(&self_)?.read();
        Ok(data.name.clone())
    }
    fn set_name(&mut self, self_: Resource<MeshRes>, value: String) -> wasm_bridge::Result<()> {
        let mut data = self.table.get(&self_)?.write();
        data.name = value;
        Ok(())
    }

    fn list_primitives(
        &mut self,
        self_: Resource<MeshRes>,
    ) -> wasm_bridge::Result<Vec<Resource<PrimitiveRes>>> {
        let primitives = self.table.get(&self_)?.read().primitives.clone();
        let list = primitives
            .into_iter()
            .map(|data| self.table.push(data))
            .collect::<Result<_, _>>()?;
        Ok(list)
    }
    fn create_primitive(
        &mut self,
        self_: Resource<MeshRes>,
    ) -> wasm_bridge::Result<Resource<PrimitiveRes>> {
        let data = PrimitiveRes::default();
        let res = self.table.push(data.clone())?;

        let mesh = self.table.get_mut(&self_)?.clone();
        mesh.write().primitives.push(data.clone());

        let default_material = self
            .api
            .wired_scene
            .as_ref()
            .unwrap()
            .default_material
            .clone();

        self.command_send
            .try_send(Box::new(move |world: &mut World| {
                let mut assets = world.resource_mut::<Assets<Mesh>>();
                let handle = assets.add(Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::all(),
                ));

                let mut data_write = data.write();
                data_write.handle.get_or_init(|| handle);

                // Create node primitives.
                for node in mesh.read().nodes.iter().map(|n| n.upgrade().unwrap()) {
                    try_create_primitive(world, &node, &mut data_write, &default_material);
                }
            }))
            .unwrap();

        Ok(res)
    }
    fn remove_primitive(
        &mut self,
        self_: Resource<MeshRes>,
        value: Resource<PrimitiveRes>,
    ) -> wasm_bridge::Result<()> {
        let primitive = self.table.get(&value)?.clone();
        let id = primitive.read().id;

        let mut data = self.table.get(&self_)?.write();
        data.primitives
            .iter()
            .position(|p| p.read().id == id)
            .map(|index| data.primitives.remove(index));

        self.command_send
            .try_send(Box::new(move |world: &mut World| {
                // Remove node primitives.
                for node_primitive in primitive.read().node_primitives.iter() {
                    world.entity_mut(node_primitive.entity).despawn();
                }
            }))
            .unwrap();

        Ok(())
    }

    fn drop(&mut self, rep: Resource<MeshRes>) -> wasm_bridge::Result<()> {
        self.table.delete(rep)?;

        Ok(())
    }
}

impl Host for ScriptData {}

/// Tries to create a node primitive.
/// If the node does not have an entity yet, nothing will happen.
pub fn try_create_primitive(
    world: &mut World,
    node: &Arc<RwLock<NodeData>>,
    primitive: &mut PrimitiveData,
    default_material: &Handle<StandardMaterial>,
) {
    if let Some(node_ent) = node.read().unwrap().entity.get() {
        let material = primitive
            .material
            .as_ref()
            .map(|m| m.read().handle.get().unwrap().clone())
            .unwrap_or_else(|| default_material.clone());

        let entity = world
            .spawn(PbrBundle {
                material,
                ..default()
            })
            .set_parent(*node_ent)
            .id();

        primitive.node_primitives.push(NodePrimitive {
            entity,
            node: Arc::downgrade(node),
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::api::{tests::init_test_data, wired::scene::bindings::node::HostNode};

    use super::*;

    #[test]
    fn test_cleanup_resource() {
        let (_, mut data) = init_test_data();

        let res = HostMesh::new(&mut data).unwrap();
        let res_weak = Resource::<MeshRes>::new_own(res.rep());
        assert!(data.table.get(&res_weak).is_ok());

        HostMesh::drop(&mut data, res).unwrap();
        assert!(data.table.get(&res_weak).is_err());
    }

    fn test_create_node_primitives(
        mut app: App,
        data: &mut ScriptData,
        node: Resource<NodeRes>,
        primitive: Resource<PrimitiveRes>,
    ) {
        let world = app.world_mut();
        data.push_commands(&mut world.commands());
        world.flush_commands();

        let inner = data.table.get(&primitive).unwrap().read();
        assert_eq!(inner.node_primitives.len(), 1);

        let node_data = data.table.get(&node).unwrap().read();
        let node_id = node_data.id;
        let node_entity = *node_data.entity.get().unwrap();
        assert_eq!(
            inner.node_primitives[0]
                .node
                .upgrade()
                .unwrap()
                .read()
                .unwrap()
                .id,
            node_id
        );

        assert_eq!(
            world
                .get::<Parent>(inner.node_primitives[0].entity)
                .unwrap()
                .get(),
            node_entity
        );

        drop(inner);
        drop(node_data);

        HostNode::drop(data, node).unwrap();
        HostPrimitive::drop(data, primitive).unwrap();
    }

    #[test]
    fn test_set_mesh() {
        let (app, mut data) = init_test_data();

        let mesh = HostMesh::new(&mut data).unwrap();

        let primitive =
            HostMesh::create_primitive(&mut data, Resource::new_own(mesh.rep())).unwrap();

        let node = HostNode::new(&mut data).unwrap();
        HostNode::set_mesh(
            &mut data,
            Resource::new_own(node.rep()),
            Some(Resource::new_own(mesh.rep())),
        )
        .unwrap();

        test_create_node_primitives(app, &mut data, node, primitive);
    }

    #[test]
    fn test_create_primitive() {
        let (app, mut data) = init_test_data();

        let node = HostNode::new(&mut data).unwrap();

        let mesh = HostMesh::new(&mut data).unwrap();
        HostNode::set_mesh(
            &mut data,
            Resource::new_own(node.rep()),
            Some(Resource::new_own(mesh.rep())),
        )
        .unwrap();

        let primitive =
            HostMesh::create_primitive(&mut data, Resource::new_own(mesh.rep())).unwrap();

        test_create_node_primitives(app, &mut data, node, primitive);
    }

    #[test]
    fn test_remove_primitive() {
        // TODO
        // let (app, mut data) = init_test_data();
        //
        // let mesh = HostMesh::new(&mut data).unwrap();
        // let primitive =
        //     HostMesh::create_primitive(&mut data, Resource::new_own(mesh.rep())).unwrap();
        //
        // let node = HostNode::new(&mut data).unwrap();
        // HostNode::set_mesh(
        //     &mut data,
        //     Resource::new_own(node.rep()),
        //     Some(Resource::new_own(mesh.rep())),
        // )
        // .unwrap();
    }
}
