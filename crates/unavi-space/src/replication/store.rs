//! [`Replicas`]: every peer's pins, holds and session layers, and the rules
//! for who may state each.
//!
//! Nothing here resolves on a local clock. The author endpoint orders by the
//! signed stamp of each author pin's record and the holder by each hold's
//! stamp, both broken by endpoint id, so every peer holding the same
//! statements resolves the same answer.

use std::{
    collections::{
        BTreeSet,
        HashMap,
        HashSet,
        hash_map::Entry,
    },
    sync::{
        Arc,
        atomic::{
            AtomicBool,
            Ordering,
        },
    },
};

use bevy::prelude::{
    FromWorld,
    Resource,
    World,
};
use holds::Holds;
use hsd::id::DocId;
use iroh::EndpointId;
use parking_lot::Mutex;
use records::Records;
use unavi_identity::authorship::Authorship;
use unavi_policy::{
    Policy,
    ledger::Record as PolicyRecord,
    quota::{
        Stock,
        StockLease,
    },
    rules::Rules,
};
use xdid::core::did::Did;

use crate::{
    authority::quota::Attribution,
    membership::SpaceId,
    replication::{
        message::{
            Batch,
            DocSnapshot,
            HoldClaim,
            Live,
            PinSnapshot,
            ReplicationMsg,
        },
        record::Record,
        session::{
            SessionError,
            SessionKey,
            layers::{
                Applied,
                Opinion,
                Orphan,
                SessionLayers,
                Writer,
            },
        },
        snapshot,
    },
};

mod holds;
mod records;

/// Messages one outbound state stream may queue before it falls back to a
/// fresh snapshot.
const OUTBOUND_QUEUE: usize = 256;

/// A peer's pin: the record it states, and whether the pinner proved the
/// author's DID.
struct Pin {
    record:    Record,
    by_author: bool,
}

/// An authorship proof whose namespace signature checked out, with the DID it
/// names.
pub struct Proven {
    pub author: Did,
    pub proof:  Authorship,
}

/// A document some peer pins. `_doc_lease` charges one `Documents` unit while
/// it exists.
struct DocPresence {
    space:       SpaceId,
    /// The DID a verified pin proved, fixed once set.
    author:      Option<Did>,
    /// The proof that pin carried, re-sent when this node pins too.
    proof:       Option<Authorship>,
    _doc_lease:  StockLease,
    pins:        u32,
    /// The author's pins, by the signed stamp of each one's record, then
    /// endpoint.
    author_pins: BTreeSet<(u64, EndpointId)>,
}

/// A registered outbound state stream.
struct Outbound {
    tx:     async_channel::Sender<ReplicationMsg>,
    resync: Arc<AtomicBool>,
}

/// A registered delta stream's cancel token.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct StreamToken(u64);

/// One outbound state stream: its token, its queue, and whether the queue
/// overflowed and has to be replaced by a snapshot.
pub struct StateFeed {
    pub token:  StreamToken,
    pub rx:     async_channel::Receiver<ReplicationMsg>,
    pub resync: Arc<AtomicBool>,
}

/// Why a pin was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PinRefused {
    #[error("document is already pinned in another space")]
    WrongSpace,
    #[error("document is already authored by another DID")]
    WrongAuthor,
    #[error("document quota exceeded")]
    Quota,
}

struct Inner {
    attribution: Attribution,
    pins:        HashMap<EndpointId, HashMap<DocId, Pin>>,
    docs:        HashMap<DocId, DocPresence>,
    holds:       Holds,
    /// Documents in the world this node minted, and so authors.
    minted:      HashSet<DocId>,
    records:     Records,
    sessions:    SessionLayers,
    /// Bumped whenever a document's author becomes known or is forgotten.
    authors:     u64,
    senders:     HashMap<StreamToken, Outbound>,
    next_token:  u64,
}

impl Inner {
    fn broadcast(&mut self, msg: &ReplicationMsg) {
        self.senders
            .retain(|_, out| match out.tx.try_send(msg.clone()) {
                Ok(()) => true,
                Err(async_channel::TrySendError::Full(_)) => {
                    out.resync.store(true, Ordering::Relaxed);
                    true
                }
                Err(async_channel::TrySendError::Closed(_)) => false,
            });
    }

