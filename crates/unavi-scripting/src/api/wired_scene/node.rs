use bevy::utils::HashSet;
use wasm_bridge::component::Resource;

use crate::{
    actions::ScriptAction,
    api::{
        wired_input::spatial_handler::SpatialHandler,
        wired_physics::wired::{
            math::types::Vec3,
            physics::types::{RigidBodyType, Shape, Sphere},
        },
    },
    state::StoreState,
};

use super::wired::{
    physics::types::{Collider, RigidBody},
    scene::{
        mesh::Mesh,
        node::{Host, HostNode, Transform},
    },
};

#[derive(Default)]
pub struct Node {
    pub children: HashSet<u32>,
    pub collider: Option<Resource<Collider>>,
    pub input_handlers: Vec<Resource<SpatialHandler>>,
    pub mesh: Option<u32>,
    pub name: String,
    pub parent: Option<u32>,
    pub rigid_body: Option<Resource<RigidBody>>,
    pub transform: Transform,
}

impl HostNode for StoreState {
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
        Ok(node.mesh.map(Resource::new_own))
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
        Ok(node.parent.map(Resource::new_own))
    }
    fn children(&mut self, self_: Resource<Node>) -> wasm_bridge::Result<Vec<Resource<Node>>> {
        let node = self.table.get_mut(&self_)?;
        Ok(node
            .children
            .iter()
            .map(|rep| Resource::new_own(*rep))
            .collect())
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
        let res = node
            .collider
            .as_ref()
            .map(|res| Resource::new_own(res.rep()));
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
        let res = node
            .rigid_body
            .as_ref()
            .map(|res| Resource::new_own(res.rep()));
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

    fn drop(&mut self, _rep: Resource<Node>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}

impl Host for StoreState {
    fn list_nodes(&mut self) -> wasm_bridge::Result<Vec<Resource<Node>>> {
        Ok(self
            .nodes
            .iter()
            .map(|res| Resource::new_own(res.rep()))
            .collect())
    }

    fn create_node(&mut self) -> wasm_bridge::Result<Resource<Node>> {
        let node = Node::default();
        let resource = self.table.push(node)?;
        let node_rep = resource.rep();
        self.nodes.push(resource);

        self.sender
            .send(ScriptAction::CreateNode { id: node_rep })?;

        Ok(Resource::new_own(node_rep))
    }

    fn remove_node(&mut self, value: Resource<Node>) -> wasm_bridge::Result<()> {
        let rep = value.rep();
        self.table.delete(value)?;

        let index =
            self.nodes
                .iter()
                .enumerate()
                .find_map(|(i, item)| if item.rep() == rep { Some(i) } else { None });
        if let Some(index) = index {
            self.nodes.remove(index);
        }

        self.sender.send(ScriptAction::RemoveNode { id: rep })?;

        Ok(())
    }
}
