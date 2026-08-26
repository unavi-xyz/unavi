use std::{
    collections::HashMap,
    sync::LazyLock,
    time::Duration,
};

use hsd::id::DocId;
use parking_lot::{
    RwLock,
    RwLockReadGuard,
    RwLockWriteGuard,
};

use crate::{
    document::DocumentPolicy,
    reach::Reach,
};

/// Longest host chain a lookup will follow before giving up. A cycle is not
/// reachable through the derived-id scheme, so the cap guards a corrupted
/// registry rather than an expected case.
const MAX_HOST_DEPTH: usize = 16;

/// Everything the host has decided about one document.
///
/// One record rather than a registry per field: all of it is keyed by the same
/// document id and read together on the write path.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Record {
    pub policy: DocumentPolicy,
    pub reach:  Reach,
    /// The space this document was registered into, ignoring any that only a
    /// peer's pin places.
    pub space:  Option<DocId>,
    /// The document that composed this one in, for a prefab instance. An
    /// instance has an id but no namespace, so its owner and its space are
    /// whatever its host's are.
    pub host:   Option<DocId>,
}

static DOCUMENTS: LazyLock<RwLock<HashMap<DocId, Record>>> = LazyLock::new(RwLock::default);

/// Longest any caller may wait for the registry.
///
/// Every critical section here is a map lookup, so a wait this long means the
/// lock will never be granted: [`RwLock`] is not reentrant, and a read taken
/// inside a write on the same thread would otherwise hang the process with no
/// panic and no log.
const LOCK_TIMEOUT: Duration = Duration::from_secs(5);

fn documents() -> RwLockReadGuard<'static, HashMap<DocId, Record>> {
    DOCUMENTS
        .try_read_for(LOCK_TIMEOUT)
        .expect("policy registry read timed out; a write is held on this thread")
}

fn documents_mut() -> RwLockWriteGuard<'static, HashMap<DocId, Record>> {
    DOCUMENTS
        .try_write_for(LOCK_TIMEOUT)
        .expect("policy registry write timed out; a lock is held on this thread")
}

/// What the host decided about `doc`.
///
/// An unregistered document answers [`Record::default`]: silence and the
/// weakest answer are the same statement.
#[must_use]
pub fn get(doc: DocId) -> Record {
    documents().get(&doc).copied().unwrap_or_default()
}

pub fn update(doc: DocId, f: impl FnOnce(&mut Record)) {
    f(documents_mut().entry(doc).or_default());
}

pub fn forget(doc: DocId) {
    documents_mut().remove(&doc);
}

/// Drops the space's own record and unregisters its members from it.
///
/// The members keep their records: each drops its own when its document id
/// goes, and a member that outlives the space must not silently regain the
/// open defaults in the meantime.
pub fn forget_space(space: DocId) {
    let mut docs = documents_mut();
    docs.remove(&space);
    for record in docs.values_mut() {
        if record.space == Some(space) {
            record.space = None;
        }
    }
}

/// Every document registered into `space`.
#[must_use]
pub fn documents_in(space: DocId) -> Vec<DocId> {
    documents()
        .iter()
        .filter(|(_, record)| record.space == Some(space))
        .map(|(doc, _)| *doc)
        .collect()
}

/// The document at the top of `doc`'s host chain.
///
/// A prefab instance is never pinned and has no namespace, so it is its host
/// that owns it; resolving through the root is what stops a peer's instanced
/// content reading as locally authored.
#[must_use]
pub fn root(doc: DocId) -> DocId {
    let docs = documents();
    let mut at = doc;
    for _ in 0..MAX_HOST_DEPTH {
        match docs.get(&at).and_then(|record| record.host) {
            Some(host) if host != at => at = host,
            _ => break,
        }
    }
    at
}

/// The space `doc` was registered into, following the host chain.
#[must_use]
pub fn registered_space(doc: DocId) -> Option<DocId> {
    let root = root(doc);
    let docs = documents();
    docs.get(&doc)
        .and_then(|record| record.space)
        .or_else(|| docs.get(&root).and_then(|record| record.space))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(seed: &[u8]) -> DocId {
        DocId(*blake3::hash(seed).as_bytes())
    }

    #[test]
    fn an_unregistered_document_answers_the_weakest_record() {
        let record = get(doc(b"never-registered"));
        assert_eq!(record.policy, DocumentPolicy::untrusted());
        assert_eq!(record.reach, Reach::default());
        assert!(record.space.is_none());
    }

    #[test]
    fn a_space_takes_its_documents_registrations_with_it() {
        let (space, member, other) = (
            doc(b"forget-space"),
            doc(b"forget-member"),
            doc(b"forget-other"),
        );
        update(space, |r| r.space = Some(space));
        update(member, |r| r.space = Some(space));
        update(other, |r| r.reach = Reach::own_only());

        forget_space(space);

        assert!(get(member).space.is_none());
        assert_eq!(
            get(other).reach,
            Reach::own_only(),
            "unloading one space must not clear an unrelated document"
        );
        forget(other);
    }

    #[test]
    fn an_instance_resolves_through_its_host() {
        let (space, host, instance, nested) = (
            doc(b"root-space"),
            doc(b"root-host"),
            doc(b"root-instance"),
            doc(b"root-nested"),
        );
        update(host, |r| r.space = Some(space));
        update(instance, |r| r.host = Some(host));
        update(nested, |r| r.host = Some(instance));

        assert_eq!(root(nested), host);
        assert_eq!(
            registered_space(nested),
            Some(space),
            "a prefab inside a prefab stands where its host stands"
        );

        for d in [host, instance, nested] {
            forget(d);
        }
    }

    #[test]
    fn a_host_cycle_terminates() {
        let (a, b) = (doc(b"cycle-a"), doc(b"cycle-b"));
        update(a, |r| r.host = Some(b));
        update(b, |r| r.host = Some(a));

        let _ = root(a);

        forget(a);
        forget(b);
    }
}