    fn author_of(&self, doc: DocId) -> Option<Did> {
        if let Some(author) = self.docs.get(&doc).and_then(|p| p.author.clone()) {
            return Some(author);
        }
        if self.minted.contains(&doc) {
            return self.attribution.local_did();
        }
        None
    }

    /// `doc`'s author as permissions and quota resolve it: the author of the
    /// document at the top of its host chain, else its own.
    fn author_resolved(&self, doc: DocId) -> Option<Did> {
        let root = self.attribution.policy().root(doc);
        self.author_of(root).or_else(|| self.author_of(doc))
    }

    /// `doc`'s presence in `space`, created on first sight with one
    /// `Documents` unit charged. Refuses a document present in another space.
    fn ensure_presence(
        &mut self,
        doc: DocId,
        space: SpaceId,
        author: &Did,
    ) -> Result<&mut DocPresence, PinRefused> {
        match self.docs.entry(doc) {
            Entry::Occupied(o) if o.get().space == space => Ok(o.into_mut()),
            Entry::Occupied(_) => Err(PinRefused::WrongSpace),
            Entry::Vacant(v) => {
                let quota = self
                    .attribution
                    .document_quota(doc, Some(space), Some(author));
                let Ok(lease) = quota.lease(Stock::Documents, 1) else {
                    forget_untracked(&self.attribution, doc);
                    return Err(PinRefused::Quota);
                };
                Ok(v.insert(DocPresence {
                    space,
                    author: None,
                    proof: None,
                    _doc_lease: lease,
                    pins: 0,
                    author_pins: BTreeSet::new(),
                }))
            }
        }
    }

    /// Releases `doc`'s presence, and its `Documents` lease, once nobody pins
    /// it.
    fn prune_presence(&mut self, doc: DocId) {
        if let Entry::Occupied(p) = self.docs.entry(doc)
            && p.get().pins == 0
        {
            let space = p.get().space;
            p.remove();
            forget_untracked(&self.attribution, doc);
            self.settle_records(doc, space);
        }
    }

    /// Records or restates `peer`'s pin, proving `did`, stating `record`. The
    /// caller has checked that the proof names its author and that the record
    /// verifies. Answers whether the pin made the author known.
    fn add_pin(
        &mut self,
        peer: EndpointId,
        did: Option<&Did>,
        Proven { author, proof }: Proven,
        record: Record,
    ) -> Result<bool, PinRefused> {
        let (doc, space) = (record.payload.doc, record.payload.space);
        if self.author_of(doc).is_some_and(|known| known != author) {
            return Err(PinRefused::WrongAuthor);
        }
        let by_author = did == Some(&author);
        let at = record.payload.at;
        self.ensure_presence(doc, space, &author)?;
        self.promote_records(doc);
        self.observe(record.clone(), peer);

        let old = self
            .pins
            .entry(peer)
            .or_default()
            .insert(doc, Pin { record, by_author });
        let Some(presence) = self.docs.get_mut(&doc) else {
            return Err(PinRefused::WrongSpace);
        };
        match old {
            None => presence.pins += 1,
            Some(old) if old.by_author => {
                presence.author_pins.remove(&(old.record.payload.at, peer));
            }
            Some(_) => {}
        }
        if by_author {
            presence.author_pins.insert((at, peer));
        }
        let attributed = presence.author.is_none();
        if attributed {
            presence.author = Some(author);
        }
        if presence.proof.is_none() {
            presence.proof = Some(proof);
        }
        Ok(attributed)
    }

    fn remove_pin(&mut self, peer: EndpointId, doc: DocId) {
        let Some(pins) = self.pins.get_mut(&peer) else {
            return;
        };
        let Some(pin) = pins.remove(&doc) else {
            return;
        };
        if pins.is_empty() {
            self.pins.remove(&peer);
        }
        if let Some(presence) = self.docs.get_mut(&doc) {
            presence.pins = presence.pins.saturating_sub(1);
            if pin.by_author {
                presence.author_pins.remove(&(pin.record.payload.at, peer));
            }
        }
        self.prune_presence(doc);
    }

