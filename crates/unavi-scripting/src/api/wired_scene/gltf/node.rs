use std::cell::Cell;

use bevy::utils::HashSet;
use wasm_bridge::component::Resource;

use crate::{
    actions::ScriptAction,
    api::{
        utils::{RefCount, RefCountCell, RefResource},
        wired_input::input_handler::InputHandler,
        wired_physics::wired::{
            math::types::Vec3,
            physics::types::{RigidBodyType, Shape, Sphere},
        },
    },
    state::StoreState,
};

use crate::api::wired_scene::wired::{
    physics::types::{Collider, RigidBody},
    scene::{
        mesh::Mesh,
        node::{Host, HostNode, Transform},
    },
};

#[derive(Default, Debug)]
pub struct Node {
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

impl RefCount for Node {
    fn ref_count(&self) -> &Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for Node {}

impl HostNode for StoreState {
    fn new(&mut self) -> wasm_bridge::Result<Resource<Node>> {
        let node = Node::default();
        let table_res = self.table.push(node)?;
        let res = Node::from_res(&table_res, &self.table)?;
        self.nodes.push(table_res);

        self.sender
            .send(ScriptAction::CreateNode { id: res.rep() })?;

        Ok(res)
    }

    fn id(&mut self, self_: Resource<Node>) -> wasm_bridge::Result<u32> {
        Ok(self_.rep())
    }

    fn name(&mut self, self_: Resource<Node>) -> wasm_bridge::Result<String> {
        let node = self.table.get(&self_)?;
        Ok(node.name.clone())
    }
    fn set_name(&mut self, self_: Resource<Node>, value: String) -> wasm_bridge::Result<()> {
        let node = self.table.get_mut(&self_)?;
        node.name = value;
        Ok(())
    }

    fn mesh(&mut self, self_: Resource<Node>) -> wasm_bridge::Result<Option<Resource<Mesh>>> {
        let node = self.table.get(&self_)?;
        let mesh = match node.mesh {
            Some(m) => Some(Mesh::from_rep(m, &self.table)?),
            None => None,
        };
        Ok(mesh)
    }
    fn set_mesh(
        &mut self,
        self_: Resource<Node>,
        value: Option<Resource<Mesh>>,
    ) -> wasm_bridge::Result<()> {
        let node = self.table.get_mut(&self_)?;
        node.mesh = value.map(|v| v.rep());

        self.sender.send(ScriptAction::SetNodeMesh {
            id: self_.rep(),
            mesh: node.mesh,
        })?;

        Ok(())
    }

    fn parent(&mut self, self_: Resource<Node>) -> wasm_bridge::Result<Option<Resource<Node>>> {
        let node = self.table.get(&self_)?;
        let parent = match node.parent {
            Some(p) => Some(Node::from_rep(p, &self.table)?),
            None => None,
        };
        Ok(parent)
    }
    fn children(&mut self, self_: Resource<Node>) -> wasm_bridge::Result<Vec<Resource<Node>>> {
        let node = self.table.get(&self_)?;
        let children = node
            .children
            .iter()
            .map(|rep| Node::from_rep(*rep, &self.table))
            .collect::<Result<_, _>>()?;
        Ok(children)
    }
    fn add_child(
        &mut self,
        self_: Resource<Node>,
        value: Resource<Node>,
    ) -> wasm_bridge::Result<()> {
        let rep = self_.rep();

        // Add child to children.
        let node = self.table.get_mut(&self_)?;
        node.children.insert(value.rep());

        // Remove child from old parent's children.
        let child = self.table.get(&value)?;
        if let Some(parent_rep) = child.parent {
            let parent_res = Resource::new_own(parent_rep);
            self.remove_child(parent_res, self_)?;
        }

        // Set parent.
        let child = self.table.get_mut(&value)?;
        child.parent = Some(rep);

        self.sender.send(ScriptAction::SetNodeParent {
            id: value.rep(),
            parent: child.parent,
        })?;

        Ok(())
    }
    fn remove_child(
        &mut self,
        self_: Resource<Node>,
        value: Resource<Node>,
    ) -> wasm_bridge::Result<()> {
        let node = self.table.get_mut(&self_)?;
        node.children.remove(&value.rep());

        self.sender.send(ScriptAction::SetNodeParent {
            id: self_.rep(),
            parent: None,
        })?;

        Ok(())
    }

