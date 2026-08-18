use std::sync::{
    Arc,
    Mutex,
};

use bevy::prelude::*;
use bevy_hsd::{
    HsdDocId,
    HsdPrimIndex,
    anchor::{
        self,
        DocAnchor,
    },
};
use hsd::{
    id::{
        DocId,
        PrimId,
    },
    state::SceneState,
};
use tokio::sync::MutexGuard;
use unavi_space::reach::{
    check_read,
    check_write,
};
use unavi_quota::{
    Flow,
    Quota,
    QuotaError,
    Stock,
};
use unavi_space::quota::document_quota;
use unavi_util::async_commands::AsyncCommands;

use crate::runtime::shared::{
    Api,
    registry::transform::DOC_ROOT_TRANSFORM_REGISTRY,
    wired::scene::{
        WiredSceneApi,
        prim::PrimRes,
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
    pub state: Arc<Mutex<SceneState>>,
    pub id:    DocId,
}

impl DocRes {
    fn with<T>(&self, f: impl FnOnce(&mut SceneState) -> T) -> anyhow::Result<T> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("scene state poisoned"))?;
        Ok(f(&mut state))
    }
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
    Ok(get_doc(api, rep).await?.id.0.to_vec())
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
    ids: Vec<PrimId>,
) -> Result<Vec<u32>, QuotaError> {
    ids.into_iter()
        .map(|id| {
            scene.prims.insert(
                PrimRes {
                    state: Arc::clone(&doc.state),
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
    let roots = doc.with(|state| state.roots())?;
    let mut scene = api.wired_scene.lock().await;
    Ok(insert_prims(&mut scene, &api.quota, &doc, roots)?)
}

pub async fn prims(api: &Api, rep: u32) -> anyhow::Result<Vec<u32>> {
    let doc = get_doc(api, rep).await?;
    let all = doc.with(|state| state.prims().collect::<Vec<_>>())?;
    let mut scene = api.wired_scene.lock().await;
    Ok(insert_prims(&mut scene, &api.quota, &doc, all)?)
}

pub async fn get_prim(api: &Api, rep: u32, prim_id: String) -> anyhow::Result<Option<u32>> {
    let doc = get_doc(api, rep).await?;
    let Ok(id) = prim_id.parse::<PrimId>() else {
        return Ok(None);
    };
    if !doc.with(|state| state.is_realized(id))? {
        return Ok(None);
    }
    let mut scene = api.wired_scene.lock().await;
    Ok(Some(scene.prims.insert(
        PrimRes {
            state: doc.state,
            doc_id: doc.id,
            id,
            is_proxy: false,
        },
        &api.quota,
    )?))
}

pub async fn create_prim(api: &Api, rep: u32) -> anyhow::Result<u32> {
    let doc = get_doc(api, rep).await?;
    check_write(api.doc_id, api.policy.tier, doc.id)?;
    api.quota.spend(Flow::CreatePrim, 1.0)?;
    let quota = document_quota(doc.id);
    quota.try_charge(Stock::Prims, 1)?;

    let id = doc.with(|state| state.create_prim(None))?;

    let mut scene = api.wired_scene.lock().await;
    match scene.prims.insert(
        PrimRes {
            state: Arc::clone(&doc.state),
            doc_id: doc.id,
            id,
            is_proxy: false,
        },
        &api.quota,
    ) {
        Ok(rep) => Ok(rep),
        Err(err) => {
            drop(scene);
            doc.with(|state| state.remove_prim(id))?;
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

    if check_read(self_doc.id, api.policy.tier, other_doc.id).is_err() {
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
    check_write(api.doc_id, api.policy.tier, prim.doc_id)?;

    let mut state = prim
        .state
        .lock()
        .map_err(|_| anyhow::anyhow!("scene state poisoned"))?;
    let before = state.prims().count();
    state.remove_prim(prim.id);
    let removed = before.saturating_sub(state.prims().count()) as u64;
    drop(state);

    document_quota(prim.doc_id).release(Stock::Prims, removed);
    Ok(())
}

/// Which half of a document's anchor a call sets. The other half is whatever
/// the document already had, so anchoring never disturbs an offset and
/// offsetting never disturbs an anchor.
pub enum Placement {
    Target(Option<(DocId, PrimId)>),
    Offset(Transform),
    /// Neither: enough to put a held document into the scene where its
    /// existing anchor already says it goes.
    Unchanged,
}

/// Puts a document into the scene, or moves one already in it.
///
/// Anchoring is per-peer runtime state, so it is applied to the world and
/// never written to the document.
pub fn place_document(world: &mut World, id: DocId, placement: Placement) -> anyhow::Result<()> {
    let doc_ent =
        find_doc(world, id).ok_or_else(|| anyhow::anyhow!("document {id} is not in the world"))?;
    let current = world.get::<DocAnchor>(doc_ent).copied();
    let offset = current.map_or_else(Transform::default, |anchor| anchor.offset);

    let anchor = match placement {
        Placement::Target(Some((target_doc, prim))) => DocAnchor {
            target: Some(find_prim(world, target_doc, prim).ok_or_else(|| {
                anyhow::anyhow!("anchor target prim {prim} of {target_doc} is not in the world")
            })?),
            offset,
        },
        Placement::Target(None) => DocAnchor::root(offset),
        Placement::Offset(offset) => DocAnchor {
            target: current.and_then(|anchor| anchor.target),
            offset,
        },
        Placement::Unchanged => current.unwrap_or_else(|| DocAnchor::root(offset)),
    };

    anchor::place(&mut world.entity_mut(doc_ent), anchor);
    Ok(())
}

async fn place(id: DocId, placement: Placement) -> anyhow::Result<()> {
    let (tx, rx) = async_channel::bounded(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            tx.try_send(place_document(world, id, placement)).ok();
        })
        .send()
        .await?;
    rx.recv().await?
}

pub async fn set_anchor(api: &Api, rep: u32, target: Option<u32>) -> anyhow::Result<()> {
    let doc = get_doc(api, rep).await?;
    check_write(api.doc_id, api.policy.tier, doc.id)?;

    let target = match target {
        Some(target_rep) => {
            let prim = api
                .wired_scene
                .lock()
                .await
                .prims
                .get(target_rep)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("invalid prim rep: {target_rep}"))?;
            Some((prim.doc_id, prim.id))
        }
        None => None,
    };

    place(doc.id, Placement::Target(target)).await
}

pub async fn set_offset(api: &Api, rep: u32, value: XformValue) -> anyhow::Result<()> {
    let doc = get_doc(api, rep).await?;
    check_write(api.doc_id, api.policy.tier, doc.id)?;

    place(
        doc.id,
        Placement::Offset(Transform {
            translation: Vec3::from_array(value.translation),
            rotation:    Quat::from_array(value.rotation),
            scale:       Vec3::from_array(value.scale),
        }),
    )
    .await
}

fn find_doc(world: &mut World, id: DocId) -> Option<Entity> {
    world
        .query::<(Entity, &HsdDocId)>()
        .iter(world)
        .find_map(|(e, d)| (d.0 == id).then_some(e))
}

fn find_prim(world: &mut World, doc: DocId, prim: PrimId) -> Option<Entity> {
    let doc_ent = find_doc(world, doc)?;
    world
        .get::<HsdPrimIndex>(doc_ent)
        .and_then(|index| index.0.get(&prim).copied())
}
