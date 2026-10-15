//! The newest pin record the local peer knows for each document, in two tiers.
//!
//! Known records belong to documents that have, or had, presence here. They
//! outlive the presence, so a recreated one cannot regress to an older record,
//! and a space holds at most [`MAX_SNAPSHOT_DOCS`]. Hearsay records belong to
//! documents with no presence: each source holds a few, they go when the
//! source disconnects, and they become known when the document gains presence.

use std::collections::{
    HashMap,
    HashSet,
};

use hsd::id::DocId;
use iroh::EndpointId;

use crate::{
    membership::SpaceId,
    replication::{
        message::MAX_SNAPSHOT_DOCS,
        record::Record,
        store::{
            Inner,
            Replicas,
        },
    },
};

/// Records one source may leave for documents with no presence here.
const HEARSAY_PER_SOURCE: usize = 64;

struct Held {
    record: Record,
    /// Arrival order, which decides what is evicted first.
    seq:    u64,
}

#[derive(Default)]
pub(super) struct Records {
    known:   HashMap<DocId, Held>,
    counts:  HashMap<SpaceId, usize>,
    hearsay: HashMap<EndpointId, HashMap<DocId, Held>>,
    next:    u64,
}

impl Records {
    /// The newest record held for `doc` in either tier.
    fn get(&self, doc: DocId) -> Option<&Record> {
        let heard = self
            .hearsay
            .values()
            .filter_map(|docs| docs.get(&doc))
            .map(|held| &held.record);
        self.known
            .get(&doc)
            .map(|held| &held.record)
            .into_iter()
            .chain(heard)
            .reduce(|a, b| if b.supersedes(a) { b } else { a })
    }

    fn is_known(&self, doc: DocId) -> bool {
        self.known.contains_key(&doc)
    }

    const fn stamp(&mut self) -> u64 {
        let seq = self.next;
        self.next += 1;
        seq
    }

    /// Keeps `record` as `doc`'s known record. Past the space's cap, the
    /// oldest arrival goes, a document with no presence before one with.
    fn insert_known(&mut self, record: Record, present: impl Fn(DocId) -> bool) {
        let (doc, space) = (record.payload.doc, record.payload.space);
        for docs in self.hearsay.values_mut() {
            docs.remove(&doc);
        }
        self.hearsay.retain(|_, docs| !docs.is_empty());
        let seq = self.stamp();
        if let Some(old) = self.known.insert(doc, Held { record, seq }) {
            self.uncount(old.record.payload.space);
        }
        *self.counts.entry(space).or_default() += 1;
        if self.counts.get(&space).copied().unwrap_or(0) > MAX_SNAPSHOT_DOCS {
            self.evict(space, doc, present);
        }
    }

    fn evict(&mut self, space: SpaceId, keep: DocId, present: impl Fn(DocId) -> bool) {
        let oldest = self
            .known
            .iter()
            .filter(|(doc, held)| held.record.payload.space == space && **doc != keep)
            .min_by_key(|(doc, held)| (present(**doc), held.seq))
            .map(|(doc, _)| *doc);
        if let Some(doc) = oldest {
            self.remove(doc);
        }
    }

    /// Keeps `record`, for a document with no presence, under `source`.
    fn insert_hearsay(&mut self, source: EndpointId, record: Record) {
        let seq = self.stamp();
        let docs = self.hearsay.entry(source).or_default();
        docs.insert(record.payload.doc, Held { record, seq });
        if docs.len() > HEARSAY_PER_SOURCE {
            let oldest = docs
                .iter()
                .min_by_key(|(_, held)| held.seq)
                .map(|(doc, _)| *doc);
            if let Some(doc) = oldest {
                docs.remove(&doc);
            }
        }
    }

    /// Makes the newest hearsay for `doc` known, now that it has presence.
    fn promote(&mut self, doc: DocId, present: impl Fn(DocId) -> bool) {
        let best = self
            .hearsay
            .values()
            .filter_map(|docs| docs.get(&doc))
            .map(|held| &held.record)
            .reduce(|a, b| if b.supersedes(a) { b } else { a })
            .cloned();
        let Some(best) = best else {
            return;
        };
        if self
            .known
            .get(&doc)
            .is_none_or(|held| best.supersedes(&held.record))
        {
            self.insert_known(best, present);
        } else {
            for docs in self.hearsay.values_mut() {
                docs.remove(&doc);
            }
            self.hearsay.retain(|_, docs| !docs.is_empty());
        }
    }