    fn pin(&self, peer: EndpointId, doc: DocId) -> Option<&Pin> {
        self.pins.get(&peer)?.get(&doc)
    }

    /// The author endpoint whose pin's record is stamped earliest. Pins by
    /// anyone else never resolve it.
    fn author_endpoint(&self, space: SpaceId, doc: DocId) -> Option<EndpointId> {
        let presence = self.docs.get(&doc).filter(|p| p.space == space)?;
        presence.author_pins.first().map(|(_, peer)| *peer)
    }

    /// The highest-ranked hold, else the author endpoint, so an author drives
    /// its objects until someone grabs them.
    fn holder(&self, space: SpaceId, doc: DocId) -> Option<EndpointId> {
        if self.docs.get(&doc).is_none_or(|p| p.space != space) {
            return None;
        }
        self.holds
            .holder(doc, space)
            .or_else(|| self.author_endpoint(space, doc))
    }

    fn self_snapshot(&self, me: EndpointId) -> Vec<DocSnapshot> {
        let mut by_doc: HashMap<DocId, DocSnapshot> = HashMap::new();
        for (doc, pin) in self.pins.get(&me).into_iter().flatten() {
            let Some(presence) = self.docs.get(doc) else {
                continue;
            };
            let snapshot = presence.proof.clone().map(|author| PinSnapshot {
                author,
                record: pin.record.signed.clone(),
            });
            by_doc.insert(*doc, empty_snapshot(*doc, presence.space, snapshot));
        }
        for (doc, space, at) in self.holds.of_peer(me) {
            by_doc
                .entry(doc)
                .or_insert_with(|| empty_snapshot(doc, space, None))
                .hold = Some(at);
        }
        for (doc, space, layer) in self.sessions.snapshot_of(me) {
            by_doc
                .entry(doc)
                .or_insert_with(|| empty_snapshot(doc, space, None))
                .layers
                .push(layer);
        }
        for withdrawal in self.records.withdrawn() {
            let (doc, space) = (withdrawal.payload.doc, withdrawal.payload.space);
            by_doc
                .entry(doc)
                .or_insert_with(|| empty_snapshot(doc, space, None));
        }
        for (doc, snapshot) in &mut by_doc {
            snapshot.withdrawal = self
                .newest_record(*doc)
                .filter(|record| !record.payload.pinned && self.in_space(snapshot.space))
                .map(|record| record.signed.clone());
        }
        by_doc
            .into_values()
            .filter(|snapshot| {
                snapshot.pin.is_some()
                    || snapshot.hold.is_some()
                    || snapshot.withdrawal.is_some()
                    || !snapshot.layers.is_empty()
            })
            .collect()
    }
}

const fn empty_snapshot(doc: DocId, space: SpaceId, pin: Option<PinSnapshot>) -> DocSnapshot {
    DocSnapshot {
        doc,
        space,
        pin,
        hold: None,
        withdrawal: None,
        layers: Vec::new(),
    }
}

/// Drops the quota a document never in the world was given, so refused or
/// departed documents do not accumulate principals.
fn forget_untracked(attribution: &Attribution, doc: DocId) {
    let policy = attribution.policy();
    if policy.get(doc) == PolicyRecord::default() {
        policy.forget_document(doc);
    }
}

/// Every peer's replicated view of the documents in play: who pins what, who
/// holds each document, and the session cells the documents carry.
///
/// One value per app, sharing the app's [`Policy`].
#[derive(Resource, Clone)]
pub struct Replicas(Arc<Mutex<Inner>>);

impl FromWorld for Replicas {
    fn from_world(world: &mut World) -> Self {
        let policy = world.get_resource_or_init::<Policy>().clone();
        let rules = world.get_resource_or_init::<Rules>().clone();
        Self::new(policy, rules)
    }
}

