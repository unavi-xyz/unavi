use std::sync::{
    Arc,
    Mutex,
};

use bevy::prelude::*;
use bevy_hsd::{
    HsdDocId,
    HsdPrimIndex,
    anchor::DocAnchor,
};
use hsd::{
    id::{
        DocId,
        PrimId,
    },
    state::SceneState,
};
use tokio::sync::MutexGuard;
use unavi_quota::{
    Flow,
    Quota,
    QuotaError,
    Stock,
};
use unavi_space::quota::document_quota;
use unavi_util::async_commands::AsyncCommands;

use crate::{
    firewall::Channel,
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
    validate_firewall(&api.doc_id, &doc.id, Channel::SceneWrite)?;
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

/// Anchoring is per-peer runtime state, so it is applied to the world and
/// never written to the document.
pub async fn set_anchor(api: &Api, rep: u32, target: Option<u32>) -> anyhow::Result<()> {
    let doc = get_doc(api, rep).await?;
    validate_firewall(&api.doc_id, &doc.id, Channel::SceneWrite)?;

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

    let id = doc.id;
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let Some(doc_ent) = find_doc(world, id) else {
                return;
            };
            let target_ent = target.and_then(|(doc, prim)| find_prim(world, doc, prim));
            if target.is_some() && target_ent.is_none() {
                return;
            }
            let offset = world
                .get::<DocAnchor>(doc_ent)
                .map_or_else(Transform::default, |a| a.offset);
            world.entity_mut(doc_ent).insert(DocAnchor {
                target: target_ent,
                offset,
            });
        })
        .send()
        .await?;
    Ok(())
}

pub async fn set_offset(api: &Api, rep: u32, value: XformValue) -> anyhow::Result<()> {
    let doc = get_doc(api, rep).await?;
    validate_firewall(&api.doc_id, &doc.id, Channel::SceneWrite)?;

    let id = doc.id;
    let offset = Transform {
        translation: Vec3::from_array(value.translation),
        rotation:    Quat::from_array(value.rotation),
        scale:       Vec3::from_array(value.scale),
    };
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let Some(doc_ent) = find_doc(world, id) else {
                return;
            };
            let target = world.get::<DocAnchor>(doc_ent).and_then(|a| a.target);
            world
                .entity_mut(doc_ent)
                .insert(DocAnchor { target, offset });
        })
        .send()
        .await?;
    Ok(())
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
