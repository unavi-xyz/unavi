use std::sync::{
    Arc,
    Mutex,
};

use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdDocId,
    HsdHeld,
    HsdNamespace,
    document as hsd_document,
};
use bevy_iroh::store::{
    LocalBlobs,
    LocalDocs,
};
use hsd::{
    id::DocId,
    key,
    state::{
        SceneState,
        save,
    },
};
use iroh_docs::NamespaceId;
use unavi_policy::{
    check::{
        read as check_read,
        space_of,
        write as check_write,
    },
    registry,
    space::Space,
};
use unavi_quota::{
    Flow,
    Stock,
};
use unavi_space::anchor::ActiveSpace;
use unavi_util::{
    async_commands::AsyncCommands,
    async_task::spawn_async_task,
};

use crate::{
    error::ScriptError,
    quota::QuotaGuards,
    runtime::shared::{
        Api,
        slot_map::SlotMap,
        wired::scene::{
            document::DocRes,
            prim::PrimRes,
        },
    },
};

pub mod document;
pub mod prim;
pub mod util;

#[derive(Default)]
pub struct WiredSceneApi {
    pub docs:  SlotMap<DocRes>,
    pub prims: SlotMap<PrimRes>,
}

fn doc_id(bytes: &[u8]) -> anyhow::Result<DocId> {
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("document id must be 32 bytes"))?;
    Ok(DocId(arr))
}

/// Mints a namespace so a document has a stable id from birth. Portal
/// receptors and `wired:kv` keys are keyed by that id, so it must never be
/// remapped later — the cost is a `drop_doc` obligation on despawn.
async fn create_namespace() -> anyhow::Result<NamespaceId> {
    let (tx, rx) = async_channel::bounded(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let Some(docs) = world
                .query::<&LocalDocs>()
                .single(world)
                .ok()
                .map(|d| d.0.clone())
            else {
                return;
            };
            spawn_async_task(async move {
                tx.try_send(docs.api().create().await.map(|doc| doc.id()))
                    .ok();
            });
        })
        .send()
        .await?;
    rx.recv().await?
}

/// Enrols a namespace in the local sync set, so peers asking for it are
/// answered rather than told `NotFound`.
async fn serve_namespace(ns: NamespaceId) -> anyhow::Result<()> {
    let (tx, rx) = async_channel::bounded(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let Some(docs) = world
                .query::<&LocalDocs>()
                .single(world)
                .ok()
                .map(|d| d.0.clone())
            else {
                return;
            };
            spawn_async_task(async move {
                tx.try_send(unavi_store::namespace::serve(&docs, ns).await)
                    .ok();
            });
        })
        .send()
        .await?;
    rx.recv().await?
}

/// The namespace backing a document.
///
/// A prefab instance derives its id and has no namespace at all.
async fn namespace_of(id: DocId) -> anyhow::Result<NamespaceId> {
    let (tx, rx) = async_channel::bounded(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let ns = world
                .query::<(&HsdDocId, &HsdNamespace)>()
                .iter(world)
                .find_map(|(doc, ns)| (doc.0 == id).then_some(ns.0));
            tx.try_send(ns).ok();
        })
        .send()
        .await?;

    rx.recv()
        .await?
        .ok_or_else(|| anyhow::anyhow!("document has no namespace: {id}"))
}

/// Writes a document's live state into its entries.
///
/// Per-key diff against what the namespace already holds: only changed keys
/// are written, so two peers editing different prims do not overwrite each
/// other.
async fn save_namespace(ns: NamespaceId, state: Arc<Mutex<SceneState>>) -> anyhow::Result<()> {
    let current = state
        .lock()
        .map_err(|_| anyhow::anyhow!("scene state poisoned"))?
        .entries();

    let (tx, rx) = async_channel::bounded(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let Some((docs, blobs)) = world
                .query::<(&LocalDocs, &LocalBlobs)>()
                .single(world)
                .ok()
                .map(|(d, b)| (d.0.clone(), b.0.clone()))
            else {
                return;
            };
            spawn_async_task(async move {
                let res = async {
                    let doc = unavi_store::namespace::ensure_open(&docs, ns).await?;
                    let author = docs.api().author_default().await?;

                    let mut base = std::collections::BTreeMap::new();
                    for entry in
                        unavi_store::entries::list(&doc, &[key::META, key::PRIM_PREFIX]).await?
                    {
                        if let Some(entry) = hsd_document::to_entry(&blobs, &entry).await {
                            base.insert(entry.key, entry.value);
                        }
                    }

                    let writes = save::diff(&base, &current)
                        .into_iter()
                        .map(hsd_document::to_write);
                    unavi_store::entries::apply(&doc, &blobs, author, writes).await
                }
                .await;
                tx.try_send(res).ok();
            });
        })
        .send()
        .await?;
    rx.recv().await?
}

