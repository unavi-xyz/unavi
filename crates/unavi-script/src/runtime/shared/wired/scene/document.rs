use bevy::prelude::*;
use bevy_hsd::{
    HsdDoc, HsdEntityMaps,
    hydrate::compile::{
        create::{
            HsdCreateMaterial, HsdCreateMesh, HsdCreateNode, HsdRemoveMaterial, HsdRemoveMesh,
            HsdRemoveNode,
        },
        node::HsdDocTransformSet,
    },
};
use blake3::Hash;
use loro::{LoroTree, TreeID};
use smol_str::SmolStr;
use unavi_util::async_commands::try_send_command;

use crate::{
    registry::TransformHandles,
    runtime::shared::wired::scene::{material::MaterialRes, mesh::MeshRes, node::NodeRes},
    util::gen_id,
};

use super::WiredSceneBackend;

pub struct DocRes {
    pub id: Hash,
    pub transforms: TransformHandles,
}

impl Clone for DocRes {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            transforms: self.transforms.clone(),
        }
    }
}

impl WiredSceneBackend {
    pub fn doc_id(&self, rep: u32) -> Option<Vec<u8>> {
        Some(self.docs.get(rep)?.id.as_bytes().to_vec())
    }

    pub fn doc_clone(&mut self, rep: u32) -> Option<u32> {
        self.docs.new_owned(rep)
    }

    pub fn mesh_clone(&mut self, rep: u32) -> Option<u32> {
        self.meshes.new_owned(rep)
    }

    pub fn material_clone(&mut self, rep: u32) -> Option<u32> {
        self.materials.new_owned(rep)
    }

    pub fn doc_translation(&self, rep: u32) -> Option<bevy::math::Vec3> {
        let id = self.docs.get(rep)?.id;
        let registry = self.transform_registry.lock().expect("registry poisoned");
        let val = registry
            .get(&id)?
            .local
            .read()
            .expect("local transform poisoned")
            .translation;
        Some(val)
    }

    pub fn doc_rotation(&self, rep: u32) -> Option<bevy::math::Quat> {
        let id = self.docs.get(rep)?.id;
        let registry = self.transform_registry.lock().expect("registry poisoned");
        let val = registry
            .get(&id)?
            .local
            .read()
            .expect("local transform poisoned")
            .rotation;
        Some(val)
    }

    pub fn doc_scale(&self, rep: u32) -> Option<bevy::math::Vec3> {
        let id = self.docs.get(rep)?.id;
        let registry = self.transform_registry.lock().expect("registry poisoned");
        let val = registry
            .get(&id)?
            .local
            .read()
            .expect("local transform poisoned")
            .scale;
        Some(val)
    }

    pub fn doc_transform(&self, rep: u32) -> Option<bevy::transform::components::Transform> {
        let id = self.docs.get(rep)?.id;
        let registry = self.transform_registry.lock().expect("registry poisoned");
        let val = registry
            .get(&id)?
            .local
            .read()
            .expect("local transform poisoned");
        Some(*val)
    }

    pub fn doc_global_transform(
        &self,
        rep: u32,
    ) -> Option<bevy::transform::components::GlobalTransform> {
        let id = self.docs.get(rep)?.id;
        let registry = self.transform_registry.lock().expect("registry poisoned");
        let val = registry
            .get(&id)?
            .global
            .read()
            .expect("global transform poisoned");
        Some(*val)
    }

    pub fn doc_set_transform(
        &self,
        rep: u32,
        t: bevy::transform::components::Transform,
    ) -> anyhow::Result<()> {
        let doc_id = self
            .docs
            .get(rep)
            .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
            .id;
        try_send_command(bevy::ecs::system::command::trigger(HsdDocTransformSet {
            doc_id,
            transform: t,
        }))?;
        Ok(())
    }

    pub async fn doc_nodes(&mut self, rep: u32) -> anyhow::Result<Vec<u32>> {
        let doc_id = self
            .docs
            .get(rep)
            .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
            .id;

        let (tx, rx) = async_channel::bounded::<Vec<TreeID>>(1);
        try_send_command(move |world: &mut World| {
            let registry = world.resource::<bevy_hsd::DocRegistryMap>();
            let Some(doc_ent) = registry.get_entity(&doc_id) else {
                tx.try_send(vec![]).ok();
                return;
            };
            let ids: Vec<TreeID> = world
                .entity(doc_ent)
                .get::<HsdEntityMaps>()
                .map(|m| m.nodes.keys().copied().collect())
                .unwrap_or_default();
            tx.try_send(ids).ok();
        })?;

        let tree_ids = rx.recv().await?;
        let reps = tree_ids
            .into_iter()
            .map(|id| self.nodes.insert(NodeRes { id, doc_id }))
            .collect();
        Ok(reps)
    }