    pub(super) fn remove(&mut self, doc: DocId) {
        if let Some(old) = self.known.remove(&doc) {
            self.uncount(old.record.payload.space);
        }
    }

    fn uncount(&mut self, space: SpaceId) {
        if let Some(count) = self.counts.get_mut(&space) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.counts.remove(&space);
            }
        }
    }

    fn drop_source(&mut self, source: EndpointId) {
        self.hearsay.remove(&source);
    }

    fn forget_space(&mut self, space: SpaceId) {
        self.known
            .retain(|_, held| held.record.payload.space != space);
        self.counts.remove(&space);
        for docs in self.hearsay.values_mut() {
            docs.retain(|_, held| held.record.payload.space != space);
        }
        self.hearsay.retain(|_, docs| !docs.is_empty());
    }

    /// The newest record of each document whose newest is a withdrawal.
    pub(super) fn withdrawn(&self) -> Vec<&Record> {
        let docs = self
            .known
            .keys()
            .chain(self.hearsay.values().flat_map(HashMap::keys))
            .collect::<HashSet<_>>();
        docs.into_iter()
            .filter_map(|doc| self.get(*doc))
            .filter(|record| !record.payload.pinned)
            .collect()
    }

    #[cfg(test)]
    fn known_in(&self, space: SpaceId) -> usize {
        self.counts.get(&space).copied().unwrap_or(0)
    }

    #[cfg(test)]
    fn heard_from(&self, source: EndpointId) -> usize {
        self.hearsay.get(&source).map_or(0, HashMap::len)
    }
}

impl Inner {
    /// Whether the local peer is in `space`.
    pub(super) fn in_space(&self, space: SpaceId) -> bool {
        self.attribution.policy().get(space.doc()).space == Some(space.doc())
    }

    pub(super) fn newest_record(&self, doc: DocId) -> Option<&Record> {
        self.records.get(doc)
    }

    /// Keeps `record`, heard from `source`, if it is newer than every record
    /// held for its document. A document with no presence needs the local peer
    /// to be in the record's space.
    pub(super) fn observe(&mut self, record: Record, source: EndpointId) {
        let (doc, space) = (record.payload.doc, record.payload.space);
        let presence = self.docs.get(&doc).map(|p| p.space);
        if presence.is_some_and(|present| present != space) {
            return;
        }
        if presence.is_none() && !self.in_space(space) {
            return;
        }
        if self
            .records
            .get(doc)
            .is_some_and(|known| !record.supersedes(known))
        {
            return;
        }
        if presence.is_some() || self.records.is_known(doc) {
            let docs = &self.docs;
            self.records
                .insert_known(record, |doc| docs.contains_key(&doc));
        } else {
            self.records.insert_hearsay(source, record);
        }
    }

    /// Makes what was heard of `doc` known, now that it has presence.
    pub(super) fn promote_records(&mut self, doc: DocId) {
        let docs = &self.docs;
        self.records.promote(doc, |doc| docs.contains_key(&doc));
    }

    /// Keeps `doc`'s record past its presence, unless the local peer is not in
    /// its space.
    pub(super) fn settle_records(&mut self, doc: DocId, space: SpaceId) {
        if !self.in_space(space) {
            self.records.remove(doc);
        }
    }
}

impl Replicas {
    /// Keeps `record`, heard from `source`, if it is newer than every record
    /// held for its document. The caller has checked that it verifies against
    /// the document's namespace.
    pub fn observe_record(&self, record: &Record, source: EndpointId) {
        self.0.lock().observe(record.clone(), source);
    }

    /// The newest record of the author's for `doc`, if it says the author
    /// pinned it.
    #[must_use]
    pub fn pinned_record(&self, doc: DocId) -> Option<Record> {
        let inner = self.0.lock();
        let record = inner
            .newest_record(doc)
            .filter(|r| r.payload.pinned)
            .cloned();
        drop(inner);
        record
    }

