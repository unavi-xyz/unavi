use crate::StoreState;
use anyhow::anyhow;
use wasm_bridge::component::Resource;

use super::{
    bindgen::wired::gltf::{
        mesh::Mesh,
        node::{Host, HostNode, Node, Transform},
    },
    WiredGltfAction,
};

impl HostNode for StoreState {
    fn id(&mut self, self_: Resource<Node>) -> wasm_bridge::Result<u32> {
        Ok(self_.rep())
    }

    fn name(&mut self, self_: Resource<Node>) -> wasm_bridge::Result<String> {
        let node = self.table.get(&self_).map_err(|e| anyhow!("{:?}", e))?;
        Ok(node.name.clone())
    }
    fn set_name(&mut self, self_: Resource<Node>, value: String) -> wasm_bridge::Result<()> {
        let node = self.table.get_mut(&self_).map_err(|e| anyhow!("{:?}", e))?;
        node.name = value;
        Ok(())
    }

    fn mesh(&mut self, self_: Resource<Node>) -> wasm_bridge::Result<Option<Resource<Mesh>>> {
        let node = self.table.get(&self_).map_err(|e| anyhow!("{:?}", e))?;
        Ok(node.mesh.map(Resource::new_own))
    }
    fn set_mesh(
        &mut self,
        self_: Resource<Node>,
        value: Option<Resource<Mesh>>,
    ) -> wasm_bridge::Result<()> {
        let node = self.table.get_mut(&self_).map_err(|e| anyhow!("{:?}", e))?;
        node.mesh = value.map(|v| v.rep());

        self.sender.send(WiredGltfAction::SetNodeMesh {
            id: self_.rep(),
            mesh: node.mesh,
        })?;

        Ok(())
    }

    fn parent(&mut self, self_: Resource<Node>) -> wasm_bridge::Result<Option<Resource<Node>>> {
        let node = self.table.get(&self_).map_err(|e| anyhow!("{:?}", e))?;
        Ok(node.parent.map(Resource::new_own))
    }
    fn children(&mut self, self_: Resource<Node>) -> wasm_bridge::Result<Vec<Resource<Node>>> {
        let node = self.table.get_mut(&self_).map_err(|e| anyhow!("{:?}", e))?;
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
        let node = self.table.get_mut(&self_).map_err(|e| anyhow!("{:?}", e))?;
        node.children.insert(value.rep());

        // Remove child from old parent's children.
        let child = self.table.get(&value).map_err(|e| anyhow!("{:?}", e))?;
        if let Some(parent_rep) = child.parent {
            let parent_res = Resource::new_own(parent_rep);
            self.remove_child(parent_res, self_)?;
        }

        // Set parent.
        let child = self.table.get_mut(&value).map_err(|e| anyhow!("{:?}", e))?;
        child.parent = Some(rep);

        self.sender.send(WiredGltfAction::SetNodeParent {
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
        let node = self.table.get_mut(&self_).map_err(|e| anyhow!("{:?}", e))?;
        node.children.remove(&value.rep());

        self.sender.send(WiredGltfAction::SetNodeParent {
            id: self_.rep(),
            parent: None,
        })?;

        Ok(())
    }

    fn transform(&mut self, self_: Resource<Node>) -> wasm_bridge::Result<Transform> {
        let node = self.table.get(&self_).map_err(|e| anyhow!("{:?}", e))?;
        Ok(node.transform)
    }
    fn set_transform(
        &mut self,
        self_: Resource<Node>,
        value: Transform,
    ) -> wasm_bridge::Result<()> {
        let node = self.table.get_mut(&self_).map_err(|e| anyhow!("{:?}", e))?;
        node.transform = value;

        self.sender.send(WiredGltfAction::SetNodeTransform {
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

    fn drop(&mut self, _rep: Resource<Node>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}

impl Host for StoreState {
    fn list_nodes(&mut self) -> wasm_bridge::Result<Vec<Resource<Node>>> {
        Ok(self
            .nodes
            .iter()
            .map(|rep| Resource::new_own(*rep))
            .collect())
    }

    fn create_node(&mut self) -> wasm_bridge::Result<Resource<Node>> {
        let resource = self
            .table
            .push(Node::default())
            .map_err(|e| anyhow!("{:?}", e))?;
        let node_rep = resource.rep();
        self.nodes.push(node_rep);

        self.sender
            .send(WiredGltfAction::CreateNode { id: node_rep })?;

        Ok(Resource::new_own(node_rep))
    }

    fn remove_node(&mut self, value: Resource<Node>) -> wasm_bridge::Result<()> {
        let rep = value.rep();
        self.table.delete(value).map_err(|e| anyhow!("{:?}", e))?;

        let index =
            self.nodes
                .iter()
                .enumerate()
                .find_map(|(i, item)| if *item == rep { Some(i) } else { None });
        if let Some(index) = index {
            self.nodes.remove(index);
        }

        self.sender.send(WiredGltfAction::RemoveNode { id: rep })?;

        Ok(())
    }
}