    fn transform(&mut self, self_: Resource<Node>) -> wasm_bridge::Result<Transform> {
        let node = self.table.get(&self_)?;
        Ok(node.transform)
    }
    fn set_transform(
        &mut self,
        self_: Resource<Node>,
        value: Transform,
    ) -> wasm_bridge::Result<()> {
        let node = self.table.get_mut(&self_)?;
        node.transform = value;

        self.sender.send(ScriptAction::SetNodeTransform {
            id: self_.rep(),
            transform: bevy::prelude::Transform {
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
            },
        })?;

        Ok(())
    }

    fn collider(
        &mut self,
        self_: Resource<Node>,
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
        self_: Resource<Node>,
        value: Option<Resource<Collider>>,
    ) -> wasm_bridge::Result<()> {
        if let Some(value) = &value {
            let collider = self.table.get(value)?;

            let collider = match collider.shape {
                Shape::Sphere(Sphere { radius }) => bevy_xpbd_3d::prelude::Collider::sphere(radius),
                Shape::Cuboid(Vec3 { x, y, z }) => bevy_xpbd_3d::prelude::Collider::cuboid(x, y, z),
            };

            self.sender.send(ScriptAction::SetNodeCollider {
                id: self_.rep(),
                collider: Some(collider),
            })?;
        } else {
            self.sender.send(ScriptAction::SetNodeCollider {
                id: self_.rep(),
                collider: None,
            })?;
        }

        let node = self.table.get_mut(&self_)?;
        node.collider = value;

        Ok(())
    }

    fn rigid_body(
        &mut self,
        self_: Resource<Node>,
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
        self_: Resource<Node>,
        value: Option<Resource<RigidBody>>,
    ) -> wasm_bridge::Result<()> {
        if let Some(value) = &value {
            let rigid_body = self.table.get(value)?;

            let rigid_body = match rigid_body.rigid_body_type {
                RigidBodyType::Dynamic => bevy_xpbd_3d::prelude::RigidBody::Dynamic,
                RigidBodyType::Fixed => bevy_xpbd_3d::prelude::RigidBody::Static,
                RigidBodyType::Kinematic => bevy_xpbd_3d::prelude::RigidBody::Kinematic,
            };

            self.sender.send(ScriptAction::SetNodeRigidBody {
                id: self_.rep(),
                rigid_body: Some(rigid_body),
            })?;
        } else {
            self.sender.send(ScriptAction::SetNodeRigidBody {
                id: self_.rep(),
                rigid_body: None,
            })?;
        }

        let node = self.table.get_mut(&self_)?;
        node.rigid_body = value;
        Ok(())
    }

    fn input_handler(
        &mut self,
        self_: Resource<Node>,
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
        self_: Resource<Node>,
        value: Option<Resource<InputHandler>>,
    ) -> wasm_bridge::Result<()> {
        let handler = if let Some(value) = &value {
            let data = self.table.get(value)?;
            Some(data.sender.clone())
        } else {
            None
        };

        let node = self.table.get_mut(&self_)?;
        node.input_handler = value;

        self.sender.send(ScriptAction::SetNodeInputHandler {
            id: self_.rep(),
            handler,
        })?;

        Ok(())
    }

    fn drop(&mut self, rep: Resource<Node>) -> wasm_bridge::Result<()> {
        let id = rep.rep();
        let deleted = Node::handle_drop(rep, &mut self.table)?;

        if deleted {
            let index = self.nodes.iter().enumerate().find_map(|(i, item)| {
                if item.rep() == id {
                    Some(i)
                } else {
                    None
                }
            });
            if let Some(index) = index {
                self.nodes.remove(index);
            }

            self.sender.send(ScriptAction::RemoveNode { id })?;
        }

        Ok(())
    }
}

impl Host for StoreState {}

#[cfg(test)]
mod tests {
    crate::generate_resource_tests!(Node);
}
