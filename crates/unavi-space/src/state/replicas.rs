use std::{
    collections::{
        HashMap,
        hash_map::Entry,
    },
    sync::Arc,
};

use bevy::prelude::Resource;
use hsd::id::DocId;
use iroh::EndpointId;
use parking_lot::Mutex;
use unavi_policy::{
    quota::{
        Quota,
        Stock,
        StockHold,
    },
    registry::Policy,
};

#[cfg(feature = "devtools")] use crate::state::debug;
use crate::{
    quota::{
        Viewer,
        document_quota,
        reassign_document_in_space,
    },
    state::{
        cell::{
            Cell,
            KvError,
            cell_bytes,
        },
        message::{
            DocSnapshot,
            KvSnapshot,
            StateMsg,
        },
    },
};

pub const KV_KEY_MAX_BYTES: usize = 256;

/// One peer's contribution to a document: its pin (timestamped, so the oldest
/// pin owns the doc) and its latest object-authority claim.
#[derive(Default)]
struct PeerDocEntry {
    pin:       Option<u64>,
    authority: Option<u64>,
}

impl PeerDocEntry {
    const fn is_empty(&self) -> bool {
        self.pin.is_none() && self.authority.is_none()
    }
}

#[derive(Default)]
struct PeerReplica {
    docs: HashMap<DocId, PeerDocEntry>,
}

/// Per-document state shared across peers. `refs` counts the live pins,
/// authority claims and KV cells keeping the presence alive; `_doc_hold`
/// charges one `Documents` unit while the doc is known locally.
struct DocPresence {
    space:     DocId,
    _doc_hold: StockHold,
    kv:        HashMap<String, Cell>,
    refs:      u32,
}

/// A registered delta stream's cancel token.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct StreamToken(u64);

/// Every peer's replicated state (self included), per-document presence, and
/// the live delta senders. Mutated only by the RAII guards in
/// [`crate::state::entities`].
#[derive(Default)]
struct Inner {
    peers:      HashMap<EndpointId, PeerReplica>,
    docs:       HashMap<DocId, DocPresence>,
    senders:    HashMap<StreamToken, async_channel::Sender<StateMsg>>,
    next_token: u64,
}

impl Inner {
    fn broadcast(&mut self, msg: &StateMsg) {
        self.senders
            .retain(|_, tx| tx.try_send(msg.clone()).is_ok());
    }

