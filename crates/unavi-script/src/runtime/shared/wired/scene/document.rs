use std::sync::Arc;

use blake3::Hash;
use hsd::HSD_CONTAINER_ID;
use loro::{LoroDoc, TreeID, TreeParentId};

use crate::{
    firewall::Channel,
    runtime::shared::{
        Api, registry::firewall::validate_firewall, wired::scene::prim::PrimRes,
    },
};

#[derive(Clone)]
pub struct DocRes {
    pub doc: Arc<LoroDoc>,
    pub id: Hash,
}

fn get_doc(api: &Api, rep: u32) -> anyhow::Result<DocRes> {
    api.wired_scene
        .try_lock()?
        .docs
        .get(rep)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))
}

pub fn id(api: &Api, rep: u32) -> anyhow::Result<Vec<u8>> {
    Ok(get_doc(api, rep)?.id.as_bytes().to_vec())
}

pub fn clone(api: &Api, rep: u32) -> anyhow::Result<u32> {
    api.wired_scene
        .try_lock()?
        .docs
        .insert_clone(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid doc"))
}

pub fn on_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_scene.try_lock()?.docs.remove(rep);
    Ok(())
}

fn insert_prims(api: &Api, doc: &DocRes, ids: Vec<TreeID>) -> anyhow::Result<Vec<u32>> {
    let mut scene = api.wired_scene.try_lock()?;
    Ok(ids
        .into_iter()
        .map(|id| {
            scene.prims.insert(PrimRes {
                doc: Arc::clone(&doc.doc),
                doc_id: doc.id,
                id,
                is_proxy: false,
            })
        })
        .collect())
}

pub fn roots(api: &Api, rep: u32) -> anyhow::Result<Vec<u32>> {
    let doc = get_doc(api, rep)?;
    let tree = doc.doc.get_tree(&*HSD_CONTAINER_ID);
    let roots = tree.roots();
    insert_prims(api, &doc, roots)
}

pub fn prims(api: &Api, rep: u32) -> anyhow::Result<Vec<u32>> {
    let doc = get_doc(api, rep)?;
    let tree = doc.doc.get_tree(&*HSD_CONTAINER_ID);
    let nodes = tree.nodes();
    insert_prims(api, &doc, nodes)
}

pub fn get_prim(api: &Api, rep: u32, prim_id: String) -> anyhow::Result<Option<u32>> {
    let doc = get_doc(api, rep)?;
    let tree_id: TreeID = match prim_id.parse() {
        Ok(t) => t,
        Err(_) => return Ok(None),
    };
    let tree = doc.doc.get_tree(&*HSD_CONTAINER_ID);
    if !tree.contains(tree_id) {
        return Ok(None);
    }
    let mut scene = api.wired_scene.try_lock()?;
    Ok(Some(scene.prims.insert(PrimRes {
        doc: doc.doc,
        doc_id: doc.id,
        id: tree_id,
        is_proxy: false,
    })))
}

pub fn create_prim(api: &Api, rep: u32) -> anyhow::Result<u32> {
    let doc = get_doc(api, rep)?;
    validate_firewall(&api.doc_id, &doc.id, Channel::SceneWrite)?;
    let tree = doc.doc.get_tree(&*HSD_CONTAINER_ID);
    let tree_id = tree.create(TreeParentId::Root)?;

    let mut scene = api.wired_scene.try_lock()?;
    Ok(scene.prims.insert(PrimRes {
        doc: doc.doc,
        doc_id: doc.id,
        id: tree_id,
        is_proxy: false,
    }))
}

pub fn remove_prim(api: &Api, prim_rep: u32) -> anyhow::Result<()> {
    let prim = {
        let scene = api.wired_scene.try_lock()?;
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
