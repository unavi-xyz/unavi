//! [`Replicas`]: every peer's pins, holds and session layers, and the rules
//! for who may state each.

use std::{
    collections::{
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

mod records;

/// Messages one outbound state stream may queue before it falls back to a
/// fresh snapshot.
const OUTBOUND_QUEUE: usize = 256;

/// One peer's contribution to a document, stamped with the local time each
/// was received.
#[derive(Default)]
struct PeerDocEntry {
    pin:    Option<PinStamp>,
    /// The record the pin states.
    record: Option<Record>,
    hold:   Option<u64>,
}

impl PeerDocEntry {
    const fn is_empty(&self) -> bool {
        self.pin.is_none() && self.hold.is_none()
    }
}

/// When a peer's pin arrived, and whether the pinner proved the author's DID.
#[derive(Clone, Copy)]
struct PinStamp {
    at:        u64,
    by_author: bool,
}

/// An authorship proof whose namespace signature checked out, with the DID it
/// names.
pub struct Proven {
    pub author: Did,
    pub proof:  Authorship,
}

#[derive(Default)]
struct PeerReplica {
    docs: HashMap<DocId, PeerDocEntry>,
}

/// Per-document state shared across peers. `refs` counts the live pins and
/// holds keeping the presence alive; `_doc_lease` charges one `Documents`
/// unit while the doc is known locally.
struct DocPresence {
    space:       SpaceId,
    /// The DID a verified pin proved, fixed once set.
    author:      Option<Did>,
    /// The proof that pin carried, re-sent when this node pins too.
    proof:       Option<Authorship>,
    /// The peer the holder reserved the next hold for. While set, only it or
    /// the author may hold.
    hold_grant:  Option<EndpointId>,
    _doc_lease:  StockLease,
    refs:        u32,
    /// Peers pinning it, and those among them who proved its author, counted
    /// so neither question scans every peer's replica.
    pins:        u32,
    author_pins: u32,
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
    peers:       HashMap<EndpointId, PeerReplica>,
    docs:        HashMap<DocId, DocPresence>,
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

    /// Ensures a [`DocPresence`] for `doc` in `space`, charging one
    /// `Documents` unit on first sight. Refuses a document already present in
    /// another space.
    fn ensure_presence(
        &mut self,
        doc: DocId,
        space: SpaceId,
        author: Option<&Did>,
    ) -> Result<&mut DocPresence, PinRefused> {
        let author = author.cloned().or_else(|| self.author_of(doc));
        match self.docs.entry(doc) {
            Entry::Occupied(o) if o.get().space == space => Ok(o.into_mut()),
            Entry::Occupied(_) => Err(PinRefused::WrongSpace),
            Entry::Vacant(v) => {
                let quota = self
                    .attribution
                    .document_quota(doc, Some(space), author.as_ref());
                let Ok(lease) = quota.lease(Stock::Documents, 1) else {
                    forget_untracked(&self.attribution, doc);
                    return Err(PinRefused::Quota);
                };
                Ok(v.insert(DocPresence {
                    space,
                    author,
                    proof: None,
                    hold_grant: None,
                    _doc_lease: lease,
                    refs: 0,
                    pins: 0,
                    author_pins: 0,
                }))
            }
        }
    }

    fn inc_ref(&mut self, doc: DocId) {
        if let Some(p) = self.docs.get_mut(&doc) {
            p.refs += 1;
        }
    }

    fn prune_presence(&mut self, doc: DocId) {
        if let Entry::Occupied(p) = self.docs.entry(doc)
            && p.get().refs == 0
        {
            let space = p.get().space;
            p.remove();
            forget_untracked(&self.attribution, doc);
            self.settle_records(doc, space);
        }
    }

    /// Drops one reference to `doc`, releasing its presence (and the
    /// `Documents` lease) once nothing references it.
    fn dec_ref(&mut self, doc: DocId) {
        if let Some(p) = self.docs.get_mut(&doc) {
            p.refs = p.refs.saturating_sub(1);
        }
        self.prune_presence(doc);
    }

    /// Removes a peer's entry for `doc` once it holds no data.
    fn prune_entry(&mut self, peer: EndpointId, doc: DocId) {
        if let Some(replica) = self.peers.get_mut(&peer)
            && let Entry::Occupied(e) = replica.docs.entry(doc)
            && e.get().is_empty()
        {
            e.remove();
            if replica.docs.is_empty() {
                self.peers.remove(&peer);
            }
        }
    }

    fn entry(&mut self, peer: EndpointId, doc: DocId) -> &mut PeerDocEntry {
        self.peers
            .entry(peer)
            .or_default()
            .docs
            .entry(doc)
            .or_default()
    }

    /// Records `peer`'s pin, proving `did`. The caller has checked that the
    /// proof names its author. A pin is by the author when `did` is it.
    fn add_pin(
        &mut self,
        peer: EndpointId,
        did: Option<&Did>,
        doc: DocId,
        space: SpaceId,
        Proven { author, proof }: Proven,
        at: u64,
    ) -> Result<bool, PinRefused> {
        if self.author_of(doc).is_some_and(|known| known != author) {
            return Err(PinRefused::WrongAuthor);
        }
        let by_author = did == Some(&author);
        self.ensure_presence(doc, space, Some(&author))?;
        self.promote_records(doc);
        let Some(presence) = self.docs.get_mut(&doc) else {
            return Err(PinRefused::WrongSpace);
        };
        let attributed = presence.author.is_none();
        if attributed {
            presence.author = Some(author);
        }
        if presence.proof.is_none() {
            presence.proof = Some(proof);
        }

        let entry = self.entry(peer, doc);
        if entry.pin.is_none() {
            entry.pin = Some(PinStamp { at, by_author });
            self.inc_ref(doc);
            if let Some(presence) = self.docs.get_mut(&doc) {
                presence.pins += 1;
                presence.author_pins += u32::from(by_author);
            }
        }
        Ok(attributed)
    }

    fn remove_pin(&mut self, peer: EndpointId, doc: DocId) {
        let Some(entry) = self.peers.get_mut(&peer).and_then(|r| r.docs.get_mut(&doc)) else {
            return;
        };
        if let Some(stamp) = entry.pin.take() {
            entry.record = None;
            if let Some(presence) = self.docs.get_mut(&doc) {
                presence.pins = presence.pins.saturating_sub(1);
                if stamp.by_author {
                    presence.author_pins = presence.author_pins.saturating_sub(1);
                }
            }
            self.prune_entry(peer, doc);
            self.dec_ref(doc);
        }
    }

    /// Whether `peer`, proving `did`, may hold `doc`: anyone while no
    /// reservation stands, the reserved peer or the author otherwise.
    fn may_hold(&self, peer: EndpointId, did: Option<&Did>, doc: DocId, space: SpaceId) -> bool {
        let Some(presence) = self.docs.get(&doc).filter(|p| p.space == space) else {
            return false;
        };
        let is_author = did.is_some_and(|did| presence.author.as_ref() == Some(did));
        is_author || presence.hold_grant.is_none_or(|grant| grant == peer)
    }

    fn add_hold(
        &mut self,
        peer: EndpointId,
        did: Option<&Did>,
        doc: DocId,
        space: SpaceId,
        at: u64,
    ) -> bool {
        if !self.may_hold(peer, did, doc, space) {
            return false;
        }
        if let Some(presence) = self.docs.get_mut(&doc)
            && presence.hold_grant.is_some_and(|grant| grant != peer)
        {
            presence.hold_grant = None;
        }
        let entry = self.entry(peer, doc);
        let was_set = entry.hold.replace(at).is_some();
        if !was_set {
            self.inc_ref(doc);
        }
        true
    }

    fn remove_hold(&mut self, peer: EndpointId, doc: DocId, to: Option<EndpointId>) {
        let Some(space) = self.docs.get(&doc).map(|p| p.space) else {
            return;
        };
        let was_holder = self.holder(space, doc) == Some(peer);
        if let Some(presence) = self.docs.get_mut(&doc) {
            if was_holder && let Some(to) = to {
                presence.hold_grant = Some(to);
            } else if presence.hold_grant == Some(peer) {
                presence.hold_grant = None;
            }
        }

        if let Some(entry) = self.peers.get_mut(&peer).and_then(|r| r.docs.get_mut(&doc))
            && entry.hold.take().is_some()
        {
            self.prune_entry(peer, doc);
            self.dec_ref(doc);
        }
    }

    /// The peer with the earliest stamp in `field`, or the latest with
    /// `newest`, ties broken by peer id.
    fn resolve_peer(
        &self,
        space: SpaceId,
        doc: DocId,
        newest: bool,
        field: impl Fn(&PeerDocEntry) -> Option<u64>,
    ) -> Option<EndpointId> {
        if self.docs.get(&doc).is_none_or(|p| p.space != space) {
            return None;
        }
        let candidates = self
            .peers
            .iter()
            .filter_map(|(pid, r)| Some((field(r.docs.get(&doc)?)?, *pid)));
        let order = |a: &(u64, EndpointId), b: &(u64, EndpointId)| {
            a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1))
        };
        if newest {
            candidates.max_by(order)
        } else {
            candidates.min_by(order)
        }
        .map(|(_, pid)| pid)
    }

    /// The first of the author's endpoints to pin `doc`. Pins by anyone else
    /// never resolve it.
    fn author_endpoint(&self, space: SpaceId, doc: DocId) -> Option<EndpointId> {
        self.resolve_peer(space, doc, false, |e| {
            e.pin.filter(|p| p.by_author).map(|p| p.at)
        })
    }

    /// The latest hold, else the author's endpoint, so an author drives its
    /// objects until someone grabs them.
    fn holder(&self, space: SpaceId, doc: DocId) -> Option<EndpointId> {
        self.resolve_peer(space, doc, true, |e| e.hold)
            .or_else(|| self.author_endpoint(space, doc))
    }

    fn self_snapshot(&self, me: EndpointId) -> Vec<DocSnapshot> {
        let mut by_doc: HashMap<DocId, DocSnapshot> = HashMap::new();
        if let Some(replica) = self.peers.get(&me) {
            for (doc, e) in &replica.docs {
                let Some(presence) = self.docs.get(doc) else {
                    continue;
                };
                let pin = e.pin.and_then(|_| {
                    Some(PinSnapshot {
                        author: presence.proof.clone()?,
                        record: e.record.as_ref()?.signed.clone(),
                    })
                });
                by_doc.insert(
                    *doc,
                    empty_snapshot(*doc, presence.space, pin, e.hold.is_some()),
                );
            }
        }
        for (doc, space, layer) in self.sessions.snapshot_of(me) {
            by_doc
                .entry(doc)
                .or_insert_with(|| empty_snapshot(doc, space, None, false))
                .layers
                .push(layer);
        }
        for withdrawal in self.records.withdrawn() {
            let (doc, space) = (withdrawal.payload.doc, withdrawal.payload.space);
            by_doc
                .entry(doc)
                .or_insert_with(|| empty_snapshot(doc, space, None, false));
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
                    || snapshot.hold
                    || snapshot.withdrawal.is_some()
                    || !snapshot.layers.is_empty()
            })
            .collect()
    }
}

