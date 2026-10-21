//! The tool side: an `edit` records what it is about to change, commits it,
//! and files it with the shell's history.

use std::{
    cell::RefCell,
    collections::BTreeMap,
};

use crate::{
    exports::unavi::author::api::GuestEdit,
    protocol::{
        CH_FILE,
        Change,
        Filed,
        MAX_LABEL_BYTES,
        PrimId,
    },
    snapshot::Snapshot,
    wired::{
        core::{
            error::Error,
            ids::DocumentId,
        },
        event::messaging::{
            self,
            Scope,
        },
        scene::{
            document::{
                Document,
                open_document,
            },
            properties::{
                Property,
                PropertyKey,
            },
        },
    },
};

/// What an edit has read so far, by document.
#[derive(Default)]
struct Tracked {
    sets:     Vec<(PrimId, PropertyKey, Option<Property>)>,
    created:  Vec<PrimId>,
    removed:  Vec<Snapshot>,
}

pub struct Recorder {
    label:   String,
    tracked: RefCell<BTreeMap<DocumentId, Tracked>>,
}

impl GuestEdit for Recorder {
    fn new(label: String) -> Result<Self, Error> {
        if label.len() > MAX_LABEL_BYTES {
            return Err(Error::InvalidArgument(format!(
                "label must be at most {MAX_LABEL_BYTES} bytes"
            )));
        }
        Ok(Self {
            label,
            tracked: RefCell::default(),
        })
    }

    fn track(&self, document: &Document, keys: Vec<(PrimId, PropertyKey)>) {
        let mut tracked = self.tracked.borrow_mut();
        let entry = tracked.entry(document.id()).or_default();
        for (prim, key) in keys {
            if entry.sets.iter().any(|(p, k, _)| *p == prim && *k == key) {
                continue;
            }
            let before = document.get(prim, &key);
            entry.sets.push((prim, key, before));
        }
    }

    fn track_creation(&self, document: &Document, prim: PrimId) -> Result<(), Error> {
        if !document.contains(prim) {
            return Err(Error::NotFound);
        }
        self.tracked
            .borrow_mut()
            .entry(document.id())
            .or_default()
            .created
            .push(prim);
        Ok(())
    }

    fn track_removal(&self, document: &Document, prim: PrimId) -> Result<(), Error> {
        let snapshot = Snapshot::capture(document, prim).ok_or(Error::NotFound)?;
        self.tracked
            .borrow_mut()
            .entry(document.id())
            .or_default()
            .removed
            .push(snapshot);
        Ok(())
    }

    /// A document that fails to commit does not stop the others: what did
    /// commit is filed, and the first failure is answered.
    fn commit(&self) -> Result<(), Error> {
        let tracked = self.tracked.take();
        let mut changes = Vec::new();
        let mut failed = None;
        for (id, tracked) in tracked {
            let committed = open_document(id)
                .and_then(|doc| doc.ok_or(Error::NotFound))
                .and_then(|doc| commit_document(&doc, tracked));
            match committed {
                Ok(committed) => changes.extend(committed),
                Err(err) => {
                    failed.get_or_insert(err);
                }
            }
        }
        file(&self.label, changes);
        failed.map_or(Ok(()), Err)
    }
}

/// Commits what was tracked on `doc` and answers it as changes.
fn commit_document(doc: &Document, tracked: Tracked) -> Result<Vec<Change>, Error> {
    let created = tracked
        .created
        .iter()
        .filter_map(|prim| Snapshot::capture(doc, *prim))
        .collect::<Vec<_>>();

    let mut keys = tracked
        .sets
        .iter()
        .map(|(prim, key, _)| (*prim, key.clone()))
        .collect::<Vec<_>>();
    for snapshot in &created {
        keys.extend(snapshot.keys());
    }
    for snapshot in &tracked.removed {
        keys.extend(snapshot.parent_keys());
    }
    doc.commit(&keys)?;

    let id = doc.id();
    let mut changes = Vec::new();
    for (prim, key, before) in tracked.sets {
        let after = doc.get(prim, &key);
        if after != before {
            changes.push(Change::Set {
                doc: id,
                prim,
                key,
                before: before.map(Box::new),
                after: after.map(Box::new),
            });
        }
    }
    changes.extend(created.into_iter().map(|prims| Change::Create { doc: id, prims }));
    changes.extend(
        tracked
            .removed
            .into_iter()
            .map(|prims| Change::Remove { doc: id, prims }),
    );
    Ok(changes)
}

/// Sends `changes` to the history. An edit too large for one message is kept
/// but not undoable, and says so.
fn file(label: &str, changes: Vec<Change>) {
    if changes.is_empty() {
        return;
    }
    let payload = match postcard::to_allocvec(&Filed {
        label: label.to_owned(),
        changes,
    }) {
        Ok(payload) => payload,
        Err(err) => {
            eprintln!("author: encode '{label}': {err}");
            return;
        }
    };
    if let Err(err) = messaging::emit(CH_FILE, &payload, None, Scope::Global) {
        eprintln!("author: '{label}' is kept but cannot be undone: {err:?}");
    }
}