impl Replicas {
    #[must_use]
    pub fn new(policy: Policy, rules: Rules) -> Self {
        Self(Arc::new(Mutex::new(Inner {
            attribution: Attribution::new(policy, rules),
            pins:        HashMap::new(),
            docs:        HashMap::new(),
            holds:       Holds::default(),
            minted:      HashSet::new(),
            records:     Records::default(),
            sessions:    SessionLayers::default(),
            authors:     0,
            senders:     HashMap::new(),
            next_token:  0,
        })))
    }

    /// The quota resolver this store charges through.
    #[must_use]
    pub fn attribution(&self) -> Attribution {
        self.0.lock().attribution.clone()
    }

    /// Registers an outbound state stream. Its first message is a snapshot
    /// of `me`'s state.
    #[must_use]
    pub fn register_stream(&self, me: EndpointId) -> StateFeed {
        let (tx, rx) = async_channel::bounded(OUTBOUND_QUEUE);
        let resync = Arc::new(AtomicBool::new(false));
        let mut inner = self.0.lock();
        let snapshot = ReplicationMsg::Snapshot {
            docs: inner.self_snapshot(me),
            more: false,
        };
        if tx.try_send(snapshot).is_err() {
            resync.store(true, Ordering::Relaxed);
        }
        let token = StreamToken(inner.next_token);
        inner.next_token += 1;
        inner.senders.insert(
            token,
            Outbound {
                tx,
                resync: Arc::clone(&resync),
            },
        );
        drop(inner);
        StateFeed { token, rx, resync }
    }

    /// Replaces an overflowed queue with a snapshot of `me`'s state. Taken
    /// under the store lock, so every delta queued afterwards is newer.
    #[must_use]
    pub fn resync(&self, feed: &StateFeed, me: EndpointId) -> ReplicationMsg {
        let inner = self.0.lock();
        while feed.rx.try_recv().is_ok() {}
        feed.resync.store(false, Ordering::Relaxed);
        ReplicationMsg::Snapshot {
            docs: inner.self_snapshot(me),
            more: false,
        }
    }

    pub fn unregister_stream(&self, token: StreamToken) {
        self.0.lock().senders.remove(&token);
    }

    pub fn broadcast(&self, msg: &ReplicationMsg) {
        self.0.lock().broadcast(msg);
    }

    /// Records or restates `peer`'s pin on the document `record` names,
    /// proving `did`. The caller has checked that the proof names its author
    /// and that the record verifies against the document. The pin is by the
    /// author when `did` is it.
    pub fn add_pin(
        &self,
        peer: EndpointId,
        did: Option<&Did>,
        proven: Proven,
        record: Record,
    ) -> Result<(), PinRefused> {
        let (doc, space) = (record.payload.doc, record.payload.space);
        let author = proven.author.clone();
        let mut inner = self.0.lock();
        let attributed = inner.add_pin(peer, did, proven, record)?;
        if attributed {
            inner.authors = inner.authors.wrapping_add(1);
        }
        let attribution = inner.attribution.clone();
        drop(inner);
        if attributed {
            attribution.reassign(doc, Some(space), Some(&author));
        }
        Ok(())
    }

    pub fn remove_pin(&self, peer: EndpointId, doc: DocId) {
        self.0.lock().remove_pin(peer, doc);
    }

    /// Whether `peer` pins `doc`.
    #[must_use]
    pub fn pins_by(&self, peer: EndpointId, doc: DocId) -> bool {
        self.0.lock().pin(peer, doc).is_some()
    }

    /// Whether `peer`'s pin on `doc` is by the author.
    #[must_use]
    pub fn pins_as_author(&self, peer: EndpointId, doc: DocId) -> bool {
        self.0
            .lock()
            .pin(peer, doc)
            .is_some_and(|pin| pin.by_author)
    }

    /// Whether any peer currently pins `doc`.
    #[must_use]
    pub fn is_pinned(&self, doc: DocId) -> bool {
        self.0.lock().docs.get(&doc).is_some_and(|p| p.pins > 0)
    }

