use bevy::{prelude::*, utils::HashMap};
use wasm_bridge::component::Resource;

use crate::{
    api::{
        utils::RefResource,
        wired::{
            input::input_handler::{InputHandler, InputHandlerSender},
            math::bindings::types::Transform,
            physics::bindings::types::{Collider, RigidBody},
            scene::{
                bindings::node::{Host, HostNode},
                mesh::MeshRes,
                MaterialState,
            },
        },
    },
    data::ScriptData,
};

use super::base::NodeRes;

#[derive(Component, Default)]
pub struct NodeMesh {
    pub id: u32,
    pub node_primitives: HashMap<u32, Entity>,
}

impl HostNode for ScriptData {
    fn new(&mut self) -> wasm_bridge::Result<Resource<NodeRes>> {
        NodeRes::new_res(self)
    }

    fn id(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<u32> {
        Ok(self_.rep())
    }
    fn ref_(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<Resource<NodeRes>> {
        let value = NodeRes::from_res(&self_, &self.table)?;
        Ok(value)
    }

    fn name(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<String> {
        let data = self.table.get(&self_)?;
        Ok(data.name.clone())
    }
    fn set_name(&mut self, self_: Resource<NodeRes>, value: String) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.name = value;
        Ok(())
    }

    fn mesh(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<Option<Resource<MeshRes>>> {
        let node = self.table.get(&self_)?;
        let mesh = match &node.mesh {
            Some(m) => Some(self.clone_res(m)?),
            None => None,
        };
        Ok(mesh)
    }
    fn set_mesh(
        &mut self,
        self_: Resource<NodeRes>,
        value: Option<Resource<MeshRes>>,
    ) -> wasm_bridge::Result<()> {
        let rep = self_.rep();

        let node = self.table.get(&self_)?;
        let prev_mesh = &node.mesh;
        if let Some(prev_mesh) = prev_mesh {
            let prev_mesh = self.clone_res(prev_mesh)?;
            let mesh = self.table.get_mut(&prev_mesh)?;
            mesh.nodes
                .iter()
                .position(|r| r.rep() == rep)
                .map(|index| mesh.nodes.remove(index));
        }

        let mut primitive_ids = Vec::new();
        if let Some(mesh_res) = &value {
            let res = self.clone_res(&self_)?;
            let mesh = self.table.get_mut(mesh_res)?;
            mesh.nodes.push(res);

            let mesh = self.table.get(mesh_res)?;
            for p in mesh.primitives.iter() {
                let p_data = self.table.get(p).unwrap();
                let material_id = p_data.material.as_ref().map(|r| r.rep());
                primitive_ids.push((p.rep(), material_id));
            }
        }

        let res = value.and_then(|v| self.clone_res(&v).ok());
        let node = self.table.get_mut(&self_)?;
        node.mesh = res;

        let default_material = self
            .api
            .wired_scene
            .as_ref()
            .unwrap()
            .default_material
            .clone();
        let materials = self
            .api
            .wired_scene
            .as_ref()
            .unwrap()
            .entities
            .materials
            .clone();
        let mesh_rep = node.mesh.as_ref().map(|res| res.rep());
        let nodes = self
            .api
            .wired_scene
            .as_ref()
            .unwrap()
            .entities
            .nodes
            .clone();
        let primitives = self
            .api
            .wired_scene
            .as_ref()
            .unwrap()
            .entities
            .primitives
            .clone();

        self.commands.push(move |world: &mut World| {
            let nodes = nodes.read().unwrap();
            let node_ent = nodes.get(&rep).unwrap();

            // Remove previous node primitives.
            let mut to_remove = Vec::new();
            if let Some(node_mesh) = world.get::<NodeMesh>(*node_ent) {
                for e in node_mesh.node_primitives.values() {
                    to_remove.push(*e);
                }
            }
            for e in to_remove {
                world.despawn(e);
            }

            let mut node_primitives = HashMap::new();

            // Add new node primitives.
            let materials = materials.read().unwrap();
            let primitives = primitives.read().unwrap();
            for (primitive_id, material_id) in primitive_ids {
                let handle = primitives.get(&primitive_id).unwrap();

                let p_ent = world
                    .spawn(PbrBundle {
                        mesh: handle.clone(),
                        ..default()
                    })
                    .set_parent(*node_ent)
                    .id();
                node_primitives.insert(primitive_id, p_ent);

                if let Some(material_id) = material_id {
                    let MaterialState { handle, .. } = materials.get(&material_id).unwrap();
                    world.entity_mut(p_ent).insert(handle.clone());
                } else {
                    world.entity_mut(p_ent).insert(default_material.clone());
                }
            }

            let mut node_ent = world.entity_mut(*node_ent);

            // Add new mesh.
            if let Some(id) = mesh_rep {
                node_ent.insert(NodeMesh {
                    id,
                    node_primitives,
                });
            } else {
                node_ent.remove::<NodeMesh>();
            }
        });

        Ok(())
    }

    fn global_transform(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<Transform> {
        NodeRes::global_transform(self, &self_)
    }
    fn transform(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<Transform> {
        let node = self.table.get(&self_)?;
        Ok(node.transform.into())
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
        NodeRes::parent(self, &self_)
    }
    fn children(
        &mut self,
        self_: Resource<NodeRes>,
    ) -> wasm_bridge::Result<Vec<Resource<NodeRes>>> {
        NodeRes::children(self, &self_)
    }
    fn add_child(
        &mut self,
        self_: Resource<NodeRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        NodeRes::add_child(self, &self_, &value)
    }
    fn remove_child(
        &mut self,
        self_: Resource<NodeRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        NodeRes::remove_child(self, &self_, &value)
    }

    fn collider(
        &mut self,
        self_: Resource<NodeRes>,
    ) -> wasm_bridge::Result<Option<Resource<Collider>>> {
        let node = self.table.get(&self_)?;
        let res = match &node.collider {
            Some(res) => Some(Collider::from_res(res, &self.table)?),
            None => None,
        };
        Ok(res)
    }
    fn set_collider(
        &mut self,
        self_: Resource<NodeRes>,
        value: Option<Resource<Collider>>,
    ) -> wasm_bridge::Result<()> {
        let value = if let Some(v) = value {
            Some(Collider::from_res(&v, &self.table)?)
        } else {
            None
        };

        let collider = match &value {
            Some(value) => {
                let collider = self.table.get(value)?;
                Some(collider.component())
            }
            None => None,
        };

        let node = self.table.get_mut(&self_)?;
        node.collider = value;

        self.node_insert_option(self_.rep(), collider);

        Ok(())
    }

    fn rigid_body(
        &mut self,
        self_: Resource<NodeRes>,
    ) -> wasm_bridge::Result<Option<Resource<RigidBody>>> {
        let node = self.table.get(&self_)?;
        let res = match &node.rigid_body {
            Some(res) => Some(RigidBody::from_res(res, &self.table)?),
            None => None,
        };
        Ok(res)
    }
    fn set_rigid_body(
        &mut self,
        self_: Resource<NodeRes>,
        value: Option<Resource<RigidBody>>,
    ) -> wasm_bridge::Result<()> {
        let rigid_body = match &value {
            Some(value) => {
                let rigid_body = self.table.get(value)?;
                Some(rigid_body.component())
            }
            None => None,
        };

        let node = self.table.get_mut(&self_)?;
        node.rigid_body = value;

        self.node_insert_option(self_.rep(), rigid_body);

        Ok(())
    }

    fn input_handler(
        &mut self,
        self_: Resource<NodeRes>,
    ) -> wasm_bridge::Result<Option<Resource<InputHandler>>> {
        let node = self.table.get(&self_)?;

        let res = if let Some(r) = &node.input_handler {
            let res = InputHandler::from_res(r, &self.table)?;
            Some(res)
        } else {
            None
        };

        Ok(res)
    }
    fn set_input_handler(
        &mut self,
        self_: Resource<NodeRes>,
        value: Option<Resource<InputHandler>>,
    ) -> wasm_bridge::Result<()> {
        let handler = if let Some(value) = &value {
            let data = self.table.get(value)?;
            Some(InputHandlerSender(data.sender.clone()))
        } else {
            None
        };

        let node = self.table.get_mut(&self_)?;
        node.input_handler = value;

        self.node_insert_option(self_.rep(), handler);

        Ok(())
    }

    fn drop(&mut self, rep: Resource<NodeRes>) -> wasm_bridge::Result<()> {
        let id = rep.rep();
        let dropped = NodeRes::handle_drop(rep, &mut self.table)?;

        if dropped {
            let nodes = self
                .api
                .wired_scene
                .as_ref()
                .unwrap()
                .entities
                .nodes
                .clone();

            self.commands.push(move |world: &mut World| {
                let mut nodes = nodes.write().unwrap();
                let entity = nodes.remove(&id).unwrap();
                world.despawn(entity);
            });
        }

        Ok(())
    }
}

impl Host for ScriptData {}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use crate::api::{
        utils::tests::init_test_data,
        wired::scene::{bindings::mesh::HostMesh, nodes::base::NodeId},
    };

    use super::*;

    #[test]
    #[traced_test]
    fn test_new() {
        let (mut app, mut data) = init_test_data();
        let world = app.world_mut();

        let res = HostNode::new(&mut data).unwrap();

        world.commands().append(&mut data.commands);
        world.flush_commands();

        world
            .query::<&NodeId>()
            .iter(world)
            .find(|n| n.0 == res.rep())
            .unwrap();
    }

    #[test]
    #[traced_test]
    fn test_children() {
        let (_, mut data) = init_test_data();

        let node_1 = HostNode::new(&mut data).unwrap();
        let node_2 = HostNode::new(&mut data).unwrap();
        assert!(data
            .children(data.clone_res(&node_1).unwrap())
            .unwrap()
            .is_empty());
        assert!(data
            .parent(data.clone_res(&node_2).unwrap())
            .unwrap()
            .is_none());

        // Add child.
        data.add_child(
            data.clone_res(&node_1).unwrap(),
            data.clone_res(&node_2).unwrap(),
        )
        .unwrap();
        let children = data.children(data.clone_res(&node_1).unwrap()).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].rep(), node_2.rep());
        assert_eq!(
            data.parent(data.clone_res(&node_2).unwrap())
                .unwrap()
                .unwrap()
                .rep(),
            node_1.rep()
        );

        // Remove child.
        data.remove_child(
            data.clone_res(&node_1).unwrap(),
            data.clone_res(&node_2).unwrap(),
        )
        .unwrap();
        assert!(data
            .children(data.clone_res(&node_1).unwrap())
            .unwrap()
            .is_empty());
        assert!(data.parent(node_2).unwrap().is_none());
    }

    #[test]
    #[traced_test]
    fn test_set_mesh() {
        let (mut app, mut data) = init_test_data();
        let world = app.world_mut();

        // Set mesh.
        let node = HostNode::new(&mut data).unwrap();
        let mesh = HostMesh::new(&mut data).unwrap();
        let _ = HostMesh::create_primitive(&mut data, Resource::new_own(mesh.rep())).unwrap();
        HostNode::set_mesh(&mut data, Resource::new_own(node.rep()), Some(mesh)).unwrap();

        world.commands().append(&mut data.commands);
        world.flush_commands();

        let (node_mesh, node_children) = world.query::<(&NodeMesh, &Children)>().single(world);
        assert_eq!(node_mesh.node_primitives.len(), 1);

        let primitive_ent = node_mesh.node_primitives.values().next().unwrap();
        let _ = world.get::<Handle<Mesh>>(*primitive_ent).unwrap();
        let _ = world.get::<GlobalTransform>(*primitive_ent).unwrap();
        assert!(node_children.contains(primitive_ent));

        // Remove mesh.
        HostNode::set_mesh(&mut data, node, None).unwrap();

        world.commands().append(&mut data.commands);
        world.flush_commands();

        let node_mesh = world.query::<&NodeMesh>().get_single(world);
        assert!(node_mesh.is_err());

        assert!(!logs_contain("ERROR"));
        assert!(!logs_contain("error"));
        assert!(!logs_contain("WARN"));
        assert!(!logs_contain("warn"));
    }
}
