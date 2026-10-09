//! Creating, copying and deleting documents.

use std::sync::{
    Arc,
    Mutex,
};

use bevy::prelude::*;
use bevy_async::task;
use bevy_hsd::document::{
    DocIndex,
    Hsd,
    HsdDocId,
    HsdNamespace,
    Minted,
    Unplaced,
};
use bevy_iroh::store::DataStore;
use hsd::{
    id::DocId,
    key,
    state::{
        HsdState,
        entry::Entry,
    },
};
use iroh_docs::NamespaceId;
use unavi_policy::{
    permissions::HostApi,
    quota::{
        Flow,
        Stock,
        StockLease,
    },
};
use unavi_space::{
    grid::ActiveSpace,
    membership::{
        Space,
        SpaceId,
    },
};
use unavi_store::{
    Document,
    Store,
};

use crate::{
    error::ScriptError,
    host::{
        ScriptHost,
        scene::DocumentRes,
    },
    quota::QuotaLeases,
};

/// This node's store, `None` when it runs without one.
async fn store(host: &ScriptHost) -> Result<Store, ScriptError> {
    host.world_call(|world| world.get_resource::<DataStore>().map(|s| s.0.clone()))
        .await?
        .ok_or_else(|| ScriptError::Internal("no local store".into()))
}

/// Writes `entries` into the namespace `ns`. An empty value deletes its key.
pub async fn write_entries(
    host: &ScriptHost,
    ns: NamespaceId,
    entries: Vec<Entry>,
) -> Result<(), ScriptError> {
    let doc = store(host)
        .await?
        .held(ns)
        .await
        .map_err(ScriptError::internal)?
        .ok_or_else(|| ScriptError::Internal("the document is not held".into()))?;
    for entry in entries {
        let written = if entry.value.is_empty() {
            doc.remove_key(entry.key).await
        } else {
            doc.set(entry.key, entry.value).await.map(drop)
        };
        written.map_err(ScriptError::internal)?;
    }
    Ok(())
}

/// The namespace backing `id`. A reference site is keyed by a derived id but
/// backed by its target's namespace, so it answers with the target's.
pub async fn namespace_of(host: &ScriptHost, id: DocId) -> Result<NamespaceId, ScriptError> {
    host.world_call(move |world| {
        let entity = world.get_resource::<DocIndex>()?.get(id)?;
        world.get::<HsdNamespace>(entity).map(|ns| ns.0.id())
    })
    .await?
    .ok_or(ScriptError::NotFound)
}

pub async fn create_document(host: &mut ScriptHost) -> Result<u32, ScriptError> {
    host.require(HostApi::CreateDocument)?;
    mint(host, Vec::new()).await
}

/// Mints a document holding what `source` durably holds.
///
/// A copy, not a reference: the two diverge from here, and the copy's scripts
/// see the copy's prims, which is what a template is for. What `source`'s
/// scripts happen to be computing this frame is not part of it.
pub async fn copy_document(host: &mut ScriptHost, source: u32) -> Result<u32, ScriptError> {
    host.require(HostApi::CreateDocument)?;
    let id = host.document(source)?.id;
    let entries = source_entries(host, id).await?;
    mint(host, entries).await
}

/// Every entry the store behind `id` holds. A reference in the scene is keyed
/// by its site rather than the document it stands for, so those are searched
/// too.
async fn source_entries(host: &ScriptHost, id: DocId) -> Result<Vec<Entry>, ScriptError> {
    let doc = host
        .world_call(move |world| {
            super::doc_entity(world, id)
                .and_then(|entity| world.get::<HsdNamespace>(entity))
                .map(|ns| ns.0.clone())
        })
        .await?
        .ok_or(ScriptError::NotFound)?;
    read_entries(&doc).await
}

async fn read_entries(doc: &Document) -> Result<Vec<Entry>, ScriptError> {
    let mut entries = Vec::new();
    for prefix in key::PREFIXES {
        for entry in doc.list(prefix).await.map_err(ScriptError::internal)? {
            let Ok(key) = String::from_utf8(entry.key().to_vec()) else {
                continue;
            };
            let value = doc
                .value(&entry)
                .await
                .map_err(ScriptError::internal)?
                .ok_or(ScriptError::NotReady)?;
            entries.push(Entry::new(key, value, entry.timestamp()));
        }
    }
    Ok(entries)
}

/// Deletes the replica of a namespace whose document never reached the world,
/// unless [`Self::keep`] is called first.
struct ReplicaGuard {
    store: Store,
    ns:    Option<NamespaceId>,
}

impl ReplicaGuard {
    const fn keep(&mut self) {
        self.ns = None;
    }
}

