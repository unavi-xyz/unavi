//! The bridge between iroh-docs entries and `SceneState`.
//!
//! Reading is one query per top-level prefix; every entry's value is its byte
//! content, fetched eagerly because a document must be complete before it
//! realizes.

use hsd::{
    key,
    package::Package,
    state::{
        SceneState,
        entry::Entry,
        save::Change,
    },
};
use iroh_blobs::api::blobs::Blobs;
use iroh_docs::api::Doc;
use unavi_store::entries::{
    self,
    DocEntry,
    Write,
};

const PREFIXES: [&str; 2] = [key::META, key::PRIM_PREFIX];

pub async fn read_state(doc: &Doc, blobs: &Blobs) -> anyhow::Result<SceneState> {
    let mut state = SceneState::new();
    for entry in entries::list(doc, &PREFIXES).await? {
        if let Some(entry) = to_entry(blobs, &entry).await {
            state.apply(&entry)?;
        }
    }
    Ok(state)
}

/// Fetches a stored entry's content so a `SceneState` can apply it.
///
/// Returns `None` when a value has not been downloaded yet; the
/// `ContentReady` event brings it back later.
pub async fn to_entry(blobs: &Blobs, entry: &DocEntry) -> Option<Entry> {
    let value = if entry.size == 0 {
        Vec::new()
    } else {
        entries::value(blobs, entry).await?.to_vec()
    };

    Some(Entry {
        key: entry.key.clone(),
        value,
        timestamp: entry.timestamp,
    })
}

/// Unpacks a compiled `.hsdz` into a fresh document's writes. Package entries
/// carry inline bytes, so there is no fetching.
pub fn unpack(package: Package, _blobs: &Blobs) -> anyhow::Result<Vec<Write>> {
    Ok(package
        .entries
        .into_iter()
        .map(|(key, value)| Write::Bytes {
            key,
            value: value.into(),
        })
        .collect())
}

/// Unpacks a package straight into state, for a prefab instance that has no
/// namespace and so no entries of its own.
pub fn unpack_into_state(package: Package) -> anyhow::Result<SceneState> {
    let mut state = SceneState::new();
    for (key, value) in package.entries {
        state.apply(&Entry {
            key,
            value,
            timestamp: 0,
        })?;
    }
    Ok(state)
}

#[must_use]
pub fn to_write(change: Change) -> Write {
    match change {
        Change::Set { key, value } => Write::Bytes {
            key,
            value: value.into(),
        },
        Change::Remove { key } => Write::Remove { key },
    }
}
