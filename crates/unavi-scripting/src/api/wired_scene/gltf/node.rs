use std::cell::Cell;

use bevy::{prelude::*, utils::HashMap};
use wasm_bridge::component::Resource;

use crate::{
    api::{
        utils::{RefCount, RefCountCell, RefResource},
        wired_input::{InputHandler, InputHandlerSender},
        wired_physics::wired::{
            math::types::Vec3,
            physics::types::{RigidBodyType, Shape, Sphere},
        },
    },
    state::{MaterialState, PrimitiveState, StoreState},
};

use crate::api::wired_scene::wired::{
    physics::types::{Collider, RigidBody},
    scene::node::{Host, HostNode, Transform},
};

use super::mesh::MeshRes;

#[derive(Component, Clone, Copy, Debug)]
pub struct NodeId(pub u32);

#[derive(Bundle)]
pub struct WiredNodeBundle {
    pub id: NodeId,
    pub spatial: SpatialBundle,
}

#[derive(Component, Default)]
pub struct NodeMesh {
    pub id: u32,
    pub node_primitives: HashMap<u32, Entity>,
}

impl WiredNodeBundle {
    pub fn new(id: u32) -> Self {
        Self {
            id: NodeId(id),
            spatial: SpatialBundle::default(),
        }
    }
}

#[derive(Default, Debug)]
pub struct NodeRes {
    pub children: Vec<Resource<NodeRes>>,
    pub collider: Option<Resource<Collider>>,
    pub input_handler: Option<Resource<InputHandler>>,
    pub mesh: Option<Resource<MeshRes>>,
    pub name: String,
    pub parent: Option<Resource<NodeRes>>,
    pub rigid_body: Option<Resource<RigidBody>>,
    pub transform: Transform,
    ref_count: RefCountCell,
}

impl RefCount for NodeRes {
    fn ref_count(&self) -> &Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for NodeRes {}

impl HostNode for StoreState {
    fn new(&mut self) -> wasm_bridge::Result<Resource<NodeRes>> {
        let node = NodeRes::default();
        let table_res = self.table.push(node)?;
        let res = self.clone_res(&table_res)?;
        let rep = res.rep();

        let nodes = self.entities.nodes.clone();
        self.commands.push(move |world: &mut World| {
            let entity = world.spawn(WiredNodeBundle::new(rep)).id();
            let mut nodes = nodes.write().unwrap();
            nodes.insert(rep, entity);
        });

        Ok(res)
    }

