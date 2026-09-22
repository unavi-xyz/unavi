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
    HsdSource,
    document as hsd_document,
};
use bevy_iroh::store::LocalStore;
use hsd::{
    id::DocId,
    key,
    state::{
        SceneState,
        entry::Entry,
        save,
    },
};
use iroh_docs::NamespaceId;
use unavi_policy::{
    quota::{
        Flow,
        Stock,
    },
    space::Space,
};
use unavi_space::anchor::ActiveSpace;
use unavi_util::{
    async_commands::AsyncCommands,
    async_task::spawn_async_task,
};
use wds::document::Document;

use crate::{
    error::ScriptError,
    quota::QuotaLeases,
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
async fn create_namespace() -> anyhow::Result<Document> {
    let (tx, rx) = async_channel::bounded(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let Some(store) = world
                .query::<&LocalStore>()
                .single(world)
                .ok()
                .map(|s| s.0.clone())
            else {
                return;
            };
            spawn_async_task(async move {
                tx.try_send(store.create().await).ok();
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
            let Some(store) = world
                .query::<&LocalStore>()
                .single(world)
                .ok()
                .map(|s| s.0.clone())
            else {
                return;
            };
            spawn_async_task(async move {
                let served = async { store.open(ns).await?.serve().await };
                tx.try_send(served.await).ok();
            });
        })
        .send()
        .await?;
    rx.recv().await?
}

/// The namespace backing a document.
///
/// A reference site is keyed by a derived id but backed by the target's
/// namespace, so it answers with the target's.
async fn namespace_of(id: DocId) -> anyhow::Result<NamespaceId> {
    let (tx, rx) = async_channel::bounded(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let ns = world
                .query::<(&HsdDocId, &HsdNamespace)>()
                .iter(world)
                .find_map(|(doc, ns)| (doc.0 == id).then_some(ns.0.id()));
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
            let Some(store) = world
                .query::<&LocalStore>()
                .single(world)
                .ok()
                .map(|s| s.0.clone())
            else {
                return;
            };
            spawn_async_task(async move {
                let res = async {
                    let doc = store.open(ns).await?;

                    let mut base = std::collections::BTreeMap::new();
                    for entry in doc.list(&[key::META, key::PRIM_PREFIX]).await? {
                        if let Some(entry) = hsd_document::to_entry(&doc, &entry).await {
                            base.insert(entry.key, entry.value);
                        }
                    }

                    for change in save::diff(&base, &current) {
                        hsd_document::apply_change(&doc, change).await?;
                    }
                    anyhow::Ok(())
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
    doc: Document,
) -> Result<(), ScriptError> {
    let doc_lease = api.quota.lease(Stock::Documents, 1)?;
    let id = DocId(*doc.id().as_bytes());

    // Seeded before the spawn command applies, so the child is never briefly
    // a document with no composer, which policy would have to attribute by
    // guessing. The host is what its author and its permissions resolve
    // through, so a script's child runs with the grant of whoever wrote the
    // script.
    let space = api.view.policy().registered_space(api.doc_id);
    api.view.policy().update(id, |record| {
        record.host = Some(api.doc_id);
        record.space = space;
    });

    api.view.policy().attribute_child_document(id, api.doc_id);
    AsyncCommands::default()
        .spawn((
            HsdHeld(state),
            HsdDocId(id),
            HsdNamespace(doc),
            QuotaLeases(vec![doc_lease]),
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

    // Minting a namespace obligates removing the replica;
    // otherwise scratch documents leak redb state.
    remove_replica(NamespaceId::from(&id.0)).await;

    Ok(())
}

async fn remove_replica(ns: NamespaceId) {
    let _ = AsyncCommands::default()
        .push(move |world: &mut World| {
            let Some(store) = world
                .query::<&LocalStore>()
                .single(world)
                .ok()
                .map(|s| s.0.clone())
            else {
                return;
            };
            spawn_async_task(async move {
                if let Err(err) = store.remove(ns).await {
                    debug!(%ns, ?err, "failed to remove document replica");
                }
            });
        })
        .send()
        .await;
}

pub async fn sync_document(api: &Api, id: Vec<u8>) -> anyhow::Result<()> {
    let id = doc_id(&id)?;
    let ns = namespace_of(id).await?;
    crate::quota::acquire(&api.quota, Flow::SyncDoc, 1.0).await?;

    let space = if let Some(s) = api.view.space_of(id) {
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
    if !api.view.self_pin(space, DocId(*ns.as_bytes())).await {
        anyhow::bail!("space state not tracked locally or pin over quota");
    }

    Ok(())
}

pub async fn save_document(api: &Api, id: Vec<u8>) -> anyhow::Result<()> {
    let id = doc_id(&id)?;

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

/// Mints an independent document holding what `id` has authored.
///
/// A copy, not a reference: the two diverge from here, and the copy's scripts
/// see the copy's prims. That is what a template is for, and what a reference
/// deliberately is not — a reference realizes the target as a child document,
/// so a script inside it would look for its siblings in the wrong place.
///
/// Only the document layer travels. A copy of what someone's script happened
/// to be computing this frame is not what "copy this template" means.
pub async fn copy_document(api: &Api, id: Vec<u8>) -> Result<u32, ScriptError> {
    let id = doc_id(&id).map_err(|err| ScriptError::other(err.to_string()))?;
    let entries = source_entries(api, id)
        .await
        .map_err(|err| ScriptError::other(err.to_string()))?
        .ok_or_else(|| ScriptError::other(format!("no document {id} to copy")))?;

    let mut state = SceneState::new();
    for (key, value) in entries {
        state
            .apply(&Entry {
                key,
                value,
                timestamp: 0,
            })
            .map_err(|err| ScriptError::other(err.to_string()))?;
    }

    mint_document(api, state).await
}

/// The authored entries of `id`, looked up by document id and then by the
/// reference sites realizing it, since a realized reference is keyed by its
/// site rather than by the document it stands for.
async fn source_entries(
    _api: &Api,
    id: DocId,
) -> anyhow::Result<Option<std::collections::BTreeMap<String, Vec<u8>>>> {
    let (tx, rx) = async_channel::bounded(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let by_id = world
                .query::<(&HsdDocId, &Hsd)>()
                .iter(world)
                .find(|(doc, _)| doc.0 == id)
                .map(|(_, live)| Arc::clone(&live.0));
            let state = by_id.or_else(|| {
                world
                    .query::<(&HsdSource, &Hsd)>()
                    .iter(world)
                    .find(|(source, _)| source.0 == id)
                    .map(|(_, live)| Arc::clone(&live.0))
            });
            let entries = state.and_then(|state| state.lock().ok().map(|s| s.entries()));
            tx.try_send(entries).ok();
        })
        .send()
        .await?;
    Ok(rx.recv().await?)
}

async fn mint_document(api: &Api, state: SceneState) -> Result<u32, ScriptError> {
    crate::quota::acquire(&api.quota, Flow::CreateDocument, 1.0).await?;

    let doc = create_namespace()
        .await
        .map_err(|err| ScriptError::other(err.to_string()))?;
    let ns = doc.id();
    let state = Arc::new(Mutex::new(state));

    spawn_child_doc(api, Arc::clone(&state), doc).await?;

    let mut scene = api.wired_scene.lock().await;
    Ok(scene.docs.insert(
        DocRes {
            state,
            id: DocId(*ns.as_bytes()),
        },
        &api.quota,
    )?)
}
