//! [`Replicas`]: every peer's pins, holds and session cells, and the rules for
//! who may state each.

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
use hsd::{
    id::{
        DocId,
        PrimId,
    },
    property::name::PropName,
};
use iroh::EndpointId;
use parking_lot::Mutex;
use unavi_identity::authorship::Authorship;
use unavi_policy::{
    Policy,
    ledger::Record,
    quota::{
        Quota,
        Stock,
        StockLease,
    },
};
use xdid::core::did::Did;

use crate::{
    authority::quota::Attribution,
    membership::SpaceId,
    replication::{
        cell::{
            Cell,
            Restored,
            SessionError,
            SessionKey,
            Standing,
            cell_bytes,
        },
        message::{
            DocSnapshot,
            ReplicationMsg,
            SessionSnapshot,
        },
        snapshot,
    },
};

/// Messages one outbound state stream may queue before it falls back to a
/// fresh snapshot.
const OUTBOUND_QUEUE: usize = 256;

/// One peer's contribution to a document, stamped with the local time each
/// was received.
#[derive(Default)]
struct PeerDocEntry {
    pin:  Option<u64>,
    hold: Option<u64>,
}

impl PeerDocEntry {
    const fn is_empty(&self) -> bool {
        self.pin.is_none() && self.hold.is_none()
    }
}

#[derive(Default)]
struct PeerReplica {
    docs: HashMap<DocId, PeerDocEntry>,
}

/// Per-document state shared across peers. `refs` counts the live pins,
/// holds and session cells keeping the presence alive; `_doc_lease`
/// charges one `Documents` unit while the doc is known locally.
struct DocPresence {
    space:      SpaceId,
    /// The DID a verified pin proved, fixed once set.
    author:     Option<Did>,
    /// The proof that pin carried, re-sent when this node pins too.
    proof:      Option<Authorship>,
    /// The peer the holder reserved the next hold for. While set, only it or
    /// the author may hold.
    hold_grant: Option<EndpointId>,
    quota:      Arc<Quota>,
    _doc_lease: StockLease,
    session:    HashMap<SessionKey, Cell>,
    refs:       u32,
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

    /// Ensures a [`DocPresence`] for `doc` in `space`, charging one
    /// `Documents` unit on first sight. Refuses a document already present in
    /// another space.
    fn ensure_presence(
        &mut self,
        doc: DocId,
        space: SpaceId,
        author: Option<&Did>,
    ) -> Result<(), PinRefused> {
        if let Some(presence) = self.docs.get(&doc) {
            return if presence.space == space {
                Ok(())
            } else {
                Err(PinRefused::WrongSpace)
            };
        }

        let author = author.cloned().or_else(|| self.author_of(doc));
        let quota = self
            .attribution
            .document_quota(doc, Some(space), author.as_ref());
        let Ok(lease) = quota.lease(Stock::Documents, 1) else {
            self.forget_untracked(doc);
            return Err(PinRefused::Quota);
        };
        self.docs.insert(
            doc,
            DocPresence {
                space,
                author,
                proof: None,
                hold_grant: None,
                quota,
                _doc_lease: lease,
                session: HashMap::new(),
                refs: 0,
            },
        );
        Ok(())
    }