impl Drop for ReplicaGuard {
    fn drop(&mut self) {
        let Some(ns) = self.ns else {
            return;
        };
        let store = self.store.clone();
        task::spawn(async move {
            if let Err(err) = store.remove(ns).await {
                debug!(%ns, ?err, "failed to remove an abandoned replica");
            }
        });
    }
}

/// Mints a namespace holding `entries` and spawns its document, held until
/// placed.
///
/// The document's quota unit is leased before anything is written, so a
/// script at its ceiling mints nothing, and a namespace whose document never
/// spawns is removed again.
async fn mint(host: &mut ScriptHost, entries: Vec<Entry>) -> Result<u32, ScriptError> {
    crate::quota::take(&host.quota, Flow::CreateDocument, 1)?;
    let lease = host.quota.lease(Stock::Documents, 1)?;

    let mut state = HsdState::new();
    for entry in &entries {
        state.project(entry).map_err(ScriptError::internal)?;
    }

    let store = store(host).await?;
    let doc = store.create().await.map_err(ScriptError::internal)?;
    let mut guard = ReplicaGuard {
        store,
        ns: Some(doc.id()),
    };
    write_entries(host, doc.id(), entries).await?;

    let id = DocId(*doc.id().as_bytes());
    let state = Arc::new(Mutex::new(state));
    spawn_minted(host, id, Arc::clone(&state), doc, lease).await?;
    guard.keep();

    host.minted.insert(id);
    Ok(host
        .documents
        .insert(DocumentRes { id, state }, &host.quota)?)
}

/// Seeds the document's policy record, serves and spawns it, and pins it into
/// its space.
async fn spawn_minted(
    host: &ScriptHost,
    id: DocId,
    state: Arc<Mutex<HsdState>>,
    doc: Document,
    lease: StockLease,
) -> Result<(), ScriptError> {
    // Seeded before the spawn applies, so the document is never one with no
    // composer, which policy would have to attribute by guessing. The host is
    // what its author and permissions resolve through.
    let space = child_space(
        host.view.policy().registered_space(host.doc),
        active_space(host).await,
    );
    host.view.policy().update(id, |record| {
        record.host = Some(host.doc);
        record.space = space;
    });
    host.view.policy().attribute_child_document(id, host.doc);

    // Serving answers a peer fetching the pin; without it the document is
    // local to this peer.
    if space.is_some()
        && let Err(err) = doc.serve().await
    {
        warn!(?err, %id, "failed to serve a script-minted document");
    }

    host.world_call(move |world| {
        world.spawn((
            Hsd(state),
            Unplaced,
            HsdDocId(id),
            HsdNamespace(doc),
            Minted,
            QuotaLeases::new(vec![lease]),
        ));
    })
    .await?;

    if let Some(space) = space
        && let Err(err) = host.view.local().pin(SpaceId::of_doc(space), id).await
    {
        warn!(?err, %id, "failed to pin a script-minted document");
    }
    Ok(())
}

/// Which space a minted document belongs to: its creator's, else the one the
/// local user stands in. The shell and its tools belong to no space, so what
/// they mint would otherwise belong nowhere.
fn child_space(host_space: Option<DocId>, active: Option<DocId>) -> Option<DocId> {
    host_space.or(active)
}

async fn active_space(host: &ScriptHost) -> Option<DocId> {
    host.world_call(|world| {
        let entity = world.get_resource::<ActiveSpace>()?.0?;
        world.get::<Space>(entity).map(Space::doc_id)
    })
    .await
    .ok()
    .flatten()
}

/// Removes a document this script created, and deletes this node's replica.
pub async fn delete_document(host: &mut ScriptHost, doc: u32) -> Result<(), ScriptError> {
    host.require(HostApi::Scene)?;
    let id = host.document(doc)?.id;
    if !host.minted.contains(&id) {
        return Err(ScriptError::Forbidden);
    }
    host.documents.remove(doc)?;
    host.minted.remove(&id);

    let store = host
        .world_call(move |world| {
            if let Some(entity) = world.get_resource::<DocIndex>().and_then(|i| i.get(id)) {
                world.despawn(entity);
            }
            world.get_resource::<DataStore>().map(|s| s.0.clone())
        })
        .await?;
    if let Some(store) = store
        && let Err(err) = store.remove(NamespaceId::from(&id.0)).await
    {
        debug!(%id, ?err, "failed to remove a deleted document's replica");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use hsd::id::DocId;

    use super::child_space;

    fn doc(seed: u8) -> DocId {
        DocId([seed; 32])
    }

    #[test]
    fn a_spaceless_creator_falls_back_to_the_active_space() {
        let space = doc(1);
        assert_eq!(child_space(None, Some(space)), Some(space));
    }

    #[test]
    fn a_creators_space_is_not_overridden() {
        let (host, active) = (doc(1), doc(2));
        assert_eq!(child_space(Some(host), Some(active)), Some(host));
    }
}