    /// Whether a peer proving `doc`'s author currently pins it.
    #[must_use]
    pub fn is_author_pinned(&self, doc: DocId) -> bool {
        self.0
            .lock()
            .docs
            .get(&doc)
            .is_some_and(|p| !p.author_pins.is_empty())
    }

    /// The author and proof the first verified pin on `doc` carried.
    #[must_use]
    pub fn proven(&self, doc: DocId) -> Option<Proven> {
        let inner = self.0.lock();
        let proven = inner.docs.get(&doc).and_then(|presence| {
            Some(Proven {
                author: presence.author.clone()?,
                proof:  presence.proof.clone()?,
            })
        });
        drop(inner);
        proven
    }

    /// Adds or restates `peer`'s hold. `false` if `peer` holds too many
    /// documents already.
    #[must_use]
    pub fn add_hold(&self, peer: EndpointId, hold: HoldClaim) -> bool {
        self.0.lock().holds.add(peer, hold)
    }

    pub fn remove_hold(&self, peer: EndpointId, doc: DocId) {
        self.0.lock().holds.remove(peer, doc);
    }

    /// The newest stamp of any hold on `doc`.
    #[must_use]
    pub fn newest_hold_at(&self, doc: DocId) -> Option<u64> {
        self.0.lock().holds.newest(doc)
    }

    #[must_use]
    pub fn holder(&self, space: SpaceId, doc: DocId) -> Option<EndpointId> {
        self.0.lock().holder(space, doc)
    }

    #[must_use]
    pub fn is_holder(&self, space: SpaceId, doc: DocId, me: EndpointId) -> bool {
        self.holder(space, doc) == Some(me)
    }

    /// The author endpoint whose pin's record is stamped earliest.
    #[must_use]
    pub fn author_endpoint(&self, space: SpaceId, doc: DocId) -> Option<EndpointId> {
        self.0.lock().author_endpoint(space, doc)
    }

    /// Whether `me` is the author endpoint resolving `doc`: the device that
    /// may commit it durably.
    #[must_use]
    pub fn is_author_endpoint(&self, space: SpaceId, doc: DocId, me: EndpointId) -> bool {
        self.author_endpoint(space, doc) == Some(me)
    }

    /// Writes a batch, or the `live` writes of it, into its writer's session
    /// layer, all or none, charging the writer. `source` is the peer the
    /// layer is held through.
    pub(crate) fn add_session(
        &self,
        source: EndpointId,
        batch: &Batch,
        live: Option<Live<'_>>,
    ) -> Result<Applied, SessionError> {
        self.write_session(source, batch, live, false)
    }

    /// Writes a batch that restates the local writer's live cells, which may
    /// take the layer past its batch cap until the older batches drop.
    pub(crate) fn add_checkpoint(
        &self,
        source: EndpointId,
        batch: &Batch,
    ) -> Result<Applied, SessionError> {
        self.write_session(source, batch, None, true)
    }

    fn write_session(
        &self,
        source: EndpointId,
        batch: &Batch,
        live: Option<Live<'_>>,
        restating: bool,
    ) -> Result<Applied, SessionError> {
        let mut inner = self.0.lock();
        let payload = &batch.payload;
        if inner
            .docs
            .get(&payload.doc)
            .is_some_and(|p| p.space != payload.space)
        {
            return Err(SessionError::WrongSpace);
        }
        let did = Some(&payload.did);
        let quota = inner.attribution.writer_quota(payload.writer, did);
        let reserved = inner.author_resolved(payload.doc).as_ref() == did
            || inner.attribution.local_did().as_ref() == did;
        let writer = Writer {
            peer: payload.writer,
            did: &payload.did,
            quota: &quota,
            reserved,
            restating,
        };
        inner.sessions.write(&writer, source, batch, live)
    }

    /// Records `source` as holding `writer`'s layer on `doc`. Answers whether
    /// the layer exists.
    pub(crate) fn add_session_source(
        &self,
        writer: EndpointId,
        doc: DocId,
        source: EndpointId,
    ) -> bool {
        self.0.lock().sessions.add_source(writer, doc, source)
    }