    /// Drops the quota a document never in the world was given, so refused or
    /// departed documents do not accumulate principals.
    fn forget_untracked(&self, doc: DocId) {
        let policy = self.attribution.policy();
        if policy.get(doc) == Record::default() {
            policy.forget_document(doc);
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
            p.remove();
            self.forget_untracked(doc);
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

    /// Records `peer`'s pin. The caller has checked that `peer` proved
    /// `author`, and that `proof` names it.
    fn add_pin(
        &mut self,
        peer: EndpointId,
        doc: DocId,
        space: SpaceId,
        author: Did,
        proof: Authorship,
        at: u64,
    ) -> Result<bool, PinRefused> {
        if self.author_of(doc).is_some_and(|known| known != author) {
            return Err(PinRefused::WrongAuthor);
        }
        self.ensure_presence(doc, space, Some(&author))?;

        let presence = self.docs.get_mut(&doc).expect("presence ensured");
        let attributed = presence.author.is_none();
        if attributed {
            presence.author = Some(author);
        }
        if presence.proof.is_none() {
            presence.proof = Some(proof);
        }

        let entry = self.entry(peer, doc);
        if entry.pin.is_none() {
            entry.pin = Some(at);
            self.inc_ref(doc);
        }
        Ok(attributed)
    }

    fn remove_pin(&mut self, peer: EndpointId, doc: DocId) {
        if let Some(entry) = self.peers.get_mut(&peer).and_then(|r| r.docs.get_mut(&doc))
            && entry.pin.take().is_some()
        {
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

    /// Whether a writer proving `did` may state session cells on `doc`: anyone
    /// on a space's own document or one with no author, only its author
    /// otherwise, and nobody on a document present in another space.
    fn may_write(&self, did: Option<&Did>, doc: DocId, space: SpaceId) -> Result<(), SessionError> {
        if self.docs.get(&doc).is_some_and(|p| p.space != space) {
            return Err(SessionError::NotAuthor);
        }
        if doc == space.doc() {
            return Ok(());
        }
        match self.author_of(doc) {
            Some(author) if did != Some(&author) => Err(SessionError::NotAuthor),
            _ => Ok(()),
        }
    }

    /// Writes a batch of cells on `doc`, all or none. Every lease is taken
    /// before any cell changes, so a refusal leaves nothing behind.
    fn add_session(
        &mut self,
        writer: EndpointId,
        did: Option<&Did>,
        doc: DocId,
        space: SpaceId,
        batch: Vec<(SessionKey, Option<Vec<u8>>)>,
        at: u64,
    ) -> Result<(), SessionError> {
        self.may_write(did, doc, space)?;
        self.ensure_presence(doc, space, None)
            .map_err(|err| match err {
                PinRefused::Quota => SessionError::QuotaExceeded,
                PinRefused::WrongSpace | PinRefused::WrongAuthor => SessionError::NotAuthor,
            })?;

        let presence = self.docs.get(&doc).expect("presence ensured");
        let mut leased = Vec::with_capacity(batch.len());
        for (key, value) in batch {
            if presence.session.get(&key).is_some_and(|cell| at < cell.at) {
                continue;
            }
            let Ok(lease) = presence
                .quota
                .lease(Stock::SessionMemory, cell_bytes(&key, value.as_deref()))
            else {
                drop(leased);
                self.prune_presence(doc);
                return Err(SessionError::QuotaExceeded);
            };
            leased.push((key, value, lease));
        }

        let presence = self.docs.get_mut(&doc).expect("presence ensured");
        let mut inserted = 0;
        for (key, value, lease) in leased {
            match presence.session.entry(key) {
                Entry::Occupied(mut o) => {
                    let cell = o.get_mut();
                    // The outgoing version becomes the fallback; whatever it
                    // replaces is dropped, releasing its lease. Depth stays
                    // one.
                    let displaced = Cell {
                        at:    cell.at,
                        peer:  cell.peer,
                        value: cell.value.take(),
                        lease: std::mem::replace(&mut cell.lease, lease),
                        prev:  None,
                    };
                    cell.at = at;
                    cell.peer = writer;
                    cell.value = value;
                    cell.prev = Some(Box::new(displaced));
                }
                Entry::Vacant(v) => {
                    v.insert(Cell {
                        at,
                        peer: writer,
                        value,
                        lease,
                        prev: None,
                    });
                    inserted += 1;
                }
            }
        }
        presence.refs += inserted;
        self.prune_presence(doc);
        Ok(())
    }

    /// Restores each cell `peer` last wrote to its prior version, or drops it
    /// when the prior version was also theirs: a peer cannot leave its own
    /// earlier write behind as the fallback.
    fn revert_writes(&mut self, peer: EndpointId) -> Vec<Restored> {
        let mut emptied = Vec::new();
        let mut restored = Vec::new();

        for (&doc, presence) in &mut self.docs {
            let mut dropped = 0;
            presence.session.retain(|key, cell| {
                if cell.peer != peer {
                    return true;
                }
                match cell.prev.take() {
                    Some(prev) if prev.peer != peer => {
                        *cell = *prev;
                        restored.push(Restored {
                            doc,
                            key: key.clone(),
                            standing: Standing::Prior {
                                value: cell.value.clone(),
                                at:    cell.at,
                            },
                        });
                        true
                    }
                    _ => {
                        dropped += 1;
                        restored.push(Restored {
                            doc,
                            key: key.clone(),
                            standing: Standing::Gone,
                        });
                        false
                    }
                }
            });
            if dropped > 0 {
                emptied.push((doc, dropped));
            }
        }

        for (doc, dropped) in emptied {
            if let Some(p) = self.docs.get_mut(&doc) {
                p.refs = p.refs.saturating_sub(dropped);
            }
            self.prune_presence(doc);
        }
        restored
    }

    fn remove_session(&mut self, doc: DocId, key: &SessionKey) {
        if let Some(presence) = self.docs.get_mut(&doc)
            && presence.session.remove(key).is_some()
        {
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

    /// The first of the author's endpoints to pin `doc`. Only the author pins,
    /// so this is one of its devices.
    fn author_endpoint(&self, space: SpaceId, doc: DocId) -> Option<EndpointId> {
        self.resolve_peer(space, doc, false, |e| e.pin)
    }

    /// The latest hold, else the author's endpoint, so an author drives its
    /// objects until someone grabs them.
    fn holder(&self, space: SpaceId, doc: DocId) -> Option<EndpointId> {
        self.resolve_peer(space, doc, true, |e| e.hold)
            .or_else(|| self.author_endpoint(space, doc))
    }

    /// The value at `key`, or `None` for a tombstone or a key nothing wrote.
    fn cell(&self, space: SpaceId, doc: DocId, key: &SessionKey) -> Option<Vec<u8>> {
        self.docs
            .get(&doc)
            .filter(|p| p.space == space)?
            .session
            .get(key)?
            .value
            .clone()
    }

    fn self_snapshot(&self, me: EndpointId) -> Vec<DocSnapshot> {
        let mut by_doc: HashMap<DocId, DocSnapshot> = HashMap::new();
        if let Some(replica) = self.peers.get(&me) {
            for (doc, e) in &replica.docs {
                let Some(presence) = self.docs.get(doc) else {
                    continue;
                };
                by_doc.insert(
                    *doc,
                    DocSnapshot {
                        doc:     *doc,
                        space:   presence.space,
                        pin:     e.pin.and_then(|_| presence.proof.clone()),
                        hold:    e.hold.is_some(),
                        session: Vec::new(),
                    },
                );
            }
        }
        for (doc, p) in &self.docs {
            for (key, c) in &p.session {
                if c.peer != me {
                    continue;
                }
                by_doc
                    .entry(*doc)
                    .or_insert_with(|| DocSnapshot {
                        doc:     *doc,
                        space:   p.space,
                        pin:     None,
                        hold:    false,
                        session: Vec::new(),
                    })
                    .session
                    .push(SessionSnapshot {
                        prim:  key.prim,
                        name:  key.name.to_string(),
                        value: c.value.clone(),
                        at:    c.at,
                    });
            }
        }
        by_doc.into_values().collect()
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
        Self::new(world.get_resource_or_init::<Policy>().clone())
    }
}

impl Replicas {
    #[must_use]
    pub fn new(policy: Policy) -> Self {
        Self(Arc::new(Mutex::new(Inner {
            attribution: Attribution::new(policy),
            peers:       HashMap::new(),
            docs:        HashMap::new(),
            minted:      HashSet::new(),
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
        let _ = tx.try_send(ReplicationMsg::Snapshot(snapshot));
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

    /// Records `peer`'s pin on `doc`. The caller has checked that `peer`
    /// proved `author` and that `proof` names it. Idempotent.
    pub fn add_pin(
        &self,
        peer: EndpointId,
        doc: DocId,
        space: SpaceId,
        author: Did,
        proof: Authorship,
        at: u64,
    ) -> Result<(), PinRefused> {
        let mut inner = self.0.lock();
        let attributed = inner.add_pin(peer, doc, space, author.clone(), proof, at)?;
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

    /// Records a batch of session opinions by `writer`, proving `did`, all or
    /// none.
    pub(crate) fn add_session(
        &self,
        writer: EndpointId,
        did: Option<&Did>,
        doc: DocId,
        space: SpaceId,
        batch: Vec<(SessionKey, Option<Vec<u8>>)>,
        at: u64,
    ) -> Result<(), SessionError> {
        self.0
            .lock()
            .add_session(writer, did, doc, space, batch, at)
    }

    pub(crate) fn remove_session(&self, doc: DocId, key: &SessionKey) {
        self.0.lock().remove_session(doc, key);
    }

    /// Rolls back every cell whose current value came from `peer`, answering
    /// what now stands on each key it touched.
    ///
    /// Pins and holds go with the peer's entity; cells live on the document,
    /// so they need this instead, and the documents composing those keys have
    /// to be told (see [`crate::replication::guards::revert_session`]).
    #[must_use]
    pub(crate) fn revert_writes(&self, peer: EndpointId) -> Vec<Restored> {
        self.0.lock().revert_writes(peer)
    }

    /// Who authors `doc`: the DID its pin proved, or this node for one it
    /// minted. `None` when nothing proved one.
    #[must_use]
    pub fn author(&self, doc: DocId) -> Option<Did> {
        self.0.lock().author_of(doc)
    }

    /// Records that this node minted `doc`, and so authors it.
    pub fn record_minted(&self, doc: DocId) {
        let mut inner = self.0.lock();
        inner.minted.insert(doc);
        let space = inner.docs.get(&doc).map(|p| p.space);
        let author = inner.author_of(doc);
        inner.attribution.reassign(doc, space, author.as_ref());
    }

    /// Forgets a minted document that left the world.
    pub fn forget_minted(&self, doc: DocId) {
        self.0.lock().minted.remove(&doc);
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

    /// What a peer said about one key of one prim, or `None` for a blocked key
    /// or one nothing stated.
    #[must_use]
    pub fn session_value(
        &self,
        space: SpaceId,
        doc: DocId,
        prim: PrimId,
        name: &PropName,
    ) -> Option<Vec<u8>> {
        self.0.lock().cell(
            space,
            doc,
            &SessionKey {
                prim,
                name: name.clone(),
            },
        )
    }

    /// Every key of `prim` holding a live value. A blocked key is stored but
    /// reads as absent, so it is not listed.
    #[must_use]
    pub fn session_keys(&self, space: SpaceId, doc: DocId, prim: PrimId) -> Vec<PropName> {
        let inner = self.0.lock();
        inner
            .docs
            .get(&doc)
            .filter(|p| p.space == space)
            .map_or_default(|presence| {
                presence
                    .session
                    .iter()
                    .filter(|(key, cell)| key.prim == prim && cell.value.is_some())
                    .map(|(key, _)| key.name.clone())
                    .collect()
            })
    }

    /// Remote peers pinning `doc`, which `me` can sync it from, the author's
    /// endpoint first.
    #[must_use]
    pub fn sync_sources(&self, doc: DocId, me: EndpointId) -> Vec<EndpointId> {
        let inner = self.0.lock();
        let Some(space) = inner.docs.get(&doc).map(|p| p.space) else {
            return Vec::new();
        };
        let author_endpoint = inner.author_endpoint(space, doc);
        let mut sources = inner
            .peers
            .iter()
            .filter(|(pid, r)| {
                **pid != me
                    && Some(**pid) != author_endpoint
                    && r.docs.get(&doc).is_some_and(|e| e.pin.is_some())
            })
            .map(|(pid, _)| *pid)
            .collect::<Vec<_>>();
        drop(inner);
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
        self.0
            .lock()
            .peers
            .values()
            .any(|r| r.docs.get(&doc).is_some_and(|e| e.pin.is_some()))
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
                            pin:   e.pin,
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
                let mut session = p
                    .session
                    .iter()
                    .map(|(k, c)| snapshot::SessionCell {
                        prim:   k.prim,
                        name:   k.name.clone(),
                        value:  c.value.clone(),
                        at:     c.at,
                        writer: c.peer,
                    })
                    .collect::<Vec<_>>();
                session.sort_unstable_by(|a, b| (a.prim, &a.name).cmp(&(b.prim, &b.name)));
                snapshot::DocState {
                    doc: *doc,
                    space: p.space,
                    author: p.author.as_ref().map(ToString::to_string),
                    session,
                }
            })
            .collect::<Vec<_>>();
        docs.sort_unstable_by_key(|d| d.doc.0);
        drop(inner);
        snapshot::ReplicaSnapshot { peers, docs }
    }
}

#[cfg(test)] mod tests;
