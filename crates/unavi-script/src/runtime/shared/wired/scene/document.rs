use std::sync::Arc;

use blake3::Hash;
use hsd::HSD_CONTAINER_ID;
use loro::{
    LoroDoc,
    TreeID,
    TreeParentId,
};
use tokio::sync::MutexGuard;

use crate::{
    firewall::Channel,
    quota::{
        Flow,
        Quota,
        QuotaError,
        Stock,
        registry::document_quota,
    },
    runtime::shared::{
        Api,
        registry::{
            firewall::validate_firewall,
            transform::DOC_ROOT_TRANSFORM_REGISTRY,
        },
        wired::scene::{
            WiredSceneApi,
            prim::PrimRes,
        },
    },
};

#[derive(Clone, Copy, Default)]
pub struct XformValue {
    pub translation: [f32; 3],
    pub rotation:    [f32; 4],
    pub scale:       [f32; 3],
}

#[derive(Clone)]
pub struct DocRes {
    pub doc: Arc<LoroDoc>,
    pub id:  Hash,
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
        .insert_clone(rep, &api.quota)
        .ok_or_else(|| anyhow::anyhow!("invalid doc"))?
        .map_err(Into::into)
}

pub async fn on_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_scene.lock().await.docs.remove(rep);
    Ok(())
}

fn insert_prims(
    scene: &mut MutexGuard<'_, WiredSceneApi>,
    quota: &Arc<Quota>,
    doc: &DocRes,
    ids: Vec<TreeID>,
) -> Result<Vec<u32>, QuotaError> {
    ids.into_iter()
        .map(|id| {
            scene.prims.insert(
                PrimRes {
                    doc: Arc::clone(&doc.doc),
                    doc_id: doc.id,
                    id,
                    is_proxy: false,
                },
                quota,
            )
        })
        .collect()
}

pub async fn roots(api: &Api, rep: u32) -> anyhow::Result<Vec<u32>> {
    let doc = get_doc(api, rep).await?;
    let tree = doc.doc.get_tree(&*HSD_CONTAINER_ID);
    let roots = tree.roots();
    let mut scene = api.wired_scene.lock().await;
    Ok(insert_prims(&mut scene, &api.quota, &doc, roots)?)
}

pub async fn prims(api: &Api, rep: u32) -> anyhow::Result<Vec<u32>> {
    let doc = get_doc(api, rep).await?;
    let tree = doc.doc.get_tree(&*HSD_CONTAINER_ID);
    let nodes = tree.nodes();
    let mut scene = api.wired_scene.lock().await;
    Ok(insert_prims(&mut scene, &api.quota, &doc, nodes)?)
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
    Ok(Some(scene.prims.insert(
        PrimRes {
            doc:      doc.doc,
            doc_id:   doc.id,
            id:       tree_id,
            is_proxy: false,
        },
        &api.quota,
    )?))
}

pub async fn create_prim(api: &Api, rep: u32) -> anyhow::Result<u32> {
    let doc = get_doc(api, rep).await?;
    validate_firewall(&api.doc_id, &doc.id, Channel::SceneWrite)?;
    api.quota
        .spend(Flow::CreatePrim, 1.0)
        .map_err(|err| anyhow::anyhow!("prim quota exceeded: {err:?}"))?;
    let quota = document_quota(doc.id);
    quota
        .try_charge(Stock::Prims, 1)
        .map_err(|err| anyhow::anyhow!("prim quota exceeded: {err:?}"))?;
    let tree = doc.doc.get_tree(&*HSD_CONTAINER_ID);
    let tree_id = tree
        .create(TreeParentId::Root)
        .inspect_err(|_| quota.release(Stock::Prims, 1))?;

    let mut scene = api.wired_scene.lock().await;
    match scene.prims.insert(
        PrimRes {
            doc:      doc.doc,
            doc_id:   doc.id,
            id:       tree_id,
            is_proxy: false,
        },
        &api.quota,
    ) {
        Ok(rep) => Ok(rep),
        Err(err) => {
            drop(scene);
            tree.delete(tree_id).ok();
            quota.release(Stock::Prims, 1);
            Err(err.into())
        }
    }
}

pub async fn offset_to(
    api: &Api,
    self_rep: u32,
    other_rep: u32,
) -> anyhow::Result<Option<XformValue>> {
    let self_doc = get_doc(api, self_rep).await?;
    let other_doc = get_doc(api, other_rep).await?;

    if !unavi_space::membership::same_space(self_doc.id, other_doc.id) {
        return Ok(None);
    }
    if validate_firewall(&api.doc_id, &other_doc.id, Channel::SceneRead).is_err() {
        return Ok(None);
    }

    let reg = DOC_ROOT_TRANSFORM_REGISTRY.read();
    let (Some(self_root), Some(other_root)) = (reg.get(&self_doc.id), reg.get(&other_doc.id))
    else {
        return Ok(None);
    };

    let relative = self_root.affine().inverse() * other_root.affine();
    let (scale, rotation, translation) =
        bevy::math::Mat4::from(relative).to_scale_rotation_translation();
    drop(reg);
    Ok(Some(XformValue {
        translation: [translation.x, translation.y, translation.z],
        rotation:    [rotation.x, rotation.y, rotation.z, rotation.w],
        scale:       [scale.x, scale.y, scale.z],
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
    let before = tree.nodes().len();
    tree.delete(prim.id)?;
    let removed = before.saturating_sub(tree.nodes().len()) as u64;
    document_quota(prim.doc_id).release(Stock::Prims, removed);
    Ok(())
}
