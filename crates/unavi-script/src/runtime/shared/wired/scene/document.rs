use bevy::prelude::*;
use bevy_hsd::{
    HsdDoc, HsdEntityMaps, HsdRecordId,
    hydrate::compile::create::{
        HsdCreateMaterial, HsdCreateMesh, HsdCreateNode, HsdRemoveMaterial, HsdRemoveMesh,
        HsdRemoveNode,
    },
};
use blake3::Hash;
use loro::{LoroTree, TreeID};
use smol_str::SmolStr;
use unavi_util::async_commands::AsyncCommands;

use crate::{
    runtime::shared::{
        Api,
        wired::scene::{material::MaterialRes, mesh::MeshRes, node::NodeRes},
    },
    util::gen_id,
};

#[derive(Clone)]
pub struct DocRes {
    pub id: Hash,
}

pub fn doc_id(api: &Api, rep: u32) -> anyhow::Result<Vec<u8>> {
    api.wired_scene
        .try_lock()?
        .docs
        .get(rep)
        .map(|d| d.id.as_bytes().to_vec())
        .ok_or_else(|| anyhow::anyhow!("invalid doc"))
}

pub fn doc_clone(api: &Api, rep: u32) -> anyhow::Result<u32> {
    api.wired_scene
        .try_lock()?
        .docs
        .insert_clone(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid doc"))
}

pub fn doc_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_scene.try_lock()?.docs.remove(rep);
    Ok(())
}

