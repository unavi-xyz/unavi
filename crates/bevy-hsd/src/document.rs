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
use unavi_store::namespace::Namespace;

const PREFIXES: [&str; 2] = [key::META, key::PRIM_PREFIX];

pub async fn read_state(ns: &Namespace) -> anyhow::Result<SceneState> {
    let mut state = SceneState::new();
    for entry in ns.list(&PREFIXES).await? {
        if let Some(entry) = to_entry(ns, &entry).await {
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
pub async fn to_entry(ns: &Namespace, entry: &iroh_docs::Entry) -> Option<Entry> {
    let key = String::from_utf8(entry.key().to_vec()).ok()?;
    let value = if entry.content_len() == 0 {
        Vec::new()
    } else {
        ns.value(entry).await?.to_vec()
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
pub async fn apply_change(ns: &Namespace, change: Change) -> anyhow::Result<()> {
    match change {
        Change::Set { key, value } => {
            ns.set(key, value).await?;
        }
        Change::Remove { key } => {
            ns.remove(key).await?;
        }
    }
    Ok(())
}
