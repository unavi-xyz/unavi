use std::sync::{
    Arc,
    Mutex,
};

use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdChild,
    HsdDocId,
    HsdPrimIndex,
    Prim,
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
    property::name::PropName,
    state::{
        CommitTarget,
        HsdState,
    },
};
use tokio::sync::MutexGuard;
use unavi_policy::quota::{
    Flow,
    Quota,
    QuotaError,
    Stock,
};
use unavi_space::quota::document_quota;
use unavi_util::async_commands::AsyncCommands;

use crate::runtime::shared::{
    Api,
    wired::scene::{
        WiredSceneApi,
        holds_write_key,
        namespace_of,
        prim::PrimRes,
        write_entries,
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
    pub state: Arc<Mutex<HsdState>>,
    pub id:    DocId,
}

impl DocRes {
    fn with<T>(&self, f: impl FnOnce(&mut HsdState) -> T) -> anyhow::Result<T> {
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
    crate::quota::acquire(&api.quota, Flow::CreatePrim, 1.0).await?;
    let quota = document_quota(
        api.view.policy(),
        api.view.replicas(),
        Some(api.view.viewer()),
        doc.id,
    );
    quota.charge(Stock::Prims, 1)?;

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

    let (Some(self_root), Some(other_root)) = (
        api.transforms.doc_root(&self_doc.id),
        api.transforms.doc_root(&other_doc.id),
    ) else {
        return Ok(None);
    };

    let relative = self_root.affine().inverse() * other_root.affine();
    let (scale, rotation, translation) =
        bevy::math::Mat4::from(relative).to_scale_rotation_translation();
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

    let mut state = prim
        .state
        .lock()
        .map_err(|_| anyhow::anyhow!("scene state poisoned"))?;
    let before = state.prims().count();
    state.remove_prim(prim.id);
    let removed = before.saturating_sub(state.prims().count()) as u64;
    drop(state);

    document_quota(
        api.view.policy(),
        api.view.replicas(),
        Some(api.view.viewer()),
        prim.doc_id,
    )
    .release(Stock::Prims, removed);
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

/// Where a commit against a document lands, and what it takes to get there.
enum Landing {
    /// This client holds the document's own write key.
    Document,
    /// It holds the key of a document referencing it, so the promotion is an
    /// override there rather than an edit of content authored elsewhere.
    Override {
        site:      PrimId,
        reference: DocId,
        state:     Arc<Mutex<HsdState>>,
    },
    /// It holds neither. The edit is still useful — visible to everyone
    /// present, attributed, and gone when they leave.
    Session,
}

/// Promotes live opinions on this document into the strongest durable layer
/// this client can write.
///
/// A no-op unless this peer holds the *calling* document. Scripts run on every
/// peer, so a tool's script calls commit on every peer, and a guest's edit
/// must not be committed by the room owner's copy of the same script.
/// `holder` is the latest claim else the document's author, so it is always
/// defined where the calling document has a space at all; one with no space
/// cannot be committed yet.
pub async fn commit(api: &Api, rep: u32, props: Vec<(String, String)>) -> anyhow::Result<()> {
    let doc = get_doc(api, rep).await?;

    let me = api.view.me();
    let Some(space) = api.view.space_of(api.doc_id) else {
        return Ok(());
    };
    if !api.view.replicas().is_holder(space, api.doc_id, me) {
        return Ok(());
    }

    let props = props
        .into_iter()
        .map(|(prim, name)| {
            let prim = prim
                .parse::<PrimId>()
                .map_err(|err| anyhow::anyhow!("invalid prim id: {err}"))?;
            let name = name
                .parse::<PropName>()
                .map_err(|err| anyhow::anyhow!("invalid property name: {err}"))?;
            Ok((prim, name))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let landing = landing(doc.id).await?;
    let entries = doc.with(|state| {
        state.commit(
            match &landing {
                Landing::Document => CommitTarget::Document,
                Landing::Override { site, .. } => CommitTarget::Override { site: *site },
                Landing::Session => CommitTarget::Session,
            },
            &props,
        )
    })?;

    // The commit already projected its entries where they compose. Writing
    // them is what makes them durable, and the session layer has no store.
    match landing {
        Landing::Document => {
            write_entries(namespace_of(doc.id).await?, entries).await?;
        }
        Landing::Override {
            reference, state, ..
        } => {
            {
                let mut referencing = state
                    .lock()
                    .map_err(|_| anyhow::anyhow!("scene state poisoned"))?;
                for entry in &entries {
                    referencing.project(entry)?;
                }
            }
            write_entries(namespace_of(reference).await?, entries).await?;
        }
        Landing::Session => {}
    }
    Ok(())
}

/// Resolves where a commit against `doc` can land: its own key, else the key
/// of the document referencing it, else nothing durable.
///
/// The walk is one hop by construction. An override key names a site prim and
/// a target prim, so a document can only state an opinion about what it
/// references directly — a room referencing a couch that references a cushion
/// has no way to name the cushion's prims.
async fn landing(doc: DocId) -> anyhow::Result<Landing> {
    if holds_write_key(doc).await? {
        return Ok(Landing::Document);
    }
    let Some((site, reference, state)) = reference_site(doc).await? else {
        return Ok(Landing::Session);
    };
    if !holds_write_key(reference).await? {
        return Ok(Landing::Session);
    }
    Ok(Landing::Override {
        site,
        reference,
        state,
    })
}

/// The prim a realized reference hangs from, and the document holding it.
///
/// A realized reference is spawned as a child of the prim that names it, so
/// that prim answers both which key an override takes and whose document it
/// belongs in.
async fn reference_site(
    doc: DocId,
) -> anyhow::Result<Option<(PrimId, DocId, Arc<Mutex<HsdState>>)>> {
    let (tx, rx) = async_channel::bounded(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            tx.try_send(reference_site_in(world, doc)).ok();
        })
        .send()
        .await?;
    Ok(rx.recv().await?)
}

fn reference_site_in(
    world: &mut World,
    doc: DocId,
) -> Option<(PrimId, DocId, Arc<Mutex<HsdState>>)> {
    let prim_ent = world
        .query::<(&HsdDocId, &ChildOf)>()
        .iter(world)
        .find_map(|(id, parent)| (id.0 == doc).then_some(parent.0))?;
    let site = world.get::<Prim>(prim_ent)?.0;
    let host = world.get::<HsdChild>(prim_ent)?.0;
    Some((
        site,
        world.get::<HsdDocId>(host)?.0,
        Arc::clone(&world.get::<Hsd>(host)?.0),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(n: u8) -> DocId {
        DocId([n; 32])
    }

    /// A document, a prim of it, and the reference that prim realizes, spawned
    /// the way `load::realize_ref` spawns them.
    fn realized_reference(world: &mut World) -> (PrimId, DocId, DocId) {
        let host = doc(1);
        let host_ent = world
            .spawn((Hsd::new(HsdState::new()), HsdDocId(host)))
            .id();
        let site = PrimId::new();
        let prim_ent = world.spawn((Prim(site), HsdChild(host_ent))).id();
        let child = DocId::site(host, site);
        world.spawn((
            Hsd::new(HsdState::new()),
            HsdDocId(child),
            ChildOf(prim_ent),
        ));
        (site, host, child)
    }

    #[test]
    fn a_realized_reference_answers_the_prim_and_document_holding_it() {
        let mut world = World::new();
        let (site, host, child) = realized_reference(&mut world);

        let (found_site, found_host, _) =
            reference_site_in(&mut world, child).expect("the site is reachable from the child");

        assert_eq!(found_site, site, "which key an override takes");
        assert_eq!(found_host, host, "and whose document it belongs in");
    }

    #[test]
    fn a_document_nothing_references_has_no_site() {
        let mut world = World::new();
        realized_reference(&mut world);
        let anchored = doc(2);
        world.spawn((Hsd::new(HsdState::new()), HsdDocId(anchored)));

        assert!(
            reference_site_in(&mut world, anchored).is_none(),
            "a document placed in a space is not a reference site, so a commit \
             against it has no override to write"
        );
    }
}