pub async fn doc_roots(api: &Api, rep: u32) -> anyhow::Result<Vec<u32>> {
    let doc_id = api
        .wired_scene
        .lock()
        .await
        .docs
        .get(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
        .id;

    let (tx, rx) = async_channel::bounded::<Vec<TreeID>>(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let doc_ent = {
                let mut qs = world.query::<(Entity, &HsdRecordId)>();
                qs.iter(world)
                    .find(|(_, id)| id.0 == doc_id)
                    .map(|(e, _)| e)
            };
            let Some(doc_ent) = doc_ent else {
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
        })
        .try_send()?;

    let tree_ids = rx.recv().await?;
    let mut scene = api.wired_scene.lock().await;
    Ok(tree_ids
        .into_iter()
        .map(|id| {
            scene.nodes.insert(NodeRes {
                id,
                document: doc_id,
            })
        })
        .collect())
}

pub async fn doc_nodes(api: &Api, rep: u32) -> anyhow::Result<Vec<u32>> {
    let doc_id = api
        .wired_scene
        .lock()
        .await
        .docs
        .get(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
        .id;

    let (tx, rx) = async_channel::bounded::<Vec<TreeID>>(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let doc_ent = {
                let mut qs = world.query::<(Entity, &HsdRecordId)>();
                qs.iter(world)
                    .find(|(_, id)| id.0 == doc_id)
                    .map(|(e, _)| e)
            };
            let Some(doc_ent) = doc_ent else {
                tx.try_send(vec![]).ok();
                return;
            };
            let ids: Vec<TreeID> = world
                .entity(doc_ent)
                .get::<HsdEntityMaps>()
                .map(|m| m.nodes.keys().copied().collect())
                .unwrap_or_default();
            tx.try_send(ids).ok();
        })
        .send()
        .await?;

    let tree_ids = rx.recv().await?;
    let mut scene = api.wired_scene.lock().await;
    Ok(tree_ids
        .into_iter()
        .map(|id| {
            scene.nodes.insert(NodeRes {
                id,
                document: doc_id,
            })
        })
        .collect())
}

pub async fn doc_create_node(api: &Api, rep: u32) -> anyhow::Result<u32> {
    let doc_id = api
        .wired_scene
        .lock()
        .await
        .docs
        .get(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
        .id;

    let (tx, rx) = async_channel::bounded::<TreeID>(1);
    AsyncCommands::default()
        .trigger(HsdCreateNode {
            doc_id,
            parent_id: None,
            tx,
        })
        .try_send()?;

    let tree_id = rx.recv().await?;
    Ok(api.wired_scene.lock().await.nodes.insert(NodeRes {
        id: tree_id,
        document: doc_id,
    }))
}

pub fn doc_remove_node(api: &Api, rep: u32) -> anyhow::Result<()> {
    let Some(node) = api.wired_scene.try_lock()?.nodes.remove(rep) else {
        return Ok(());
    };
    AsyncCommands::default()
        .trigger(HsdRemoveNode {
            doc_id: node.document,
            id: node.id,
        })
        .try_send()?;
    Ok(())
}

pub async fn doc_meshes(api: &Api, rep: u32) -> anyhow::Result<Vec<u32>> {
    let doc_id = api
        .wired_scene
        .lock()
        .await
        .docs
        .get(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
        .id;

    let (tx, rx) = async_channel::bounded::<Vec<SmolStr>>(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let doc_ent = {
                let mut qs = world.query::<(Entity, &HsdRecordId)>();
                qs.iter(world)
                    .find(|(_, id)| id.0 == doc_id)
                    .map(|(e, _)| e)
            };
            let Some(doc_ent) = doc_ent else {
                tx.try_send(vec![]).ok();
                return;
            };
            let ids: Vec<SmolStr> = world
                .entity(doc_ent)
                .get::<HsdEntityMaps>()
                .map(|m| m.meshes.keys().cloned().collect())
                .unwrap_or_default();
            tx.try_send(ids).ok();
        })
        .try_send()?;

    let ids = rx.recv().await?;
    let mut scene = api.wired_scene.lock().await;
    Ok(ids
        .into_iter()
        .map(|id| scene.meshes.insert(MeshRes { id, doc_id }))
        .collect())
}

pub fn doc_create_mesh(api: &Api, rep: u32) -> anyhow::Result<u32> {
    let mut scene = api.wired_scene.try_lock()?;
    let doc_id = scene
        .docs
        .get(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
        .id;
    let id = gen_id();
    AsyncCommands::default()
        .trigger(HsdCreateMesh {
            doc_id,
            id: id.clone(),
        })
        .try_send()?;
    Ok(scene.meshes.insert(MeshRes { id, doc_id }))
}

pub fn doc_remove_mesh(api: &Api, rep: u32) -> anyhow::Result<()> {
    let Some(mesh) = api.wired_scene.try_lock()?.meshes.remove(rep) else {
        return Ok(());
    };
    AsyncCommands::default()
        .trigger(HsdRemoveMesh {
            doc_id: mesh.doc_id,
            id: mesh.id,
        })
        .try_send()?;
    Ok(())
}

pub async fn doc_materials(api: &Api, rep: u32) -> anyhow::Result<Vec<u32>> {
    let doc_id = api
        .wired_scene
        .lock()
        .await
        .docs
        .get(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
        .id;

    let (tx, rx) = async_channel::bounded::<Vec<SmolStr>>(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let doc_ent = {
                let mut qs = world.query::<(Entity, &HsdRecordId)>();
                qs.iter(world)
                    .find(|(_, id)| id.0 == doc_id)
                    .map(|(e, _)| e)
            };
            let Some(doc_ent) = doc_ent else {
                tx.try_send(vec![]).ok();
                return;
            };
            let ids: Vec<SmolStr> = world
                .entity(doc_ent)
                .get::<HsdEntityMaps>()
                .map(|m| m.materials.keys().cloned().collect())
                .unwrap_or_default();
            tx.try_send(ids).ok();
        })
        .try_send()?;

    let ids = rx.recv().await?;
    let mut scene = api.wired_scene.lock().await;
    Ok(ids
        .into_iter()
        .map(|id| scene.materials.insert(MaterialRes { id, doc_id }))
        .collect())
}

pub fn doc_create_material(api: &Api, rep: u32) -> anyhow::Result<u32> {
    let mut scene = api.wired_scene.try_lock()?;
    let doc_id = scene
        .docs
        .get(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
        .id;
    let id = gen_id();
    AsyncCommands::default()
        .trigger(HsdCreateMaterial {
            doc_id,
            id: id.clone(),
        })
        .try_send()?;
    Ok(scene.materials.insert(MaterialRes { id, doc_id }))
}

pub fn doc_remove_material(api: &Api, rep: u32) -> anyhow::Result<()> {
    let Some(mat) = api.wired_scene.try_lock()?.materials.remove(rep) else {
        return Ok(());
    };
    AsyncCommands::default()
        .trigger(HsdRemoveMaterial {
            doc_id: mat.doc_id,
            id: mat.id,
        })
        .try_send()?;
    Ok(())
}