const fn empty_snapshot(
    doc: DocId,
    space: SpaceId,
    pin: Option<PinSnapshot>,
    hold: bool,
) -> DocSnapshot {
    DocSnapshot {
        doc,
        space,
        pin,
        hold,
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
            peers:       HashMap::new(),
            docs:        HashMap::new(),
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
        let snapshot = inner.self_snapshot(me);
        if tx.try_send(ReplicationMsg::Snapshot(snapshot)).is_err() {
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
        ReplicationMsg::Snapshot(inner.self_snapshot(me))
    }

    pub fn unregister_stream(&self, token: StreamToken) {
        self.0.lock().senders.remove(&token);
    }

    pub fn broadcast(&self, msg: &ReplicationMsg) {
        self.0.lock().broadcast(msg);
    }

    /// Records `peer`'s pin on `doc`, proving `did`. The caller has checked
    /// that `proof` names `author`. The pin is by the author when `did` is it.
    /// Idempotent.
    pub fn add_pin(
        &self,
        peer: EndpointId,
        did: Option<&Did>,
        doc: DocId,
        space: SpaceId,
        author: Did,
        proof: Authorship,
        at: u64,
    ) -> Result<(), PinRefused> {
        let attributed_to = author.clone();
        let mut inner = self.0.lock();
        let attributed = inner.add_pin(peer, did, doc, space, Proven { author, proof }, at)?;
        if attributed {
            inner.authors = inner.authors.wrapping_add(1);
        }
        let attribution = inner.attribution.clone();
        drop(inner);
        if attributed {
            attribution.reassign(doc, Some(space), Some(&attributed_to));
        }
        Ok(())
    }

    pub fn remove_pin(&self, peer: EndpointId, doc: DocId) {
        self.0.lock().remove_pin(peer, doc);
    }

    /// Whether `peer`'s pin on `doc` is by the author.
    #[must_use]
    pub fn pins_as_author(&self, peer: EndpointId, doc: DocId) -> bool {
        self.0
            .lock()
            .peers
            .get(&peer)
            .and_then(|r| r.docs.get(&doc))
            .and_then(|e| e.pin)
            .is_some_and(|p| p.by_author)
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

    /// Whether `peer`, proving `did`, may hold `doc`.
    #[must_use]
    pub fn may_hold(
        &self,
        peer: EndpointId,
        did: Option<&Did>,
        doc: DocId,
        space: SpaceId,
    ) -> bool {
        self.0.lock().may_hold(peer, did, doc, space)
    }

    /// Adds or refreshes `peer`'s hold on `doc`. `false` if it may not hold.
    #[must_use]
    pub fn add_hold(
        &self,
        peer: EndpointId,
        did: Option<&Did>,
        doc: DocId,
        space: SpaceId,
        at: u64,
    ) -> bool {
        self.0.lock().add_hold(peer, did, doc, space, at)
    }

    /// Drops `peer`'s hold. With `to`, the holder reserves the next hold for
    /// that peer.
    pub fn remove_hold(&self, peer: EndpointId, doc: DocId, to: Option<EndpointId>) {
        self.0.lock().remove_hold(peer, doc, to);
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

    /// The first of the author's endpoints to pin `doc`.
    #[must_use]
    pub fn author_endpoint(&self, space: SpaceId, doc: DocId) -> Option<EndpointId> {
        self.0.lock().author_endpoint(space, doc)
    }

    #[must_use]
    pub fn holder(&self, space: SpaceId, doc: DocId) -> Option<EndpointId> {
        self.0.lock().holder(space, doc)
    }

    #[must_use]
    pub fn is_holder(&self, space: SpaceId, doc: DocId, me: EndpointId) -> bool {
        self.holder(space, doc) == Some(me)
    }

    /// Whether `me` is the author's endpoint resolving `doc`: the device that
    /// may commit it durably.
    #[must_use]
    pub fn is_author_endpoint(&self, space: SpaceId, doc: DocId, me: EndpointId) -> bool {
        self.author_endpoint(space, doc) == Some(me)
    }

    #[must_use]
    pub fn has_doc(&self, space: SpaceId, doc: DocId) -> bool {
        self.0
            .lock()
            .docs
            .get(&doc)
            .is_some_and(|p| p.space == space)
    }

    /// Remote peers pinning `doc`, which `me` can sync it from: the author's
    /// endpoint first, then the rest by pin stamp and peer id.
    #[must_use]
    pub fn sync_sources(&self, doc: DocId, me: EndpointId) -> Vec<EndpointId> {
        let inner = self.0.lock();
        let Some(space) = inner.docs.get(&doc).map(|p| p.space) else {
            return Vec::new();
        };
        let author_endpoint = inner.author_endpoint(space, doc);
        let mut pinners = inner
            .peers
            .iter()
            .filter(|(pid, _)| **pid != me && Some(**pid) != author_endpoint)
            .filter_map(|(pid, r)| Some((r.docs.get(&doc)?.pin?.at, *pid)))
            .collect::<Vec<_>>();
        drop(inner);
        pinners.sort_unstable();
        let mut sources = pinners.into_iter().map(|(_, pid)| pid).collect::<Vec<_>>();
        if let Some(author_endpoint) = author_endpoint.filter(|o| *o != me) {
            sources.insert(0, author_endpoint);
        }
        sources
    }

    /// The space `doc` is present in, per any peer's replica.
    #[must_use]
    pub fn space_of(&self, doc: DocId) -> Option<SpaceId> {
        self.0.lock().docs.get(&doc).map(|p| p.space)
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
            .is_some_and(|p| p.author_pins > 0)
    }

    /// Whether `peer` pins `doc`.
    #[must_use]
    pub fn pins_by(&self, peer: EndpointId, doc: DocId) -> bool {
        self.0
            .lock()
            .peers
            .get(&peer)
            .and_then(|r| r.docs.get(&doc))
            .is_some_and(|e| e.pin.is_some())
    }

    /// A deterministically ordered snapshot, for inspection.
    #[must_use]
    pub fn snapshot(&self) -> snapshot::ReplicaSnapshot {
        let inner = self.0.lock();
        let mut peers = inner
            .peers
            .iter()
            .map(|(pid, r)| {
                let mut docs = r
                    .docs
                    .iter()
                    .filter_map(|(doc, e)| {
                        Some(snapshot::PeerDoc {
                            doc:   *doc,
                            space: inner.docs.get(doc)?.space,
                            pin:   e.pin.map(|p| p.at),
                            hold:  e.hold,
                        })
                    })
                    .collect::<Vec<_>>();
                docs.sort_unstable_by_key(|d| d.doc.0);
                snapshot::PeerState { peer: *pid, docs }
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
