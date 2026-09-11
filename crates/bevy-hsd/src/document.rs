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
use wds::document::Document;

const PREFIXES: [&str; 2] = [key::META, key::PRIM_PREFIX];

pub async fn read_state(doc: &Document) -> anyhow::Result<SceneState> {
    let mut state = SceneState::new();
    for entry in doc.list(&PREFIXES).await? {
        if let Some(entry) = to_entry(doc, &entry).await {
            state.apply(&entry)?;
        }
    }
    Ok(state)
}

/// Fetches a stored entry's content so a `SceneState` can apply it.
///
/// Returns `None` when a value has not been downloaded yet — the `ContentReady`
/// event brings it back later — or when the key is not UTF-8, which no key this
/// workspace writes ever is.
pub async fn to_entry(doc: &Document, entry: &iroh_docs::Entry) -> Option<Entry> {
    let key = String::from_utf8(entry.key().to_vec()).ok()?;
    let value = if entry.content_len() == 0 {
        Vec::new()
    } else {
        match doc.value(entry).await {
            Ok(Some(bytes)) => bytes.to_vec(),
            _ => return None,
        }
    };

    Some(Entry {
        key,
        value,
        timestamp: entry.timestamp(),
    })
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

/// Applies one `SceneState` change to the document backing it.
pub async fn apply_change(doc: &Document, change: Change) -> anyhow::Result<()> {
    match change {
        Change::Set { key, value } => {
            doc.set(key, value).await?;
        }
        Change::Remove { key } => {
            doc.remove(key).await?;
        }
    }
    Ok(())
}
