use bevy::prelude::*;
use wasm_bridge::component::Resource;

use crate::{
    api::wired::{
        input::input_handler::{InputHandlerRes, InputHandlerSender},
        math::bindings::types::Transform,
        physics::{collider::ColliderRes, rigid_body::RigidBodyRes},
        scene::{
            bindings::node::{Host, HostNode},
            mesh::MeshRes,
        },
    },
    data::ScriptData,
};

use super::base::NodeRes;

/// Instance of a mesh under a node.
#[derive(Component)]
pub struct NodeMesh {
    pub primitives: Vec<Entity>,
}

impl HostNode for ScriptData {
    fn new(&mut self) -> wasm_bridge::Result<Resource<NodeRes>> {
        NodeRes::new_res(self)
    }

    fn id(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<u32> {
        let data = self.table.get(&self_)?.0.read().unwrap();
        Ok(data.id.into())
    }
    fn ref_(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<Resource<NodeRes>> {
        let data = self.table.get(&self_)?;
        let res = self.table.push(data.clone())?;
        Ok(res)
    }

    fn name(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<String> {
        let data = self.table.get(&self_)?.0.read().unwrap();
        Ok(data.name.clone())
    }
    fn set_name(&mut self, self_: Resource<NodeRes>, value: String) -> wasm_bridge::Result<()> {
        let mut data = self.table.get(&self_)?.0.write().unwrap();
        data.name = value;
        Ok(())
    }

    fn mesh(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<Option<Resource<MeshRes>>> {
        let mesh = self.table.get(&self_)?.0.read().unwrap().mesh.clone();
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
        if let Some(prev_mesh) = &data.0.read().unwrap().mesh {
            let mut prev_mesh_data = prev_mesh.0.write().unwrap();
            prev_mesh_data
                .nodes
                .iter()
                .position(|r| r.0.read().unwrap().id == prev_mesh_data.id)
                .map(|index| prev_mesh_data.nodes.remove(index));

            // TODO: Remove node primitive
        }

        // Set new mesh.
        let mesh_data = match &value {
            Some(v) => Some(self.table.get(v)?.clone()),
            None => None,
        };

        let primitives = mesh_data
            .as_ref()
            .map(|d| {
                let mut d_write = d.0.write().unwrap();
                d_write.nodes.push(data.clone());
                d_write.primitives.clone()
            })
            .unwrap_or_default();

        let mut data_write = data.0.write().unwrap();
        data_write.mesh = mesh_data;
        drop(data_write);

        let default_material = self
            .api
            .wired_scene
            .as_ref()
            .unwrap()
            .default_material
            .clone();

        self.commands.push(move |world: &mut World| {
            let data_read = data.0.read().unwrap();
            let node_ent = data_read.entity.get().unwrap();

            // Remove previous node primitives.
            let to_remove = world
                .get::<NodeMesh>(*node_ent)
                .map(|m| m.primitives.clone())
                .unwrap_or_default();

            for e in to_remove {
                world.despawn(e);
            }

            let mut p_ents = Vec::new();

            // Add new node primitives.
            for p in primitives.iter() {
                let p_data = p.0.read().unwrap();

                let mesh = p_data.handle.get().unwrap().clone();
                let material = p_data
                    .material
                    .as_ref()
                    .map(|m| m.0.read().unwrap().handle.get().unwrap().clone())
                    .unwrap_or_else(|| default_material.clone());

                let p_ent = world
                    .spawn(PbrBundle {
                        mesh,
                        material,
                        ..default()
                    })
                    .set_parent(*node_ent)
                    .id();

                p_ents.push(p_ent);
            }

            let mut node_ent = world.entity_mut(*node_ent);

            // Add new mesh.
            if primitives.is_empty() {
                node_ent.remove::<NodeMesh>();
            } else {
                node_ent.insert(NodeMesh { primitives: p_ents });
            }
        });

        Ok(())
    }

    fn global_transform(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<Transform> {
        NodeRes::global_transform(self, &self_)
    }
    fn transform(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<Transform> {
        let data = self.table.get(&self_)?.0.read().unwrap();
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
        let collider = self.table.get(&self_)?.0.read().unwrap().collider.clone();
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

        let component = c_data.as_ref().map(|d| d.0.read().unwrap().component());

        let data = self.table.get(&self_)?;
        data.0.write().unwrap().collider = c_data;

        self.node_insert_option(data.clone(), component);

        Ok(())
    }

    fn rigid_body(
        &mut self,
        self_: Resource<NodeRes>,
    ) -> wasm_bridge::Result<Option<Resource<RigidBodyRes>>> {
        let rigid_body = self.table.get(&self_)?.0.read().unwrap().rigid_body.clone();
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

        let component = r_data.as_ref().map(|d| d.0.read().unwrap().component());

        let data = self.table.get(&self_)?;
        data.0.write().unwrap().rigid_body = r_data;

        self.node_insert_option(data.clone(), component);

        Ok(())
    }

    fn input_handler(
        &mut self,
        self_: Resource<NodeRes>,
    ) -> wasm_bridge::Result<Option<Resource<InputHandlerRes>>> {
        let input_handler = self
            .table
            .get(&self_)?
            .0
            .read()
            .unwrap()
            .input_handler
            .clone();

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
            .map(|r| InputHandlerSender(r.0.read().unwrap().sender.clone()));

        let data = self.table.get(&self_)?;
        data.0.write().unwrap().input_handler = handler;

        self.node_insert_option(data.clone(), component);

        Ok(())
    }

    fn drop(&mut self, rep: Resource<NodeRes>) -> wasm_bridge::Result<()> {
        self.table.delete(rep)?;
        // if dropped {
        //     let nodes = self
        //         .api
        //         .wired_scene
        //         .as_ref()
        //         .unwrap()
        //         .entities
        //         .nodes
        //         .clone();
        //
        //     self.commands.push(move |world: &mut World| {
        //         let mut nodes = nodes.write().unwrap();
        //         let entity = nodes.remove(&id).unwrap();
        //         world.despawn(entity);
        //     });
        // }

        Ok(())
    }
}

impl Host for ScriptData {}
