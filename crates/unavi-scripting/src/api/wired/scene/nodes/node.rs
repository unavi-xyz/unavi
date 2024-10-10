use bevy::prelude::*;
use wasm_bridge::component::Resource;

use crate::{
    api::wired::{
        input::input_handler::{InputHandlerRes, InputHandlerSender},
        math::bindings::types::Transform,
        physics::{collider::ColliderRes, rigid_body::RigidBodyRes},
        scene::{
            bindings::node::{Host, HostNode},
            mesh::{try_create_primitive, MeshRes},
        },
    },
    data::ScriptData,
};

use super::base::NodeRes;

impl HostNode for ScriptData {
    fn new(&mut self) -> wasm_bridge::Result<Resource<NodeRes>> {
        NodeRes::new_res(self)
    }

    fn id(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<u32> {
        let data = self.table.get(&self_)?.read();
        Ok(data.id.into())
    }
    fn ref_(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<Resource<NodeRes>> {
        let data = self.table.get(&self_)?;
        let res = self.table.push(data.clone())?;
        Ok(res)
    }

    fn name(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<String> {
        let data = self.table.get(&self_)?.read();
        Ok(data.name.clone())
    }
    fn set_name(&mut self, self_: Resource<NodeRes>, value: String) -> wasm_bridge::Result<()> {
        let mut data = self.table.get(&self_)?.write();
        data.name = value;
        Ok(())
    }

    fn mesh(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<Option<Resource<MeshRes>>> {
        let mesh = self.table.get(&self_)?.read().mesh.clone();
        let res = match mesh {
            Some(m) => Some(self.table.push(m)?),
            None => None,
        };
        Ok(res)
    }
    fn set_mesh(
        &mut self,
        self_: Resource<NodeRes>,
        value: Option<Resource<MeshRes>>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get(&self_)?.clone();

        // Remove previous mesh.
        if let Some(prev_mesh) = data.read().mesh.clone() {
            let mut prev_write = prev_mesh.write();
            prev_write
                .nodes
                .iter()
                .position(|n| n.upgrade().unwrap().read().unwrap().id == prev_write.id)
                .map(|index| prev_write.nodes.remove(index));
            drop(prev_write);

            let data = data.clone();
            self.command_send
                .try_send(Box::new(move |world: &mut World| {
                    let data_read = data.read();

                    for primitive in prev_mesh.read().primitives.iter() {
                        let mut primitive_data = primitive.write();

                        if let Some((i, e)) = primitive_data
                            .node_primitives
                            .iter()
                            .enumerate()
                            .find_map(|(i, p)| {
                                if let Some(n) = p.node.upgrade() {
                                    if n.read().unwrap().id == data_read.id {
                                        return Some((i, p.entity));
                                    }
                                }

                                None
                            })
                        {
                            primitive_data.node_primitives.remove(i);
                            world.entity_mut(e).despawn();
                        }
                    }
                }))
                .unwrap();
        }

        // Set new mesh.
        let mesh_data = match &value {
            Some(v) => Some(self.table.get(v)?.clone()),
            None => None,
        };

        let mut data_write = data.write();
        data_write.mesh = mesh_data.clone();
        drop(data_write);

        let default_material = self
            .api
            .wired_scene
            .as_ref()
            .unwrap()
            .default_material
            .clone();

        if let Some(mesh_data) = mesh_data {
            // Create node primitives.
            self.command_send
                .try_send(Box::new(move |world: &mut World| {
                    for primitive in mesh_data.read().primitives.iter() {
                        let mut primitive_write = primitive.write();
                        try_create_primitive(
                            world,
                            data.raw_data(),
                            &mut primitive_write,
                            &default_material,
                        );
                    }
                }))
                .unwrap();
        }

        Ok(())
    }

    fn global_transform(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<Transform> {
        NodeRes::global_transform(self, &self_)
    }
    fn transform(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<Transform> {
        let data = self.table.get(&self_)?.read();
        Ok(data.transform.into())
    }
    fn set_transform(
        &mut self,
        self_: Resource<NodeRes>,
        value: Transform,
    ) -> wasm_bridge::Result<()> {
        NodeRes::set_transform(self, &self_, value)
    }

    fn parent(
        &mut self,
        self_: Resource<NodeRes>,
    ) -> wasm_bridge::Result<Option<Resource<NodeRes>>> {
        NodeRes::parent(self, self.table.get(&self_)?.clone())
    }
    fn children(
        &mut self,
        self_: Resource<NodeRes>,
    ) -> wasm_bridge::Result<Vec<Resource<NodeRes>>> {
        NodeRes::children(self, self.table.get(&self_)?.clone())
    }
    fn add_child(
        &mut self,
        self_: Resource<NodeRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        NodeRes::add_child(
            self,
            self.table.get(&self_)?.clone(),
            self.table.get(&value)?.clone(),
        );
        Ok(())
    }
    fn remove_child(
        &mut self,
        self_: Resource<NodeRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        NodeRes::remove_child(
            self,
            self.table.get(&self_)?.clone(),
            self.table.get(&value)?.clone(),
        );
        Ok(())
    }

    fn collider(
        &mut self,
        self_: Resource<NodeRes>,
    ) -> wasm_bridge::Result<Option<Resource<ColliderRes>>> {
        let collider = self.table.get(&self_)?.read().collider.clone();
        let res = match collider {
            Some(c) => Some(self.table.push(c)?),
            None => None,
        };
        Ok(res)
    }
    fn set_collider(
        &mut self,
        self_: Resource<NodeRes>,
        value: Option<Resource<ColliderRes>>,
    ) -> wasm_bridge::Result<()> {
        let c_data = if let Some(v) = &value {
            Some(self.table.get(v)?.clone())
        } else {
            None
        };

        let component = c_data.as_ref().map(|d| d.read().component());

        let data = self.table.get(&self_)?;
        data.write().collider = c_data;

        self.node_insert_option(data.clone(), component);

        Ok(())
    }

    fn rigid_body(
        &mut self,
        self_: Resource<NodeRes>,
    ) -> wasm_bridge::Result<Option<Resource<RigidBodyRes>>> {
        let rigid_body = self.table.get(&self_)?.read().rigid_body.clone();
        let res = match rigid_body {
            Some(r) => Some(self.table.push(r)?),
            None => None,
        };
        Ok(res)
    }
    fn set_rigid_body(
        &mut self,
        self_: Resource<NodeRes>,
        value: Option<Resource<RigidBodyRes>>,
    ) -> wasm_bridge::Result<()> {
        let r_data = match &value {
            Some(value) => Some(self.table.get(value)?.clone()),
            None => None,
        };

        let component = r_data.as_ref().map(|d| d.read().component());

        let data = self.table.get(&self_)?;
        data.write().rigid_body = r_data;

        self.node_insert_option(data.clone(), component);

        Ok(())
    }

    fn input_handler(
        &mut self,
        self_: Resource<NodeRes>,
    ) -> wasm_bridge::Result<Option<Resource<InputHandlerRes>>> {
        let input_handler = self.table.get(&self_)?.read().input_handler.clone();

        let res = if let Some(r) = input_handler {
            Some(self.table.push(r)?)
        } else {
            None
        };

        Ok(res)
    }
    fn set_input_handler(
        &mut self,
        self_: Resource<NodeRes>,
        value: Option<Resource<InputHandlerRes>>,
    ) -> wasm_bridge::Result<()> {
        let handler = if let Some(value) = &value {
            Some(self.table.get(value)?.clone())
        } else {
            None
        };

        let component = handler
            .as_ref()
            .map(|r| InputHandlerSender(r.read().sender.clone()));

        let data = self.table.get(&self_)?;
        data.write().input_handler = handler;

        self.node_insert_option(data.clone(), component);

        Ok(())
    }

    fn drop(&mut self, rep: Resource<NodeRes>) -> wasm_bridge::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}

impl Host for ScriptData {}

#[cfg(test)]
mod tests {
    use crate::api::{tests::init_test_data, wired::scene::nodes::base::NodeId};

    use super::*;

    #[test]
    fn test_cleanup_resource() {
        let (_, mut data) = init_test_data();

        let res = HostNode::new(&mut data).unwrap();
        let res_weak = Resource::<NodeRes>::new_own(res.rep());

        HostNode::drop(&mut data, res).unwrap();
        assert!(data.table.get(&res_weak).is_err());
    }

    #[test]
    fn test_cleanup_entity() {
        let (mut app, mut data) = init_test_data();
        let world = app.world_mut();

        let res = HostNode::new(&mut data).unwrap();

        data.push_commands(&mut world.commands());
        world.flush_commands();

        // Clone the inner resource data.
        // The entity should not be despawned until this is also dropped.
        let inner_clone = data.table.get(&res).unwrap().clone();

        let entity = *inner_clone.read().entity.get().unwrap();
        assert!(world.get::<NodeId>(entity).is_some());

        HostNode::drop(&mut data, res).unwrap();

        assert!(world.get::<NodeId>(entity).is_some());

        drop(inner_clone);

        data.push_commands(&mut world.commands());
        world.flush_commands();
        assert!(world.get::<NodeId>(entity).is_none());
    }
}