    /// Drops `source` from `writer`'s layer on `doc`, and the layer once no
    /// source is left. Answers whether the layer is gone.
    pub(crate) fn remove_session_source(
        &self,
        writer: EndpointId,
        doc: DocId,
        source: EndpointId,
    ) -> bool {
        self.0.lock().sessions.remove_source(writer, doc, source)
    }

    /// The peers `writer`'s layer on `doc` is held through.
    pub(crate) fn session_sources(&self, writer: EndpointId, doc: DocId) -> Vec<EndpointId> {
        self.0.lock().sessions.sources(writer, doc)
    }

    /// Each document `writer` has a layer on, with each peer it is held
    /// through.
    pub(crate) fn session_sources_of_writer(&self, writer: EndpointId) -> Vec<(DocId, EndpointId)> {
        self.0.lock().sessions.sources_of_writer(writer)
    }

    /// Layers `source` alone holds, other than `me`'s own.
    pub(crate) fn orphaned_by(&self, source: EndpointId, me: EndpointId) -> Vec<Orphan> {
        self.0.lock().sessions.orphaned_by(source, me)
    }

    /// Whether `writer`'s retained batches on `doc` are due a checkpoint.
    pub(crate) fn needs_checkpoint(&self, writer: EndpointId, doc: DocId) -> bool {
        self.0.lock().sessions.needs_checkpoint(writer, doc)
    }

    /// Records how a checkpoint of `writer`'s layer on `doc` ended.
    pub(crate) fn record_checkpoint(&self, writer: EndpointId, doc: DocId, succeeded: bool) {
        self.0
            .lock()
            .sessions
            .record_checkpoint(writer, doc, succeeded);
    }

    /// Writers holding a session layer on `doc`, with the DID each layer
    /// names.
    pub(crate) fn session_layers(&self, doc: DocId) -> Vec<(EndpointId, Did)> {
        self.0.lock().sessions.layers(doc)
    }

    /// Writers holding a session layer on `doc`.
    #[must_use]
    pub fn session_writers(&self, doc: DocId) -> Vec<EndpointId> {
        self.0.lock().sessions.writers(doc)
    }

    /// `writer`'s session opinions on `doc`.
    pub(crate) fn session_opinions(&self, writer: EndpointId, doc: DocId) -> Vec<Opinion> {
        self.0.lock().sessions.opinions(writer, doc)
    }

    /// The keys `writer` has cleared on `doc`, with the stamp of each.
    pub(crate) fn session_tombstones(
        &self,
        writer: EndpointId,
        doc: DocId,
    ) -> Vec<(SessionKey, u64)> {
        self.0.lock().sessions.tombstones(writer, doc)
    }

    /// Who authors `doc`: the DID a pin proved, or this node for one it
    /// minted, for the document at the top of its host chain and else for
    /// `doc` itself. `None` when nothing proved one.
    #[must_use]
    pub fn author(&self, doc: DocId) -> Option<Did> {
        self.0.lock().author_resolved(doc)
    }

    /// Records that this node minted `doc`, and so authors it.
    pub fn record_minted(&self, doc: DocId) {
        let mut inner = self.0.lock();
        inner.minted.insert(doc);
        inner.authors = inner.authors.wrapping_add(1);
        let space = inner.docs.get(&doc).map(|p| p.space);
        let author = inner.author_of(doc);
        inner.attribution.reassign(doc, space, author.as_ref());
    }

    /// Forgets a minted document that left the world.
    pub fn forget_minted(&self, doc: DocId) {
        let mut inner = self.0.lock();
        inner.minted.remove(&doc);
        inner.authors = inner.authors.wrapping_add(1);
    }

    /// Counts changes to who authors what, so a reader can tell cheaply that
    /// none came.
    #[must_use]
    pub fn author_generation(&self) -> u64 {
        self.0.lock().authors
    }

    #[must_use]
    pub fn has_doc(&self, space: SpaceId, doc: DocId) -> bool {
        self.0
            .lock()
            .docs
            .get(&doc)
            .is_some_and(|p| p.space == space)
    }