    /// Ensures a [`DocPresence`] for `doc`, charging one `Documents` unit on
    /// first sight. Returns `false` when that charge is refused.
    fn ensure_presence(&mut self, doc: DocId, space: DocId, quota: &Arc<Quota>) -> bool {
        if let Entry::Vacant(v) = self.docs.entry(doc) {
            let Ok(hold) = quota.hold(Stock::Documents, 1) else {
                return false;
            };
            v.insert(DocPresence {
                space,
                _doc_hold: hold,
                kv: HashMap::new(),
                refs: 0,
            });
        }
        true
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
        }
    }

    /// Drops one reference to `doc`, releasing its presence (and the
    /// `Documents` hold) once nothing references it.
    fn dec_ref(&mut self, doc: DocId) {
        if let Entry::Occupied(mut p) = self.docs.entry(doc) {
            p.get_mut().refs = p.get().refs.saturating_sub(1);
            if p.get().refs == 0 {
                p.remove();
            }
        }
    }

    /// Removes a peer's entry for `doc` once it holds no data.
    fn prune_entry(&mut self, peer: EndpointId, doc: DocId) {
        if let Some(replica) = self.peers.get_mut(&peer)
            && let Entry::Occupied(e) = replica.docs.entry(doc)
            && e.get().is_empty()
        {
            e.remove();
        }
    }

    fn add_pin(
        &mut self,
        peer: EndpointId,
        doc: DocId,
        space: DocId,
        at: u64,
        quota: &Arc<Quota>,
        reassign: &mut Vec<(DocId, DocId)>,
    ) -> bool {
        if !self.ensure_presence(doc, space, quota) {
            return false;
        }
        let entry = self
            .peers
            .entry(peer)
            .or_default()
            .docs
            .entry(doc)
            .or_default();
        if entry.pin.is_none() {
            entry.pin = Some(at);
            self.inc_ref(doc);
            reassign.push((doc, space));
        }
        true
    }

    fn remove_pin(&mut self, peer: EndpointId, doc: DocId, reassign: &mut Vec<(DocId, DocId)>) {
        if let Some(entry) = self.peers.get_mut(&peer).and_then(|r| r.docs.get_mut(&doc))
            && entry.pin.take().is_some()
        {
            if let Some(space) = self.docs.get(&doc).map(|p| p.space) {
                reassign.push((doc, space));
            }
            self.dec_ref(doc);
            self.prune_entry(peer, doc);
        }
    }

    fn add_authority(
        &mut self,
        peer: EndpointId,
        doc: DocId,
        space: DocId,
        at: u64,
        quota: &Arc<Quota>,
    ) -> bool {
        if !self.ensure_presence(doc, space, quota) {
            return false;
        }
        let entry = self
            .peers
            .entry(peer)
            .or_default()
            .docs
            .entry(doc)
            .or_default();
        let was_set = entry.authority.is_some();
        entry.authority = Some(at);
        if !was_set {
            self.inc_ref(doc);
        }
        true
    }

    fn remove_authority(&mut self, peer: EndpointId, doc: DocId) {
        if let Some(entry) = self.peers.get_mut(&peer).and_then(|r| r.docs.get_mut(&doc))
            && entry.authority.take().is_some()
        {
            self.dec_ref(doc);
            self.prune_entry(peer, doc);
        }
    }

    /// Writes a cell on `doc`, or refuses when `peer` may not write it.
    ///
    /// Who may write is the whole difference between a space-owned document and
    /// a peer-owned one. Where the cell is stored is not: both land on the
    /// document.
    fn add_kv(
        &mut self,
        peer: EndpointId,
        doc: DocId,
        space: DocId,
        key: String,
        value: Option<Vec<u8>>,
        at: u64,
        quota: &Arc<Quota>,
    ) -> Result<(), KvError> {
        if !self.is_space_owned(space, doc) && self.owner(space, doc) != Some(peer) {
            return Err(KvError::NotOwner);
        }
        if !self.ensure_presence(doc, space, quota) {
            return Err(KvError::QuotaExceeded);
        }

        let new_bytes = cell_bytes(&key, value.as_deref());
        let inserted = {
            let presence = self.docs.get_mut(&doc).expect("presence ensured");
            match presence.kv.entry(key) {
                Entry::Occupied(mut o) => {
                    if at < o.get().at {
                        Ok(false)
                    } else if let Ok(hold) = quota.hold(Stock::KvMemory, new_bytes) {
                        let cell = o.get_mut();
                        // The outgoing version keeps its own hold and becomes
                        // the fallback; whatever it replaces is dropped,
                        // releasing that hold. Depth stays one.
                        let displaced = Cell {
                            at:    cell.at,
                            peer:  cell.peer,
                            value: cell.value.take(),
                            hold:  std::mem::replace(&mut cell.hold, hold),
                            prev:  None,
                        };
                        cell.at = at;
                        cell.peer = peer;
                        cell.value = value;
                        cell.prev = Some(Box::new(displaced));
                        Ok(false)
                    } else {
                        Err(KvError::QuotaExceeded)
                    }
                }
                Entry::Vacant(v) => quota.hold(Stock::KvMemory, new_bytes).map_or(
                    Err(KvError::QuotaExceeded),
                    |hold| {
                        v.insert(Cell {
                            at,
                            peer,
                            value,
                            hold,
                            prev: None,
                        });
                        Ok(true)
                    },
                ),
            }
        };

        match inserted {
            Ok(true) => self.inc_ref(doc),
            Ok(false) => {}
            Err(err) => {
                self.prune_presence(doc);
                return Err(err);
            }
        }
        Ok(())
    }

    /// Restores each neutral cell `peer` last wrote to its prior version, or
    /// drops it when the prior version was also theirs — a peer cannot leave
    /// its own earlier write behind as the fallback.
    fn revert_writes(&mut self, peer: EndpointId) -> usize {
        let mut emptied = Vec::new();
        let mut reverted = 0;

        for (&doc, presence) in &mut self.docs {
            let mut dropped = 0;
            presence.kv.retain(|_, cell| {
                if cell.peer != peer {
                    return true;
                }
                reverted += 1;
                // Depth is one, so at most one fallback needs examining.
                match cell.prev.take() {
                    Some(prev) if prev.peer != peer => {
                        *cell = *prev;
                        true
                    }
                    _ => {
                        dropped += 1;
                        false
                    }
                }
            });
            if dropped > 0 {
                emptied.push((doc, dropped));
            }
        }

        for (doc, dropped) in emptied {
            for _ in 0..dropped {
                self.dec_ref(doc);
            }
        }
        reverted
    }

    fn remove_kv(&mut self, doc: DocId, key: &str) {
        if let Some(presence) = self.docs.get_mut(&doc)
            && presence.kv.remove(key).is_some()
        {
            self.dec_ref(doc);
        }
    }

    /// Owner of `doc`: the oldest valid pin, breaking ties by peer id. Resolved
    /// per-client; ownership migrates to the next-oldest pinner when an owner
    /// leaves.
    fn resolve_peer(
        &self,
        space: DocId,
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
        let tiebreak = |a: &(u64, EndpointId), b: &(u64, EndpointId)| {
            a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1))
        };
        if newest {
            candidates.max_by(tiebreak)
        } else {
            candidates.min_by(tiebreak)
        }
        .map(|(_, pid)| pid)
    }

    fn owner(&self, space: DocId, doc: DocId) -> Option<EndpointId> {
        self.resolve_peer(space, doc, false, |e| e.pin)
    }

    /// Transform authority for `doc`: the latest explicit claimer, or the
    /// document's owner when no one has claimed, so an owner drives its objects
    /// by default until a peer grabs them.
    fn authority(&self, space: DocId, doc: DocId) -> Option<EndpointId> {
        self.resolve_peer(space, doc, true, |e| e.authority)
            .or_else(|| self.owner(space, doc))
    }

    /// The value at `key`, or `None` for a tombstone or a key nothing wrote.
    ///
    /// One cell per key, so the last-write-wins merge already happened at write
    /// time and there is nothing to resolve here.
    fn cell(&self, space: DocId, doc: DocId, key: &str) -> Option<Vec<u8>> {
        self.docs
            .get(&doc)
            .filter(|p| p.space == space)?
            .kv
            .get(key)?
            .value
            .clone()
    }

    /// Whether `doc` is space-owned: the space's base document, or a doc with
    /// no pin-owner. Space-owned documents accept KV writes from any peer;
    /// peer-owned documents only from their owner.
    fn is_space_owned(&self, space: DocId, doc: DocId) -> bool {
        doc == space || self.owner(space, doc).is_none()
    }

    fn self_snapshot(&self, me: EndpointId) -> Vec<DocSnapshot> {
        let mut by_doc: HashMap<DocId, DocSnapshot> = HashMap::new();
        if let Some(replica) = self.peers.get(&me) {
            for (doc, e) in &replica.docs {
                let Some(space) = self.docs.get(doc).map(|p| p.space) else {
                    continue;
                };
                by_doc.insert(
                    *doc,
                    DocSnapshot {
                        doc: *doc,
                        space,
                        pin: e.pin,
                        authority: e.authority,
                        kv: Vec::new(),
                    },
                );
            }
        }
        for (doc, p) in &self.docs {
            for (key, c) in &p.kv {
                if c.peer != me {
                    continue;
                }
                by_doc
                    .entry(*doc)
                    .or_insert_with(|| DocSnapshot {
                        doc:       *doc,
                        space:     p.space,
                        pin:       None,
                        authority: None,
                        kv:        Vec::new(),
                    })
                    .kv
                    .push(KvSnapshot {
                        key:   key.clone(),
                        value: c.value.clone(),
                        at:    c.at,
                    });
            }
        }
        by_doc.into_values().collect()
    }
}