    fn id(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<u32> {
        Ok(self_.rep())
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

        let default_material = self.default_material.clone();
        let materials = self.entities.materials.clone();
        let mesh_rep = node.mesh.as_ref().map(|res| res.rep());
        let nodes = self.entities.nodes.clone();
        let primitives = self.entities.primitives.clone();
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
                let PrimitiveState { handle, .. } = primitives.get(&primitive_id).unwrap();

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

    fn parent(
        &mut self,
        self_: Resource<NodeRes>,
    ) -> wasm_bridge::Result<Option<Resource<NodeRes>>> {
        let data = self.table.get(&self_)?;
        let parent = match &data.parent {
            Some(r) => Some(self.clone_res(r)?),
            None => None,
        };
        Ok(parent)
    }
    fn children(
        &mut self,
        self_: Resource<NodeRes>,
    ) -> wasm_bridge::Result<Vec<Resource<NodeRes>>> {
        let node = self.table.get(&self_)?;
        let children = node
            .children
            .iter()
            .map(|res| self.clone_res(res))
            .collect::<Result<_, _>>()?;
        Ok(children)
    }
    fn add_child(
        &mut self,
        self_: Resource<NodeRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        let child_rep = value.rep();
        let parent_rep = self_.rep();

        // Add child to children.
        let res = self.clone_res(&value)?;
        let node = self.table.get_mut(&self_)?;
        node.children.push(res);

        // Remove child from old parent's children.
        let child = self.table.get(&value)?;
        if let Some(parent) = &child.parent {
            let parent_res = self.clone_res(parent)?;
            let child_res = self.clone_res(&value)?;
            self.remove_child(parent_res, child_res)?;
        }

        // Set parent.
        let parent_res = self.clone_res(&self_)?;
        let child = self.table.get_mut(&value)?;
        child.parent = Some(parent_res);

        // Update ECS.
        let nodes = self.entities.nodes.clone();
        self.commands.push(move |world: &mut World| {
            let nodes = nodes.read().unwrap();
            let child_ent = nodes.get(&child_rep).unwrap();
            let parent_ent = nodes.get(&parent_rep).unwrap();

            world.entity_mut(*parent_ent).push_children(&[*child_ent]);
        });

        Ok(())
    }
    fn remove_child(
        &mut self,
        self_: Resource<NodeRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        let child_rep = value.rep();

        let node = self.table.get_mut(&self_)?;
        node.children
            .iter()
            .position(|r| r.rep() == child_rep)
            .map(|index| node.children.remove(index));

        let nodes = self.entities.nodes.clone();
        self.commands.push(move |world: &mut World| {
            let nodes = nodes.read().unwrap();
            let child_ent = nodes.get(&child_rep).unwrap();
            world.entity_mut(*child_ent).remove_parent();
        });

        Ok(())
    }

    fn transform(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<Transform> {
        let node = self.table.get(&self_)?;
        Ok(node.transform)
    }
    fn set_transform(
        &mut self,
        self_: Resource<NodeRes>,
        value: Transform,
    ) -> wasm_bridge::Result<()> {
        let node = self.table.get_mut(&self_)?;
        node.transform = value;

        let transform = bevy::prelude::Transform {
            translation: bevy::prelude::Vec3::new(
                node.transform.translation.x,
                node.transform.translation.y,
                node.transform.translation.z,
            ),
            rotation: bevy::prelude::Quat::from_xyzw(
                node.transform.rotation.x,
                node.transform.rotation.y,
                node.transform.rotation.z,
                node.transform.rotation.w,
            ),
            scale: bevy::prelude::Vec3::new(
                node.transform.scale.x,
                node.transform.scale.y,
                node.transform.scale.z,
            ),
        };

        self.node_insert(self_.rep(), transform);

        Ok(())
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
        let collider = match &value {
            Some(value) => {
                let collider = self.table.get(value)?;
                let collider = match collider.shape {
                    Shape::Sphere(Sphere { radius }) => avian3d::prelude::Collider::sphere(radius),
                    Shape::Cuboid(Vec3 { x, y, z }) => avian3d::prelude::Collider::cuboid(x, y, z),
                };
                Some(collider)
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
                let rigid_body = match rigid_body.rigid_body_type {
                    RigidBodyType::Dynamic => avian3d::prelude::RigidBody::Dynamic,
                    RigidBodyType::Fixed => avian3d::prelude::RigidBody::Static,
                    RigidBodyType::Kinematic => avian3d::prelude::RigidBody::Kinematic,
                };
                Some(rigid_body)
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
            let nodes = self.entities.nodes.clone();
            self.commands.push(move |world: &mut World| {
                let mut nodes = nodes.write().unwrap();
                let entity = nodes.remove(&id).unwrap();
                world.despawn(entity);
            });
        }

        Ok(())
    }
}

impl Host for StoreState {}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use crate::api::{utils::tests::init_test_state, wired_scene::wired::scene::mesh::HostMesh};

    use super::*;

    #[test]
    #[traced_test]
    fn test_new() {
        let (mut world, mut state) = init_test_state();

        let _ = HostNode::new(&mut state).unwrap();

        world.commands().append(&mut state.commands);
        world.flush_commands();

        let (found_id, _) = world
            .query::<(&NodeId, &bevy::prelude::Transform)>()
            .single(&world);
        assert_eq!(found_id.0, 1);
    }

    #[test]
    #[traced_test]
    fn test_set_mesh() {
        let (mut world, mut state) = init_test_state();

        // Set mesh.
        let node = HostNode::new(&mut state).unwrap();
        let mesh = HostMesh::new(&mut state).unwrap();
        let _ = HostMesh::create_primitive(&mut state, Resource::new_own(mesh.rep())).unwrap();
        HostNode::set_mesh(&mut state, Resource::new_own(node.rep()), Some(mesh)).unwrap();

        world.commands().append(&mut state.commands);
        world.flush_commands();

        let (node_mesh, node_children) = world.query::<(&NodeMesh, &Children)>().single(&world);
        assert_eq!(node_mesh.node_primitives.len(), 1);

        let primitive_ent = node_mesh.node_primitives.values().next().unwrap();
        let _ = world.get::<Handle<Mesh>>(*primitive_ent).unwrap();
        let _ = world.get::<GlobalTransform>(*primitive_ent).unwrap();
        assert!(node_children.contains(primitive_ent));

        // Remove mesh.
        HostNode::set_mesh(&mut state, node, None).unwrap();

        world.commands().append(&mut state.commands);
        world.flush_commands();

        let node_mesh = world.query::<&NodeMesh>().get_single(&world);
        assert!(node_mesh.is_err());

        assert!(!logs_contain("ERROR"));
        assert!(!logs_contain("error"));
        assert!(!logs_contain("WARN"));
        assert!(!logs_contain("warn"));
    }
}