    pub async fn doc_roots(&mut self, rep: u32) -> anyhow::Result<Vec<u32>> {
        let doc_id = self
            .docs
            .get(rep)
            .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
            .id;
        info!(%doc_id, "root (1)");

        let (tx, rx) = async_channel::bounded::<Vec<TreeID>>(1);
        try_send_command(move |world: &mut World| {
            let registry = world.resource::<bevy_hsd::DocRegistryMap>();
            let Some(doc_ent) = registry.get_entity(&doc_id) else {
                tx.try_send(vec![]).ok();
                return;
            };
            let ids: Vec<TreeID> = world
                .entity(doc_ent)
                .get::<HsdDoc>()
                .map(|d| {
                    d.0.get_map("hsd")
                        .get_or_create_container("nodes", LoroTree::new())
                        .map(|tree| tree.roots())
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            tx.try_send(ids).ok();
        })?;

        info!(%doc_id, "root (2)");
        let tree_ids = rx.recv().await?;
        info!(%doc_id, "root (3)");
        let reps = tree_ids
            .into_iter()
            .map(|id| self.nodes.insert(NodeRes { id, doc_id }))
            .collect();
        info!(%doc_id, "root (4)");
        Ok(reps)
    }

    pub async fn doc_meshes(&mut self, rep: u32) -> anyhow::Result<Vec<u32>> {
        let doc_id = self
            .docs
            .get(rep)
            .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
            .id;

        let (tx, rx) = async_channel::bounded::<Vec<SmolStr>>(1);
        try_send_command(move |world: &mut World| {
            let registry = world.resource::<bevy_hsd::DocRegistryMap>();
            let Some(doc_ent) = registry.get_entity(&doc_id) else {
                tx.try_send(vec![]).ok();
                return;
            };
            let ids: Vec<SmolStr> = world
                .entity(doc_ent)
                .get::<HsdEntityMaps>()
                .map(|m| m.meshes.keys().cloned().collect())
                .unwrap_or_default();
            tx.try_send(ids).ok();
        })?;

        let ids = rx.recv().await?;
        let reps = ids
            .into_iter()
            .map(|id| self.meshes.insert(MeshRes { id, doc_id }))
            .collect();
        Ok(reps)
    }

    pub async fn doc_materials(&mut self, rep: u32) -> anyhow::Result<Vec<u32>> {
        let doc_id = self
            .docs
            .get(rep)
            .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
            .id;

        let (tx, rx) = async_channel::bounded::<Vec<SmolStr>>(1);
        try_send_command(move |world: &mut World| {
            let registry = world.resource::<bevy_hsd::DocRegistryMap>();
            let Some(doc_ent) = registry.get_entity(&doc_id) else {
                tx.try_send(vec![]).ok();
                return;
            };
            let ids: Vec<SmolStr> = world
                .entity(doc_ent)
                .get::<HsdEntityMaps>()
                .map(|m| m.materials.keys().cloned().collect())
                .unwrap_or_default();
            tx.try_send(ids).ok();
        })?;

        let ids = rx.recv().await?;
        let reps = ids
            .into_iter()
            .map(|id| self.materials.insert(MaterialRes { id, doc_id }))
            .collect();
        Ok(reps)
    }

    pub async fn doc_create_node(&mut self, rep: u32) -> anyhow::Result<u32> {
        let doc_id = self
            .docs
            .get(rep)
            .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
            .id;

        let (tx, rx) = async_channel::bounded::<TreeID>(1);
        try_send_command(bevy::ecs::system::command::trigger(HsdCreateNode {
            doc_id,
            parent_id: None,
            tx,
        }))?;

        let tree_id = rx.recv().await?;
        Ok(self.nodes.insert(NodeRes {
            id: tree_id,
            doc_id,
        }))
    }

    pub fn doc_create_mesh(&mut self, rep: u32) -> anyhow::Result<u32> {
        let doc_id = self
            .docs
            .get(rep)
            .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
            .id;
        let id = gen_id();
        try_send_command(bevy::ecs::system::command::trigger(HsdCreateMesh {
            doc_id,
            id: id.clone(),
        }))?;
        Ok(self.meshes.insert(MeshRes { id, doc_id }))
    }

    pub fn doc_create_material(&mut self, rep: u32) -> anyhow::Result<u32> {
        let doc_id = self
            .docs
            .get(rep)
            .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
            .id;
        let id = gen_id();
        try_send_command(bevy::ecs::system::command::trigger(HsdCreateMaterial {
            doc_id,
            id: id.clone(),
        }))?;
        Ok(self.materials.insert(MaterialRes { id, doc_id }))
    }

    pub fn doc_remove_node(&mut self, node_rep: u32) {
        let Some(node) = self.nodes.remove(node_rep) else {
            return;
        };
        let _ = try_send_command(bevy::ecs::system::command::trigger(HsdRemoveNode {
            doc_id: node.doc_id,
            id: node.id,
        }));
    }

    pub fn doc_remove_mesh(&mut self, mesh_rep: u32) {
        let Some(mesh) = self.meshes.remove(mesh_rep) else {
            return;
        };
        let _ = try_send_command(bevy::ecs::system::command::trigger(HsdRemoveMesh {
            doc_id: mesh.doc_id,
            id: mesh.id,
        }));
    }

    pub fn doc_remove_material(&mut self, mat_rep: u32) {
        let Some(mat) = self.materials.remove(mat_rep) else {
            return;
        };
        let _ = try_send_command(bevy::ecs::system::command::trigger(HsdRemoveMaterial {
            doc_id: mat.doc_id,
            id: mat.id,
        }));
    }
}
