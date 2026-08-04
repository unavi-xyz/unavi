//! The bridge between iroh-docs entries and `SceneState`.
//!
//! Reading is one query per top-level prefix; there is no dependency pool to
//! keep consistent, because an entry's value is a blob hash by construction.

use hsd::{
    id::BlobId,
    key,
    package::Package,
    state::{
        SceneState,
        entry::{
            BulkRef,
            Entry,
            EntryValue,
        },
        save::Change,
    },
};
use iroh_blobs::api::blobs::Blobs;
use iroh_docs::api::Doc;
use wds::entries::{
    self,
    DocEntry,
    Write,
};

const PREFIXES: [&str; 3] = [key::META, key::PRIM_PREFIX, key::BULK_PREFIX];

/// Reads a whole document into fresh state.
///
/// A `p/` value is fetched because it is small and the scene cannot be built
/// without it; a `b/` value stays a reference, which is what makes deferring
/// bulk possible.
pub async fn read_state(doc: &Doc, blobs: &Blobs) -> anyhow::Result<SceneState> {
    let mut state = SceneState::new();
    for entry in entries::list(doc, &PREFIXES).await? {
        if let Some(entry) = to_entry(blobs, &entry).await {
            state.apply(&entry)?;
        }
    }
    Ok(state)
}

/// Converts a stored entry into one state can apply, fetching inline content.
///
/// Returns `None` when a `p/` value has not been downloaded yet; the
/// `ContentReady` event brings it back later.
pub async fn to_entry(blobs: &Blobs, entry: &DocEntry) -> Option<Entry> {
    let value = if entry.key.starts_with(key::BULK_PREFIX) {
        EntryValue::Blob(BulkRef {
            hash: BlobId(*entry.hash.as_bytes()),
            size: entry.size,
        })
    } else if entry.size == 0 {
        EntryValue::Bytes(Vec::new())
    } else {
        EntryValue::Bytes(entries::value(blobs, entry).await?.to_vec())
    };

    Some(Entry {
        key: entry.key.clone(),
        value,
        timestamp: entry.timestamp,
    })
}

/// Unpacks a compiled `.hsdz` into the blob store and a document's entries.
///
/// Bulk arrives inlined, so this is a straight walk with no fetching, no
/// nested-asset queue, and no wait-for-every-dependency state machine.
pub async fn unpack(package: Package, blobs: &Blobs) -> anyhow::Result<Vec<Write>> {
    let (inline, bulk) = hsd::package::split(package);

    let mut writes = Vec::with_capacity(inline.len() + bulk.len());
    for (key, value) in inline {
        writes.push(Write::Bytes {
            key,
            value: value.into(),
        });
    }
    for (key, value) in bulk {
        let size = value.len() as u64;
        let info = blobs.add_bytes(value).await?;
        writes.push(Write::Hash {
            key,
            hash: info.hash,
            size,
        });
    }
    Ok(writes)
}

/// Unpacks a package straight into state, for a prefab instance that has no
/// namespace and so no entries of its own.
pub async fn unpack_into_state(package: Package, blobs: &Blobs) -> anyhow::Result<SceneState> {
    let (inline, bulk) = hsd::package::split(package);

    let mut state = SceneState::new();
    for (key, value) in inline {
        state.apply(&Entry {
            key,
            value: EntryValue::Bytes(value),
            timestamp: 0,
        })?;
    }
    for (key, value) in bulk {
        let size = value.len() as u64;
        let info = blobs.add_bytes(value).await?;
        state.apply(&Entry {
            key,
            value: EntryValue::Blob(BulkRef {
                hash: BlobId(*info.hash.as_bytes()),
                size,
            }),
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
        Change::SetBlob { key, hash, size } => Write::Hash {
            key,
            hash: iroh_blobs::Hash::from_bytes(hash.0),
            size,
        },
        Change::Remove { key } => Write::Remove { key },
    }
}