async fn spawn_child_doc(
    api: &Api,
    state: Arc<Mutex<SceneState>>,
    ns: NamespaceId,
) -> Result<(), ScriptError> {
    let doc_guard = api.quota.charge(Stock::Documents, 1)?;
    let id = DocId(*ns.as_bytes());

    // Seeded before the spawn command applies, so the child is never briefly
    // an unplaced document that policy would have to attribute by guessing.
    let parent = registry::get(api.doc_id);
    let space = registry::registered_space(api.doc_id);
    registry::update(id, |record| {
        record.policy = parent.policy;
        record.space = space;
    });

    unavi_quota::registry::child_document_quota(ns, NamespaceId::from(&api.doc_id.0));
    AsyncCommands::default()
        .spawn((
            HsdHeld(state),
            HsdDocId(id),
            HsdNamespace(ns),
            parent.policy,
            QuotaGuards(vec![doc_guard]),
        ))
        .send()
        .await
        .map_err(|err| ScriptError::other(err.to_string()))?;
    Ok(())
}

pub async fn self_prim(api: &Api) -> anyhow::Result<u32> {
    let mut scene = api.wired_scene.lock().await;
    Ok(scene.prims.insert(
        PrimRes {
            state:    Arc::clone(&api.state),
            doc_id:   api.doc_id,
            id:       api.prim,
            is_proxy: false,
        },
        &api.quota,
    )?)
}

pub async fn self_document(api: &Api) -> anyhow::Result<u32> {
    let mut scene = api.wired_scene.lock().await;
    Ok(scene.docs.insert(
        DocRes {
            state: Arc::clone(&api.state),
            id:    api.doc_id,
        },
        &api.quota,
    )?)
}

pub async fn get_document(api: &Api, id: Vec<u8>) -> anyhow::Result<Option<u32>> {
    let id = doc_id(&id)?;
    check_read(api.doc_id, id)?;

    let mut scene = api.wired_scene.lock().await;

    let existing = scene.docs.iter().find(|(_, v)| v.id == id).map(|(k, _)| k);
    if let Some(key) = existing {
        return Ok(scene.docs.insert_clone(key, &api.quota).transpose()?);
    }

    if id == api.doc_id {
        return Ok(Some(scene.docs.insert(
            DocRes {
                state: Arc::clone(&api.state),
                id,
            },
            &api.quota,
        )?));
    }

    let (tx, rx) = async_channel::bounded::<Option<Arc<Mutex<SceneState>>>>(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let state = world
                .query::<(&HsdDocId, AnyOf<(&Hsd, &HsdHeld)>)>()
                .iter(world)
                .find(|(rid, _)| rid.0 == id)
                .and_then(|(_, (live, held))| match (live, held) {
                    (Some(live), _) => Some(Arc::clone(&live.0)),
                    (None, Some(held)) => Some(Arc::clone(&held.0)),
                    (None, None) => None,
                });
            tx.try_send(state).ok();
        })
        .send()
        .await?;

    let Some(state) = rx.recv().await? else {
        return Ok(None);
    };
    Ok(Some(scene.docs.insert(DocRes { state, id }, &api.quota)?))
}

pub async fn remove_document(api: &Api, id: Vec<u8>) -> anyhow::Result<()> {
    let id = doc_id(&id)?;
    if let Err(err) = check_write(api.doc_id, id) {
        debug!(?err, "remove_document out of reach, skipping");
        return Ok(());
    }

    let mut scene = api.wired_scene.lock().await;
    let key = scene
        .docs
        .iter()
        .find(|(_, v)| v.id == id)
        .map(|(k, _)| k)
        .ok_or_else(|| anyhow::anyhow!("resource not found"))?;
    let Some(doc) = scene.docs.remove(key) else {
        return Ok(());
    };
    drop(scene);

    AsyncCommands::default()
        .push(move |world: &mut World| {
            let mut query = world.query::<(Entity, &HsdDocId)>();
            if let Some((entity, _)) = query.iter(world).find(|(_, v)| v.0 == doc.id) {
                world.despawn(entity);
            }
        })
        .send()
        .await?;

    // Minting a namespace at creation obligates dropping the replica;
    // otherwise scratch documents leak redb state.
    drop_replica(NamespaceId::from(&id.0)).await;

    Ok(())
}

