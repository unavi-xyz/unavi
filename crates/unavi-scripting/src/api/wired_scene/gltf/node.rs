use std::cell::Cell;

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use wasm_bridge::component::Resource;

use crate::{
    api::{
        utils::{RefCount, RefCountCell, RefResource},
        wired_input::input_handler::{InputHandler, InputHandlerSender},
        wired_physics::wired::{
            math::types::Vec3,
            physics::types::{RigidBodyType, Shape, Sphere},
        },
    },
    state::{PrimitiveState, StoreState},
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
    pub children: HashSet<u32>,
    pub collider: Option<Resource<Collider>>,
    pub input_handler: Option<Resource<InputHandler>>,
    pub mesh: Option<u32>,
    pub name: String,
    pub parent: Option<u32>,
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
        let res = NodeRes::from_res(&table_res, &self.table)?;
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
        let node = self.table.get(&self_)?;
        Ok(node.name.clone())
    }
    fn set_name(&mut self, self_: Resource<NodeRes>, value: String) -> wasm_bridge::Result<()> {
        let node = self.table.get_mut(&self_)?;
        node.name = value;
        Ok(())
    }

    fn mesh(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<Option<Resource<MeshRes>>> {
        let node = self.table.get(&self_)?;
        let mesh = match node.mesh {
            Some(m) => Some(MeshRes::from_rep(m, &self.table)?),
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
        let prev_mesh_rep = node.mesh;
        if let Some(prev_mesh_rep) = prev_mesh_rep {
            let mesh = self
                .table
                .get_mut::<MeshRes>(&Resource::new_own(prev_mesh_rep))?;
            mesh.nodes.remove(&rep);
        }

        let mut primitive_ids = Vec::new();
        if let Some(mesh_res) = &value {
            let mesh = self.table.get_mut::<MeshRes>(mesh_res)?;
            mesh.nodes.insert(rep);

            for p in mesh.primitives.iter() {
                primitive_ids.push(*p);
            }
        }

        let node = self.table.get_mut(&self_)?;
        node.mesh = value.map(|v| v.rep());

        let mesh_rep = node.mesh;
        let primitives = self.entities.primitives.clone();
        let nodes = self.entities.nodes.clone();
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

            let mut node_ent = world.entity_mut(*node_ent);
            let mut node_primitives = HashMap::new();

            // Add new node primitives.
            node_ent.with_children(|world| {
                let primitives = primitives.read().unwrap();
                for p in primitive_ids {
                    let PrimitiveState { handle, .. } = primitives.get(&p).unwrap();

                    let p_ent = world.spawn((SpatialBundle::default(), handle.clone())).id();
                    node_primitives.insert(p, p_ent);

                    // TODO: add material
                }
            });

            // Add new mesh.
            if let Some(mesh_rep) = mesh_rep {
                node_ent.insert(NodeMesh {
                    id: mesh_rep,
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
        let node = self.table.get(&self_)?;
        let parent = match node.parent {
            Some(p) => Some(NodeRes::from_rep(p, &self.table)?),
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
            .map(|rep| NodeRes::from_rep(*rep, &self.table))
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
        let node = self.table.get_mut(&self_)?;
        node.children.insert(child_rep);

        // Remove child from old parent's children.
        let child = self.table.get(&value)?;
        if let Some(parent_rep) = child.parent {
            let parent_res = Resource::new_own(parent_rep);
            self.remove_child(parent_res, self_)?;
        }

        // Set parent.
        let child = self.table.get_mut(&value)?;
        child.parent = Some(parent_rep);

        // Update ECS.
        let nodes = self.entities.nodes.clone();
        self.commands.push(move |world: &mut World| {
            let nodes = nodes.read().unwrap();
            let child_ent = nodes.get(&child_rep).unwrap();
            let parent_ent = nodes.get(&parent_rep).unwrap();

            world
                .commands()
                .entity(*parent_ent)
                .push_children(&[*child_ent]);
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
        node.children.remove(&child_rep);

        let nodes = self.entities.nodes.clone();
        self.commands.push(move |world: &mut World| {
            let nodes = nodes.read().unwrap();
            let child_ent = nodes.get(&child_rep).unwrap();
            world.commands().entity(*child_ent).remove_parent();
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
                    Shape::Sphere(Sphere { radius }) => {
                        bevy_xpbd_3d::prelude::Collider::sphere(radius)
                    }
                    Shape::Cuboid(Vec3 { x, y, z }) => {
                        bevy_xpbd_3d::prelude::Collider::cuboid(x, y, z)
                    }
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
                    RigidBodyType::Dynamic => bevy_xpbd_3d::prelude::RigidBody::Dynamic,
                    RigidBodyType::Fixed => bevy_xpbd_3d::prelude::RigidBody::Static,
                    RigidBodyType::Kinematic => bevy_xpbd_3d::prelude::RigidBody::Kinematic,
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
