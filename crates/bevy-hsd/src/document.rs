//! The bridge between iroh-docs entries and `HsdState`.
//!
//! Reading is one query per top-level prefix; every entry's value is its byte
//! content, fetched eagerly because a document must be complete before it
//! realizes.

use bevy::log::warn;
use hsd::{
    key,
    state::{
        HsdState,
        entry::Entry,
        save::Change,
    },
};
use wds::document::Document;

pub async fn read_state(doc: &Document) -> anyhow::Result<HsdState> {
    let mut state = HsdState::new();
    for entry in doc.list(&key::PREFIXES).await? {
        if let Some(entry) = to_entry(doc, &entry).await
            && let Err(err) = state.apply(&entry)
        {
            warn!(key = %entry.key, ?err, "dropping an unreadable entry");
        }
    }
    Ok(state)
}

/// Fetches a stored entry's content so a `HsdState` can apply it.
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
        timestamp: entry.timestamp() / 1000,
    })
}

/// Applies one `HsdState` change to the document backing it.
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