async fn drop_replica(ns: NamespaceId) {
    let _ = AsyncCommands::default()
        .push(move |world: &mut World| {
            let Some(docs) = world
                .query::<&LocalDocs>()
                .single(world)
                .ok()
                .map(|d| d.0.clone())
            else {
                return;
            };
            spawn_async_task(async move {
                if let Err(err) = docs.api().drop_doc(ns).await {
                    debug!(%ns, ?err, "failed to drop document replica");
                }
            });
        })
        .send()
        .await;
}

pub async fn sync_document(api: &Api, id: Vec<u8>) -> anyhow::Result<()> {
    let id = doc_id(&id)?;
    let ns = namespace_of(id).await?;
    check_write(api.doc_id, id)?;
    crate::quota::acquire(&api.quota, Flow::SyncDoc, 1.0).await?;

    let space = if let Some(s) = space_of(id) {
        s
    } else {
        let (tx, rx) = async_channel::bounded::<Option<DocId>>(1);
        AsyncCommands::default()
            .push(move |world: &mut World| {
                let active = world.get_resource::<ActiveSpace>().and_then(|a| a.0);
                let id = active.and_then(|e| world.get::<Space>(e).map(Space::doc_id));
                tx.try_send(id).ok();
            })
            .send()
            .await?;
        rx.recv()
            .await?
            .ok_or_else(|| anyhow::anyhow!("doc has no space and no active space"))?
    };

    let state = {
        let scene = api.wired_scene.lock().await;
        scene
            .docs
            .iter()
            .find_map(|(_, d)| (d.id == id).then(|| Arc::clone(&d.state)))
    };
    let Some(state) = state else {
        anyhow::bail!("published doc not held by script");
    };
    save_namespace(ns, state).await?;

    // A document the space is asked to carry has to be in the space; one still
    // held goes in where its anchor already says it does.
    let (tx, rx) = async_channel::bounded(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            tx.try_send(document::place_document(
                world,
                id,
                document::Placement::Unchanged,
            ))
            .ok();
        })
        .send()
        .await?;
    rx.recv().await??;

    serve_namespace(ns).await?;

    // Ownership follows from the local pin; its quota is charged to the
    // resulting owner.
    if !unavi_space::state::entities::self_pin(NamespaceId::from(&space.0), ns).await {
        anyhow::bail!("space state not tracked locally or pin over quota");
    }

    Ok(())
}

pub async fn save_document(api: &Api, id: Vec<u8>) -> anyhow::Result<()> {
    let id = doc_id(&id)?;
    check_write(api.doc_id, id)?;

    let state = {
        let scene = api.wired_scene.lock().await;
        scene
            .docs
            .iter()
            .find_map(|(_, d)| (d.id == id).then(|| Arc::clone(&d.state)))
    };
    let Some(state) = state else {
        anyhow::bail!("saved doc not held by script");
    };
    save_namespace(namespace_of(id).await?, state).await
}

pub async fn create_document(api: &Api) -> Result<u32, ScriptError> {
    mint_document(api, SceneState::new()).await
}

/// Unpacks a prefab into a document with a namespace of its own.
pub async fn create_document_from_prefab(api: &Api, prefab: Vec<u8>) -> Result<u32, ScriptError> {
    let state = unpack_prefab(&prefab).map_err(|err| ScriptError::other(err.to_string()))?;

    mint_document(api, state).await
}

async fn mint_document(api: &Api, state: SceneState) -> Result<u32, ScriptError> {
    crate::quota::acquire(&api.quota, Flow::CreateDocument, 1.0).await?;

    let ns = create_namespace()
        .await
        .map_err(|err| ScriptError::other(err.to_string()))?;
    let state = Arc::new(Mutex::new(state));

    spawn_child_doc(api, Arc::clone(&state), ns).await?;

    let mut scene = api.wired_scene.lock().await;
    Ok(scene.docs.insert(
        DocRes {
            state,
            id: DocId(*ns.as_bytes()),
        },
        &api.quota,
    )?)
}

fn unpack_prefab(bytes: &[u8]) -> anyhow::Result<SceneState> {
    bevy_hsd::load::unpack_prefab(bytes)
}