/// Runs `reassign` against the document quotas after the state lock is
/// released, since the owner resolver re-enters the store.
fn settle_reassigns(
    policy: &Policy,
    replicas: &Replicas,
    viewer: Option<Viewer>,
    reassign: Vec<(DocId, DocId)>,
) {
    for (doc, space) in reassign {
        reassign_document_in_space(policy, replicas, viewer, doc, space);
    }
}

/// Every peer's replicated view of the documents in play: who pins what, who
/// holds transform authority, and the KV cells the documents carry.
///
/// One value, constructed once per app.
#[derive(Resource, Clone)]
pub struct Replicas(Arc<Mutex<Inner>>);

impl Default for Replicas {
    fn default() -> Self {
        Self::new()
    }
}

impl Replicas {
    #[must_use]
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Inner::default())))
    }

    /// Registers a delta stream, returning its cancel token and a receiver
    /// whose first message is a full snapshot of `me`'s state.
    #[must_use]
    pub fn register_stream(
        &self,
        me: EndpointId,
    ) -> (StreamToken, async_channel::Receiver<StateMsg>) {
        let (tx, rx) = async_channel::unbounded();
        let mut inner = self.0.lock();
        let snapshot = inner.self_snapshot(me);
        let _ = tx.try_send(StateMsg::Snapshot(snapshot));
        let token = StreamToken(inner.next_token);
        inner.next_token += 1;
        inner.senders.insert(token, tx);
        drop(inner);
        (token, rx)
    }

    pub fn unregister_stream(&self, token: StreamToken) {
        self.0.lock().senders.remove(&token);
    }

    pub fn broadcast(&self, msg: &StateMsg) {
        self.0.lock().broadcast(msg);
    }

    /// Adds a peer's pin on `doc`. Returns `false` if the document quota
    /// refuses the presence. Idempotent: a repeat pin from the same peer is a
    /// no-op.
    #[must_use]
    pub fn add_pin(
        &self,
        policy: &Policy,
        viewer: Option<Viewer>,
        peer: EndpointId,
        doc: DocId,
        space: DocId,
        at: u64,
    ) -> bool {
        let quota = document_quota(policy, self, viewer, doc);
        let mut reassign = Vec::new();
        let mut inner = self.0.lock();
        let ok = inner.add_pin(peer, doc, space, at, &quota, &mut reassign);
        drop(inner);
        settle_reassigns(policy, self, viewer, reassign);
        ok
    }

    pub fn remove_pin(
        &self,
        policy: &Policy,
        viewer: Option<Viewer>,
        peer: EndpointId,
        doc: DocId,
    ) {
        let mut reassign = Vec::new();
        let mut inner = self.0.lock();
        inner.remove_pin(peer, doc, &mut reassign);
        drop(inner);
        settle_reassigns(policy, self, viewer, reassign);
    }

    /// Adds or refreshes a peer's authority claim on `doc`. Returns `false` if
    /// the document quota refuses the presence.
    #[must_use]
    pub fn add_authority(
        &self,
        policy: &Policy,
        viewer: Option<Viewer>,
        peer: EndpointId,
        doc: DocId,
        space: DocId,
        at: u64,
    ) -> bool {
        let quota = document_quota(policy, self, viewer, doc);
        self.0.lock().add_authority(peer, doc, space, at, &quota)
    }

    pub fn remove_authority(&self, peer: EndpointId, doc: DocId) {
        self.0.lock().remove_authority(peer, doc);
    }

    /// Applies a KV write for `peer`.
    ///
    /// Rejects writes to a peer-owned document by a non-owner, and drops
    /// writes that exceed quota.
    pub fn add_kv(
        &self,
        policy: &Policy,
        viewer: Option<Viewer>,
        peer: EndpointId,
        doc: DocId,
        space: DocId,
        key: String,
        value: Option<Vec<u8>>,
        at: u64,
    ) -> Result<(), KvError> {
        let quota = document_quota(policy, self, viewer, doc);
        self.0
            .lock()
            .add_kv(peer, doc, space, key, value, at, &quota)
    }

    pub fn remove_kv(&self, doc: DocId, key: &str) {
        self.0.lock().remove_kv(doc, key);
    }

    /// Rolls back every cell whose current value came from `peer`, returning
    /// how many changed.
    ///
    /// The undo that pins and authority claims get from the peer's entity
    /// cascade. Cells live on the document rather than the peer, so they need
    /// this instead.
    #[must_use]
    pub fn revert_writes(&self, peer: EndpointId) -> usize {
        self.0.lock().revert_writes(peer)
    }

    #[must_use]
    pub fn owner(&self, space: DocId, doc: DocId) -> Option<EndpointId> {
        self.0.lock().owner(space, doc)
    }

    #[must_use]
    pub fn authority(&self, space: DocId, doc: DocId) -> Option<EndpointId> {
        self.0.lock().authority(space, doc)
    }

    #[must_use]
    pub fn is_owner(&self, space: DocId, doc: DocId, me: EndpointId) -> bool {
        self.owner(space, doc) == Some(me)
    }

    #[must_use]
    pub fn is_authority(&self, space: DocId, doc: DocId, me: EndpointId) -> bool {
        self.authority(space, doc) == Some(me)
    }

    #[must_use]
    pub fn has_doc(&self, space: DocId, doc: DocId) -> bool {
        self.0
            .lock()
            .docs
            .get(&doc)
            .is_some_and(|p| p.space == space)
    }

    #[must_use]
    pub fn kv_get(&self, space: DocId, doc: DocId, key: &str) -> Option<Vec<u8>> {
        self.0.lock().cell(space, doc, key)
    }

    /// Every key holding a live value. A tombstone is stored but reads as
    /// absent, so it is not listed.
    #[must_use]
    pub fn kv_keys(&self, space: DocId, doc: DocId) -> Vec<String> {
        let inner = self.0.lock();
        let Some(presence) = inner.docs.get(&doc).filter(|p| p.space == space) else {
            return Vec::new();
        };
        presence
            .kv
            .iter()
            .filter(|(_, cell)| cell.value.is_some())
            .map(|(key, _)| key.clone())
            .collect()
    }

    #[must_use]
    pub fn kv_total_bytes(&self, space: DocId, doc: DocId) -> usize {
        let inner = self.0.lock();
        let Some(presence) = inner.docs.get(&doc).filter(|p| p.space == space) else {
            return 0;
        };
        presence
            .kv
            .iter()
            .filter_map(|(key, cell)| cell.value.as_ref().map(|v| key.len() + v.len()))
            .sum()
    }

    /// Remote peers that hold `doc`, those `me` can sync the record from.
    /// Excludes `me`; the owner is listed first as the freshest source.
    #[must_use]
    pub fn holders(&self, doc: DocId, me: EndpointId) -> Vec<EndpointId> {
        let inner = self.0.lock();
        let Some(space) = inner.docs.get(&doc).map(|p| p.space) else {
            return Vec::new();
        };
        let owner = inner.owner(space, doc);
        let mut holders = inner
            .peers
            .iter()
            .filter(|(pid, r)| {
                **pid != me
                    && Some(**pid) != owner
                    && r.docs.get(&doc).is_some_and(|e| e.pin.is_some())
            })
            .map(|(pid, _)| *pid)
            .collect::<Vec<_>>();
        drop(inner);
        if let Some(owner) = owner.filter(|o| *o != me) {
            holders.insert(0, owner);
        }
        holders
    }

    /// The space `doc` is pinned in, per any peer's replica. Lets membership
    /// resolve a synced doc's space without a local ownership claim.
    #[must_use]
    pub fn space_of(&self, doc: DocId) -> Option<DocId> {
        self.0.lock().docs.get(&doc).map(|p| p.space)
    }

    /// Whether any peer currently pins `doc`. Drives the scene's fetch/despawn
    /// of tracked documents.
    #[must_use]
    pub fn is_pinned(&self, doc: DocId) -> bool {
        self.0
            .lock()
            .peers
            .values()
            .any(|r| r.docs.get(&doc).is_some_and(|e| e.pin.is_some()))
    }

    /// A deterministically ordered snapshot, so the panel can fingerprint it
    /// and rebuild only on change.
    #[cfg(feature = "devtools")]
    #[must_use]
    pub fn snapshot(&self) -> debug::DebugSnapshot {
        let inner = self.0.lock();
        let mut peers = inner
            .peers
            .iter()
            .map(|(pid, r)| {
                let mut docs = r
                    .docs
                    .iter()
                    .map(|(doc, e)| debug::DebugPeerDoc {
                        doc:       *doc,
                        space:     inner.docs.get(doc).map_or(*doc, |p| p.space),
                        pin:       e.pin,
                        authority: e.authority,
                    })
                    .collect::<Vec<_>>();
                docs.sort_unstable_by_key(|d| d.doc.0);
                debug::DebugPeer { peer: *pid, docs }
            })
            .collect::<Vec<_>>();
        peers.sort_unstable_by_key(|p| p.peer);

        let mut docs = inner
            .docs
            .iter()
            .filter(|(_, p)| !p.kv.is_empty())
            .map(|(doc, p)| {
                let mut kv =
                    p.kv.iter()
                        .map(|(k, c)| debug::DebugKv {
                            key:    k.clone(),
                            value:  c.value.clone(),
                            at:     c.at,
                            writer: c.peer,
                        })
                        .collect::<Vec<_>>();
                kv.sort_unstable_by(|a, b| a.key.cmp(&b.key));
                debug::DebugDoc {
                    doc: *doc,
                    space: p.space,
                    kv,
                }
            })
            .collect::<Vec<_>>();
        docs.sort_unstable_by_key(|d| d.doc.0);
        drop(inner);
        debug::DebugSnapshot { peers, docs }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(seed: &[u8]) -> DocId {
        DocId(*blake3::hash(seed).as_bytes())
    }

    /// A distinct, valid endpoint id per seed. Arbitrary bytes are not a curve
    /// point, so a key has to be derived rather than written down.
    fn peer(seed: u8) -> EndpointId {
        iroh::SecretKey::from_bytes(&[seed; 32]).public()
    }

    #[test]
    fn pin_owner_is_oldest_and_releases() {
        let replicas = Replicas::new();
        let policy = Policy::new();
        let space = doc(b"oldest-space");
        let doc = doc(b"oldest-doc");
        let early = peer(1);
        let late = peer(2);

        assert!(replicas.add_pin(&policy, None, late, doc, space, 20));
        assert_eq!(replicas.owner(space, doc), Some(late));
        assert!(replicas.add_pin(&policy, None, early, doc, space, 10));
        assert_eq!(replicas.owner(space, doc), Some(early));

        replicas.remove_pin(&policy, None, early, doc);
        assert_eq!(replicas.owner(space, doc), Some(late));
        replicas.remove_pin(&policy, None, late, doc);
        assert_eq!(replicas.owner(space, doc), None);
        assert!(!replicas.has_doc(space, doc));
    }

    #[test]
    fn authority_latest_and_defaults_to_owner() {
        let replicas = Replicas::new();
        let policy = Policy::new();
        let space = doc(b"auth-space");
        let doc = doc(b"auth-doc");
        let owner_peer = peer(1);
        let grabber = peer(2);

        assert!(replicas.add_pin(&policy, None, owner_peer, doc, space, 10));
        assert!(replicas.add_pin(&policy, None, grabber, doc, space, 20));
        assert_eq!(replicas.authority(space, doc), Some(owner_peer));

        assert!(replicas.add_authority(&policy, None, grabber, doc, space, 200));
        assert_eq!(replicas.authority(space, doc), Some(grabber));
        assert_eq!(replicas.owner(space, doc), Some(owner_peer));

        replicas.remove_authority(grabber, doc);
        assert_eq!(replicas.authority(space, doc), Some(owner_peer));
    }

    /// The document outlives its first owner through the next-oldest pin, so
    /// its state has to outlive them too. Cells kept under the owner's replica
    /// went with them, leaving the content behind with an empty KV.
    #[test]
    fn kv_survives_the_owner_leaving() {
        let replicas = Replicas::new();
        let policy = Policy::new();
        let space = doc(b"handoff-space");
        let doc = doc(b"handoff-doc");
        let (first, second) = (peer(1), peer(2));

        assert!(replicas.add_pin(&policy, None, first, doc, space, 10));
        assert!(replicas.add_pin(&policy, None, second, doc, space, 20));
        assert_eq!(replicas.owner(space, doc), Some(first));

        replicas
            .add_kv(
                &policy,
                None,
                first,
                doc,
                space,
                "colour".into(),
                Some(b"red".to_vec()),
                1,
            )
            .expect("the owner may write");

        replicas.remove_pin(&policy, None, first, doc);

        assert_eq!(
            replicas.owner(space, doc),
            Some(second),
            "ownership hands off"
        );
        assert_eq!(
            replicas.kv_get(space, doc, "colour").as_deref(),
            Some(&b"red"[..]),
            "the new owner inherits the state, not an empty document"
        );

        replicas
            .add_kv(
                &policy,
                None,
                second,
                doc,
                space,
                "colour".into(),
                Some(b"blue".to_vec()),
                2,
            )
            .expect("the new owner may write what it inherited");
        assert_eq!(
            replicas.kv_get(space, doc, "colour").as_deref(),
            Some(&b"blue"[..])
        );
    }

    #[test]
    fn neutral_kv_persists_across_owned_kv() {
        let replicas = Replicas::new();
        let policy = Policy::new();
        let space = doc(b"kv-space");
        let alice = peer(2);

        assert_eq!(
            replicas.add_kv(
                &policy,
                None,
                alice,
                space,
                space,
                "link".into(),
                Some(b"dest".to_vec()),
                1
            ),
            Ok(())
        );
        assert_eq!(
            replicas.kv_get(space, space, "link").as_deref(),
            Some(&b"dest"[..])
        );

        replicas.remove_kv(space, "missing");
        assert_eq!(
            replicas.kv_get(space, space, "link").as_deref(),
            Some(&b"dest"[..])
        );

        replicas.remove_kv(space, "link");
        assert_eq!(replicas.kv_get(space, space, "link"), None);
        assert!(!replicas.has_doc(space, space));
    }

    #[test]
    fn reverting_a_peer_restores_the_value_it_overwrote() {
        let replicas = Replicas::new();
        let policy = Policy::new();
        let space = doc(b"revert-space");
        let (alice, mallory) = (peer(2), peer(3));

        replicas
            .add_kv(
                &policy,
                None,
                alice,
                space,
                space,
                "sign".into(),
                Some(b"welcome".to_vec()),
                1,
            )
            .expect("alice writes the sign");
        replicas
            .add_kv(
                &policy,
                None,
                mallory,
                space,
                space,
                "sign".into(),
                Some(b"defaced".to_vec()),
                2,
            )
            .expect("mallory defaces it");
        assert_eq!(
            replicas.kv_get(space, space, "sign").as_deref(),
            Some(&b"defaced"[..])
        );

        assert_eq!(replicas.revert_writes(mallory), 1);
        assert_eq!(
            replicas.kv_get(space, space, "sign").as_deref(),
            Some(&b"welcome"[..]),
            "blocking must put back what the blocked peer wrote over"
        );
    }

    #[test]
    fn reverting_drops_a_cell_the_peer_created() {
        let replicas = Replicas::new();
        let policy = Policy::new();
        let space = doc(b"revert-new-space");
        let mallory = peer(3);

        replicas
            .add_kv(
                &policy,
                None,
                mallory,
                space,
                space,
                "spam".into(),
                Some(b"x".to_vec()),
                1,
            )
            .expect("mallory writes a new cell");

        assert_eq!(replicas.revert_writes(mallory), 1);
        assert_eq!(
            replicas.kv_get(space, space, "spam"),
            None,
            "a cell with no prior version has nothing to fall back to"
        );
        assert!(
            !replicas.has_doc(space, space),
            "dropping the last cell must release the document presence"
        );
    }

    #[test]
    fn a_peer_cannot_leave_its_own_earlier_write_as_the_fallback() {
        let replicas = Replicas::new();
        let policy = Policy::new();
        let space = doc(b"revert-own-space");
        let mallory = peer(3);

        replicas
            .add_kv(
                &policy,
                None,
                mallory,
                space,
                space,
                "sign".into(),
                Some(b"first".to_vec()),
                1,
            )
            .expect("first");
        replicas
            .add_kv(
                &policy,
                None,
                mallory,
                space,
                space,
                "sign".into(),
                Some(b"second".to_vec()),
                2,
            )
            .expect("second");

        assert_eq!(replicas.revert_writes(mallory), 1);
        assert_eq!(
            replicas.kv_get(space, space, "sign"),
            None,
            "falling back to the blocked peer's own earlier write undoes nothing"
        );
    }

    #[test]
    fn reverting_leaves_another_peers_cells_alone() {
        let replicas = Replicas::new();
        let policy = Policy::new();
        let space = doc(b"revert-other-space");
        let (alice, mallory) = (peer(2), peer(3));

        replicas
            .add_kv(
                &policy,
                None,
                alice,
                space,
                space,
                "keep".into(),
                Some(b"mine".to_vec()),
                1,
            )
            .expect("alice");

        assert_eq!(replicas.revert_writes(mallory), 0);
        assert_eq!(
            replicas.kv_get(space, space, "keep").as_deref(),
            Some(&b"mine"[..])
        );
    }

    #[test]
    fn owned_kv_gated_by_ownership() {
        let replicas = Replicas::new();
        let policy = Policy::new();
        let space = doc(b"perm-space");
        let owner_peer = peer(1);
        let other = peer(2);
        let doc = doc(b"perm-doc");

        assert!(replicas.add_pin(&policy, None, owner_peer, doc, space, 1));
        assert_eq!(
            replicas.add_kv(
                &policy,
                None,
                owner_peer,
                doc,
                space,
                "k".into(),
                Some(b"v".to_vec()),
                2
            ),
            Ok(())
        );
        assert!(matches!(
            replicas.add_kv(
                &policy,
                None,
                other,
                doc,
                space,
                "k".into(),
                Some(b"v".to_vec()),
                3
            ),
            Err(KvError::NotOwner)
        ));
        assert_eq!(replicas.kv_get(space, doc, "k").as_deref(), Some(&b"v"[..]));
    }

    #[test]
    fn refs_release_presence_and_quota() {
        let replicas = Replicas::new();
        let policy = Policy::new();
        let space = doc(b"refund-space");
        let doc = doc(b"refund-doc");
        let peer = peer(11);

        assert!(replicas.add_pin(&policy, None, peer, doc, space, 1));
        assert_eq!(
            replicas.add_kv(
                &policy,
                None,
                peer,
                doc,
                space,
                "k".into(),
                Some(b"value".to_vec()),
                2
            ),
            Ok(())
        );
        let quota = document_quota(&policy, &replicas, None, doc);
        assert!(quota.usage(Stock::KvMemory) > 0);
        assert_eq!(quota.usage(Stock::Documents), 1);

        replicas.remove_kv(doc, "k");
        replicas.remove_pin(&policy, None, peer, doc);
        assert_eq!(quota.usage(Stock::KvMemory), 0);
        assert_eq!(quota.usage(Stock::Documents), 0);
        assert!(!replicas.has_doc(space, doc));
    }
}
