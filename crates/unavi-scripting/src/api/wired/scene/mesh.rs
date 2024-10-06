use std::sync::{Arc, RwLock};

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

use super::{nodes::base::NodeRes, primitive::PrimitiveRes};

#[derive(Component, Clone, Copy, Debug)]
pub struct MeshId(pub u32);

#[derive(Default, Debug, Clone)]
pub struct MeshRes(pub Arc<RwLock<MeshData>>);

#[derive(Default, Debug)]
pub struct MeshData {
    pub id: ResourceId,
    pub name: String,
    /// Nodes that are using this mesh.
    pub nodes: Vec<NodeRes>,
    pub primitives: Vec<PrimitiveRes>,
}

impl HostMesh for ScriptData {
    fn new(&mut self) -> wasm_bridge::Result<Resource<MeshRes>> {
        let res = self.table.push(MeshRes::default())?;
        Ok(res)
    }

    fn id(&mut self, self_: Resource<MeshRes>) -> wasm_bridge::Result<u32> {
        let data = self.table.get(&self_)?;
        Ok(data.0.read().unwrap().id.into())
    }
    fn ref_(&mut self, self_: Resource<MeshRes>) -> wasm_bridge::Result<Resource<MeshRes>> {
        let data = self.table.get(&self_)?;
        let res = self.table.push(data.clone())?;
        Ok(res)
    }

    fn name(&mut self, self_: Resource<MeshRes>) -> wasm_bridge::Result<String> {
        let data = self.table.get(&self_)?.0.read().unwrap();
        Ok(data.name.clone())
    }
    fn set_name(&mut self, self_: Resource<MeshRes>, value: String) -> wasm_bridge::Result<()> {
        let mut data = self.table.get(&self_)?.0.write().unwrap();
        data.name = value;
        Ok(())
    }

    fn list_primitives(
        &mut self,
        self_: Resource<MeshRes>,
    ) -> wasm_bridge::Result<Vec<Resource<PrimitiveRes>>> {
        let primitives = self.table.get(&self_)?.0.read().unwrap().primitives.clone();
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

        let mesh = self.table.get_mut(&self_)?;
        mesh.0.write().unwrap().primitives.push(data.clone());

        self.commands.push(move |world: &mut World| {
            let mut assets = world.resource_mut::<Assets<Mesh>>();
            let handle = assets.add(Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::all(),
            ));

            data.0.write().unwrap().handle.get_or_init(|| handle);

            // Create node primitives.
            // let nodes = nodes.read().unwrap();
            // for node_id in mesh_nodes {
            //     let node_ent = nodes.get(&node_id).unwrap();
            //
            //     let p_ent = world
            //         .spawn(PbrBundle {
            //             mesh: handle.clone(),
            //             ..default()
            //         })
            //         .set_parent(*node_ent)
            //         .id();
            //
            //     let mut node_mesh = world.get_mut::<NodeMesh>(*node_ent).unwrap();
            //     node_mesh.node_primitives.insert(primitive_rep, p_ent);
            // }
        });

        Ok(res)
    }
    fn remove_primitive(
        &mut self,
        self_: Resource<MeshRes>,
        value: Resource<PrimitiveRes>,
    ) -> wasm_bridge::Result<()> {
        let primitive = self.table.delete(value)?;
        let id = primitive.0.read().unwrap().id;

        let mut data = self.table.get(&self_)?.0.write().unwrap();
        data.primitives
            .iter()
            .position(|p| p.0.read().unwrap().id == id)
            .map(|index| data.primitives.remove(index));

        // let node_ids = mesh.nodes.iter().map(|res| res.rep()).collect::<Vec<_>>();

        // self.commands.push(move |world: &mut World| {
        //     // Remove node primitives.
        //     let nodes = nodes.read().unwrap();
        //     for node_id in node_ids {
        //         let entity = nodes.get(&node_id).unwrap();
        //         let mut node_mesh = world.get_mut::<NodeMesh>(*entity).unwrap();
        //         if let Some(p_ent) = node_mesh.node_primitives.remove(&rep) {
        //             world.despawn(p_ent);
        //         }
        //     }
        // });

        Ok(())
    }

    fn drop(&mut self, rep: Resource<MeshRes>) -> wasm_bridge::Result<()> {
        self.table.delete(rep)?;

        Ok(())
    }
}

impl Host for ScriptData {}
