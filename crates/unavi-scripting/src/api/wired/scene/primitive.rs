use std::sync::{Arc, OnceLock, RwLock};

use bevy::{
    prelude::*,
    render::mesh::{Indices, VertexAttributeValues},
};
use wasm_bridge::component::Resource;

use crate::{
    api::{
        id::ResourceId,
        wired::scene::bindings::mesh::{HostMesh, HostPrimitive, Material},
    },
    data::ScriptData,
};

use super::{material::MaterialRes, mesh::MeshId};

#[derive(Default, Debug, Clone)]
pub struct PrimitiveRes(pub Arc<RwLock<PrimitiveData>>);

#[derive(Debug, Default)]
pub struct PrimitiveData {
    pub id: ResourceId,
    pub handle: OnceLock<Handle<Mesh>>,
    pub material: Option<MaterialRes>,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct PrimitiveId(pub u32);

#[derive(Bundle)]
pub struct GltfPrimitiveBundle {
    pub id: PrimitiveId,
    pub handle: Handle<Mesh>,
    pub mesh: MeshId,
}

impl HostPrimitive for ScriptData {
    fn id(&mut self, self_: Resource<PrimitiveRes>) -> wasm_bridge::Result<u32> {
        let data = self.table.get(&self_)?.0.read().unwrap();
        Ok(data.id.into())
    }

    fn material(
        &mut self,
        self_: Resource<PrimitiveRes>,
    ) -> wasm_bridge::Result<Option<Resource<Material>>> {
        let material = self.table.get(&self_)?.0.read().unwrap().material.clone();
        let res = match material.clone() {
            Some(m) => Some(self.table.push(m)?),
            None => None,
        };
        Ok(res)
    }
    fn set_material(
        &mut self,
        self_: Resource<PrimitiveRes>,
        value: Option<Resource<Material>>,
    ) -> wasm_bridge::Result<()> {
        let material = match &value {
            Some(m) => {
                let data = self.table.get(m)?;
                Some(data.clone())
            }
            None => None,
        };

        let primitive = self.table.get(&self_)?;
        primitive.0.write().unwrap().material = material;

        // let mesh = Resource::<MeshRes>::new_own(primitive.mesh);
        // let mesh = self.table.get(&mesh)?;
        //
        // let mesh_nodes = mesh.nodes.iter().map(|r| r.rep()).collect::<Vec<_>>();
        //
        // let default_material = self
        //     .api
        //     .wired_scene
        //     .as_ref()
        //     .unwrap()
        //     .default_material
        //     .clone();
        // let nodes = self
        //     .api
        //     .wired_scene
        //     .as_ref()
        //     .unwrap()
        //     .entities
        //     .nodes
        //     .clone();
        //
        // self.commands.push(move |world: &mut World| {
        //     let nodes = nodes.read().unwrap();
        //
        //     // let material = material_rep.map(|r| materials.get(&r).unwrap().handle.clone());
        //
        //     for nid in mesh_nodes {
        //         let node_ent = nodes.get(&nid).unwrap(); // TODO: is this unwrap safe?
        //         let node_mesh = world.get::<NodeMesh>(*node_ent).unwrap();
        //
        //         for (pid, p_ent) in node_mesh.node_primitives.clone() {
        //             if pid == rep {
        //                 // if let Some(material) = &material {
        //                 //     world.entity_mut(p_ent).insert(material.clone());
        //                 // } else {
        //                 //     world.entity_mut(p_ent).insert(default_material.clone());
        //                 // }
        //             }
        //         }
        //     }
        // });

        Ok(())
    }

    fn set_indices(
        &mut self,
        self_: Resource<PrimitiveRes>,
        value: Vec<u32>,
    ) -> wasm_bridge::Result<()> {
        let handle = self.table.get(&self_)?.0.read().unwrap().handle.clone();

        self.commands.push(move |world: &mut World| {
            let mut assets = world.resource_mut::<Assets<Mesh>>();
            let mesh = assets.get_mut(handle.get().unwrap()).unwrap();
            mesh.insert_indices(Indices::U32(value));
        });
        Ok(())
    }
    fn set_positions(
        &mut self,
        self_: Resource<PrimitiveRes>,
        value: Vec<f32>,
    ) -> wasm_bridge::Result<()> {
        let handle = self.table.get(&self_)?.0.read().unwrap().handle.clone();

        self.commands.push(move |world: &mut World| {
            let mut assets = world.resource_mut::<Assets<Mesh>>();
            let mesh = assets.get_mut(handle.get().unwrap()).unwrap();

            let value = value.chunks(3).map(|x| [x[0], x[1], x[2]]).collect();

            mesh.insert_attribute(
                Mesh::ATTRIBUTE_POSITION,
                VertexAttributeValues::Float32x3(value),
            );
        });
        Ok(())
    }
    fn set_normals(
        &mut self,
        self_: Resource<PrimitiveRes>,
        value: Vec<f32>,
    ) -> wasm_bridge::Result<()> {
        let handle = self.table.get(&self_)?.0.read().unwrap().handle.clone();

        self.commands.push(move |world: &mut World| {
            let mut assets = world.resource_mut::<Assets<Mesh>>();
            let mesh = assets.get_mut(handle.get().unwrap()).unwrap();

            let value = value.chunks(3).map(|x| [x[0], x[1], x[2]]).collect();

            mesh.insert_attribute(
                Mesh::ATTRIBUTE_NORMAL,
                VertexAttributeValues::Float32x3(value),
            );
        });
        Ok(())
    }
    fn set_uvs(
        &mut self,
        self_: Resource<PrimitiveRes>,
        value: Vec<f32>,
    ) -> wasm_bridge::Result<()> {
        let handle = self.table.get(&self_)?.0.read().unwrap().handle.clone();

        self.commands.push(move |world: &mut World| {
            let mut assets = world.resource_mut::<Assets<Mesh>>();
            let mesh = assets.get_mut(handle.get().unwrap()).unwrap();

            if value.len() % 2 != 0 {
                warn!("UVs do not have an even length! Got: {}", value.len());
            } else {
                let value = value.chunks(2).map(|x| [x[0], x[1]]).collect::<Vec<_>>();

                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_UV_0,
                    VertexAttributeValues::Float32x2(value),
                );
            }
        });
        Ok(())
    }

    fn drop(&mut self, _rep: Resource<PrimitiveRes>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}
