use std::sync::Arc;

use blake3::Hash;
use hsd::HSD_CONTAINER_ID;
use loro::{LoroDoc, TreeID, TreeParentId};
use tokio::sync::MutexGuard;

use crate::{
    firewall::Channel,
    runtime::shared::{
        Api, registry::firewall::validate_firewall, wired::scene::WiredSceneApi,
        wired::scene::prim::PrimRes,
    },
};

#[derive(Clone)]
pub struct DocRes {
    pub doc: Arc<LoroDoc>,
    pub id: Hash,
}

async fn get_doc(api: &Api, rep: u32) -> anyhow::Result<DocRes> {
    api.wired_scene
        .lock()
        .await
        .docs
        .get(rep)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))
}

pub async fn id(api: &Api, rep: u32) -> anyhow::Result<Vec<u8>> {
    Ok(get_doc(api, rep).await?.id.as_bytes().to_vec())
}

pub async fn clone(api: &Api, rep: u32) -> anyhow::Result<u32> {
    api.wired_scene
        .lock()
        .await
        .docs
        .insert_clone(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid doc"))
}

pub async fn on_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_scene.lock().await.docs.remove(rep);
    Ok(())
}

fn insert_prims(
    scene: &mut MutexGuard<'_, WiredSceneApi>,
    doc: &DocRes,
    ids: Vec<TreeID>,
) -> Vec<u32> {
    ids.into_iter()
        .map(|id| {
            scene.prims.insert(PrimRes {
                doc: Arc::clone(&doc.doc),
                doc_id: doc.id,
                id,
                is_proxy: false,
            })
        })
        .collect()
}

pub async fn roots(api: &Api, rep: u32) -> anyhow::Result<Vec<u32>> {
    let doc = get_doc(api, rep).await?;
    let tree = doc.doc.get_tree(&*HSD_CONTAINER_ID);
    let roots = tree.roots();
    let mut scene = api.wired_scene.lock().await;
    Ok(insert_prims(&mut scene, &doc, roots))
}

pub async fn prims(api: &Api, rep: u32) -> anyhow::Result<Vec<u32>> {
    let doc = get_doc(api, rep).await?;
    let tree = doc.doc.get_tree(&*HSD_CONTAINER_ID);
    let nodes = tree.nodes();
    let mut scene = api.wired_scene.lock().await;
    Ok(insert_prims(&mut scene, &doc, nodes))
}

pub async fn get_prim(api: &Api, rep: u32, prim_id: String) -> anyhow::Result<Option<u32>> {
    let doc = get_doc(api, rep).await?;
    let Ok(tree_id) = TreeID::try_from(prim_id.as_str()) else {
        return Ok(None);
    };
    let tree = doc.doc.get_tree(&*HSD_CONTAINER_ID);
    if !tree.contains(tree_id) {
        return Ok(None);
    }
    let mut scene = api.wired_scene.lock().await;
    Ok(Some(scene.prims.insert(PrimRes {
        doc: doc.doc,
        doc_id: doc.id,
        id: tree_id,
        is_proxy: false,
    })))
}

pub async fn create_prim(api: &Api, rep: u32) -> anyhow::Result<u32> {
    let doc = get_doc(api, rep).await?;
    validate_firewall(&api.doc_id, &doc.id, Channel::SceneWrite)?;
    let tree = doc.doc.get_tree(&*HSD_CONTAINER_ID);
    let tree_id = tree.create(TreeParentId::Root)?;

    let mut scene = api.wired_scene.lock().await;
    Ok(scene.prims.insert(PrimRes {
        doc: doc.doc,
        doc_id: doc.id,
        id: tree_id,
        is_proxy: false,
    }))
}

pub async fn remove_prim(api: &Api, prim_rep: u32) -> anyhow::Result<()> {
    let prim = {
        let scene = api.wired_scene.lock().await;
        scene
            .prims
            .get(prim_rep)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("invalid prim rep: {prim_rep}"))?
    };
    if prim.is_proxy {
        return Ok(());
    }
    validate_firewall(&api.doc_id, &prim.doc_id, Channel::SceneWrite)?;
    let tree = prim.doc.get_tree(&*HSD_CONTAINER_ID);
    tree.delete(prim.id)?;
    Ok(())
}