    /// The newest record of the author's for `doc`, if it is a withdrawal.
    #[must_use]
    pub fn withdrawal(&self, doc: DocId) -> Option<Record> {
        let inner = self.0.lock();
        let record = inner
            .newest_record(doc)
            .filter(|r| !r.payload.pinned)
            .cloned();
        drop(inner);
        record
    }

    /// Whether the newest record of the author's for `doc` says it is pinned.
    #[must_use]
    pub fn is_pin_current(&self, doc: DocId) -> bool {
        self.0
            .lock()
            .newest_record(doc)
            .is_some_and(|record| record.payload.pinned)
    }

    /// Whether the newest record of the author's for `doc` is a withdrawal.
    #[must_use]
    pub fn is_withdrawn(&self, doc: DocId) -> bool {
        self.0
            .lock()
            .newest_record(doc)
            .is_some_and(|record| !record.payload.pinned)
    }

    /// The ordering stamp of the newest record held for `doc`.
    #[must_use]
    pub fn newest_record_at(&self, doc: DocId) -> Option<u64> {
        self.0.lock().newest_record(doc).map(|record| record.stamp)
    }

    /// Drops every record held for `space`.
    pub fn forget_space_records(&self, space: SpaceId) {
        self.0.lock().records.forget_space(space);
    }

    /// Drops the records heard from `source`.
    pub fn drop_hearsay(&self, source: EndpointId) {
        self.0.lock().records.drop_source(source);
    }

    #[cfg(test)]
    pub(super) fn known_in(&self, space: SpaceId) -> usize {
        self.0.lock().records.known_in(space)
    }

    #[cfg(test)]
    pub(super) fn heard_from(&self, source: EndpointId) -> usize {
        self.0.lock().records.heard_from(source)
    }
}

#[cfg(test)]
mod tests {
    use iroh_docs::NamespaceSecret;

    use super::*;
    use crate::replication::record::PinRecord;

    fn secret(n: usize) -> NamespaceSecret {
        let mut bytes = [3; 32];
        bytes[..8].copy_from_slice(&(n as u64).to_le_bytes());
        NamespaceSecret::from_bytes(&bytes)
    }

    fn doc(n: usize) -> DocId {
        DocId(*secret(n).id().as_bytes())
    }

    fn withdrawal(n: usize, space: SpaceId) -> Record {
        let payload = PinRecord {
            doc: doc(n),
            space,
            at: 1,
            pinned: false,
        };
        Record::sign(&secret(n), payload).expect("sign")
    }

    #[test]
    fn known_records_are_bounded_per_space_and_a_document_without_presence_goes_first() {
        let (full, other) = (SpaceId([1; 32]), SpaceId([2; 32]));
        let present = |d: DocId| d == doc(0);
        let mut records = Records::default();

        records.insert_known(withdrawal(0, full), present);
        for n in 1..=MAX_SNAPSHOT_DOCS {
            records.insert_known(withdrawal(n, full), present);
        }
        records.insert_known(withdrawal(MAX_SNAPSHOT_DOCS + 1, other), present);

        assert_eq!(records.known_in(full), MAX_SNAPSHOT_DOCS);
        assert!(
            records.get(doc(0)).is_some(),
            "a present document's record is kept"
        );
        assert!(
            records.get(doc(1)).is_none(),
            "the oldest of the rest is evicted"
        );
        assert_eq!(
            records.known_in(other),
            1,
            "another space has its own budget"
        );
    }

    #[test]
    fn leaving_a_space_forgets_both_tiers() {
        let (left, kept) = (SpaceId([1; 32]), SpaceId([2; 32]));
        let source = iroh::SecretKey::from_bytes(&[4; 32]).public();
        let mut records = Records::default();
        records.insert_known(withdrawal(0, left), |_| false);
        records.insert_hearsay(source, withdrawal(1, left));
        records.insert_hearsay(source, withdrawal(2, kept));

        records.forget_space(left);

        assert!(records.get(doc(0)).is_none() && records.get(doc(1)).is_none());
        assert!(records.get(doc(2)).is_some());
        assert_eq!(records.known_in(left), 0);
    }
}