    /// The space `doc` is present in, per any peer's replica.
    #[must_use]
    pub fn space_of(&self, doc: DocId) -> Option<SpaceId> {
        self.0.lock().docs.get(&doc).map(|p| p.space)
    }

    /// Remote peers pinning `doc`, which `me` can sync it from: the author
    /// endpoint first, then the rest by endpoint id.
    #[must_use]
    pub fn sync_sources(&self, doc: DocId, me: EndpointId) -> Vec<EndpointId> {
        let inner = self.0.lock();
        let Some(space) = inner.docs.get(&doc).map(|p| p.space) else {
            return Vec::new();
        };
        let author_endpoint = inner.author_endpoint(space, doc);
        let mut pinners = inner
            .pins
            .iter()
            .filter(|(pid, pins)| {
                **pid != me && Some(**pid) != author_endpoint && pins.contains_key(&doc)
            })
            .map(|(pid, _)| *pid)
            .collect::<Vec<_>>();
        drop(inner);
        pinners.sort_unstable();
        if let Some(author_endpoint) = author_endpoint.filter(|o| *o != me) {
            pinners.insert(0, author_endpoint);
        }
        pinners
    }

    /// A deterministically ordered snapshot, for inspection.
    #[must_use]
    pub fn snapshot(&self) -> snapshot::ReplicaSnapshot {
        let inner = self.0.lock();
        let ids = inner
            .pins
            .keys()
            .copied()
            .chain(inner.holds.peers())
            .collect::<HashSet<_>>();
        let mut peers = ids
            .into_iter()
            .map(|pid| {
                let mut docs: HashMap<DocId, snapshot::PeerDoc> = HashMap::new();
                for (doc, pin) in inner.pins.get(&pid).into_iter().flatten() {
                    let Some(presence) = inner.docs.get(doc) else {
                        continue;
                    };
                    docs.insert(
                        *doc,
                        snapshot::PeerDoc {
                            doc:   *doc,
                            space: presence.space,
                            pin:   Some(pin.record.payload.at),
                            hold:  None,
                        },
                    );
                }
                for (doc, space, at) in inner.holds.of_peer(pid) {
                    docs.entry(doc)
                        .or_insert(snapshot::PeerDoc {
                            doc,
                            space,
                            pin: None,
                            hold: None,
                        })
                        .hold = Some(at);
                }
                let mut docs = docs.into_values().collect::<Vec<_>>();
                docs.sort_unstable_by_key(|d| d.doc.0);
                snapshot::PeerState { peer: pid, docs }
            })
            .collect::<Vec<_>>();
        peers.sort_unstable_by_key(|p| p.peer);

        let mut docs = inner
            .docs
            .iter()
            .map(|(doc, p)| {
                (
                    *doc,
                    snapshot::DocState {
                        doc:     *doc,
                        space:   p.space,
                        author:  p.author.as_ref().map(ToString::to_string),
                        session: Vec::new(),
                    },
                )
            })
            .collect::<HashMap<_, _>>();
        for (doc, writer, opinions) in inner.sessions.all() {
            let Some(space) = inner.sessions.space_of(doc) else {
                continue;
            };
            let state = docs.entry(doc).or_insert_with(|| snapshot::DocState {
                doc,
                space,
                author: None,
                session: Vec::new(),
            });
            state
                .session
                .extend(
                    opinions
                        .into_iter()
                        .map(|(key, value, at)| snapshot::SessionCell {
                            prim: key.prim,
                            name: key.name,
                            value,
                            at,
                            writer,
                        }),
                );
        }
        let mut docs = docs.into_values().collect::<Vec<_>>();
        for doc in &mut docs {
            doc.session.sort_unstable_by(|a, b| {
                (a.prim, &a.name, a.writer).cmp(&(b.prim, &b.name, b.writer))
            });
        }
        docs.sort_unstable_by_key(|d| d.doc.0);
        drop(inner);
        snapshot::ReplicaSnapshot { peers, docs }
    }
}

#[cfg(test)] mod tests;
