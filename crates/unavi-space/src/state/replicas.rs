use std::{
    collections::{
        HashMap,
        HashSet,
        hash_map::Entry,
    },
    sync::{
        Arc,
        LazyLock,
        atomic::{
            AtomicU64,
            Ordering,
        },
    },
    time::{
        SystemTime,
        UNIX_EPOCH,
    },
};

use hsd::id::DocId;
use iroh_docs::NamespaceId;
use parking_lot::Mutex;
use unavi_quota::{
    Quota,
    Stock,
    StockHold,
};

use crate::{
    peer::self_peer_id,
    quota::{
        document_quota,
        reassign_document_in_space,
    },
    state::message::{
        DocSnapshot,
        KvSnapshot,
        StateMsg,
    },
};

pub type PeerId = [u8; 32];

pub const KV_KEY_MAX_BYTES: usize = 256;

/// Caps peer-supplied timestamps to within the clock skew of local time, so a
/// forged future `at` cannot pin ownership/authority or win KV merges forever.
const MAX_CLOCK_SKEW_MILLIS: u64 = 5 * 60 * 1000;

/// A KV cell merged last-write-wins. `value: None` is a tombstone retained so a
/// delete keeps winning over an older live write on another peer. `_hold`
/// charges the cell's bytes to the document's quota for exactly its lifetime,
/// so dropping the cell refunds with no separate bookkeeping.
struct OwnedCell {
    at:    u64,
    value: Option<Vec<u8>>,
    hold:  StockHold,
}

/// A space-owned (neutral) KV cell. Stored on the document rather than under
/// any peer, so it persists once written until explicitly deleted or the
/// document is forgotten by everyone; `peer` is retained only as the merge
/// tiebreak.
struct NeutralCell {
    at:    u64,
    peer:  PeerId,
    value: Option<Vec<u8>>,
    hold:  StockHold,
    /// Exactly one prior version, which is what makes "revert everything peer
    /// X wrote" a scan of the neutral map rather than a general undo log.
    /// Neutral cells outlive their writer's disconnect, so without this they
    /// are the one live surface a block cannot reach.
    prev:  Option<Box<Self>>,
}

fn cell_bytes(key: &str, value: Option<&[u8]>) -> u64 {
    (key.len() + value.map_or(0, <[u8]>::len)) as u64
}

/// One peer's contribution to a document: its pin (timestamped, so the oldest
/// pin owns the doc), its latest object-authority claim, and the owner-authored
/// KV it wrote while owning the doc.
#[derive(Default)]
struct PeerDocEntry {
    pin:       Option<u64>,
    authority: Option<u64>,
    kv:        HashMap<String, OwnedCell>,
}

impl PeerDocEntry {
    fn is_empty(&self) -> bool {
        self.pin.is_none() && self.authority.is_none() && self.kv.is_empty()
    }
}

#[derive(Default)]
struct PeerReplica {
    docs: HashMap<NamespaceId, PeerDocEntry>,
}

/// Per-document state shared across peers: its space, its quota (charged to the
/// owner, migrated on handoff), a `Documents` hold held while the doc is known
/// locally, its space-owned KV, and a reference count of the live state data
/// (pins, authority claims, and KV cells) keeping the presence alive.
struct DocPresence {
    space:      NamespaceId,
    _doc_hold:  StockHold,
    neutral_kv: HashMap<String, NeutralCell>,
    refs:       u32,
}

/// Holds every peer's replicated state (self included), per-document presence,
/// and the live delta senders. State is mutated only by the RAII guards in
/// [`crate::state::entities`]; everything here is a read index plus the
/// low-level add/remove those guards drive.
#[derive(Default)]
struct ReplicatedPeerState {
    peers:   HashMap<PeerId, PeerReplica>,
    docs:    HashMap<NamespaceId, DocPresence>,
    senders: HashMap<u64, async_channel::Sender<StateMsg>>,
}

impl ReplicatedPeerState {
    fn broadcast(&mut self, msg: &StateMsg) {
        self.senders
            .retain(|_, tx| tx.try_send(msg.clone()).is_ok());
    }

    /// Ensures a [`DocPresence`] for `doc`, charging one `Documents` unit on
    /// first sight. Returns `false` when that charge is refused.
    fn ensure_presence(
        &mut self,
        doc: NamespaceId,
        space: NamespaceId,
        quota: &Arc<Quota>,
    ) -> bool {
        if let Entry::Vacant(v) = self.docs.entry(doc) {
            let Ok(hold) = quota.hold(Stock::Documents, 1) else {
                return false;
            };
            v.insert(DocPresence {
                space,
                _doc_hold: hold,
                neutral_kv: HashMap::new(),
                refs: 0,
            });
        }
        true
    }

    fn inc_ref(&mut self, doc: NamespaceId) {
        if let Some(p) = self.docs.get_mut(&doc) {
            p.refs += 1;
        }
    }

    fn prune_presence(&mut self, doc: NamespaceId) {
        if let Entry::Occupied(p) = self.docs.entry(doc)
            && p.get().refs == 0
        {
            p.remove();
        }
    }

    /// Drops one reference to `doc`, releasing its presence (and the
    /// `Documents` hold) once nothing references it.
    fn dec_ref(&mut self, doc: NamespaceId) {
        if let Entry::Occupied(mut p) = self.docs.entry(doc) {
            p.get_mut().refs = p.get().refs.saturating_sub(1);
            if p.get().refs == 0 {
                p.remove();
            }
        }
    }

    /// Removes a peer's entry for `doc` once it holds no data.
    fn prune_entry(&mut self, peer: PeerId, doc: NamespaceId) {
        if let Some(replica) = self.peers.get_mut(&peer)
            && let Entry::Occupied(e) = replica.docs.entry(doc)
            && e.get().is_empty()
        {
            e.remove();
        }
    }

    fn add_pin(
        &mut self,
        peer: PeerId,
        doc: NamespaceId,
        space: NamespaceId,
        at: u64,
        quota: &Arc<Quota>,
        reassign: &mut Vec<(NamespaceId, NamespaceId)>,
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

    fn remove_pin(
        &mut self,
        peer: PeerId,
        doc: NamespaceId,
        reassign: &mut Vec<(NamespaceId, NamespaceId)>,
    ) {
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
        peer: PeerId,
        doc: NamespaceId,
        space: NamespaceId,
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

    fn remove_authority(&mut self, peer: PeerId, doc: NamespaceId) {
        if let Some(entry) = self.peers.get_mut(&peer).and_then(|r| r.docs.get_mut(&doc))
            && entry.authority.take().is_some()
        {
            self.dec_ref(doc);
            self.prune_entry(peer, doc);
        }
    }

    fn add_kv(
        &mut self,
        peer: PeerId,
        doc: NamespaceId,
        space: NamespaceId,
        key: String,
        value: Option<Vec<u8>>,
        at: u64,
        quota: &Arc<Quota>,
    ) -> Result<KvPlacement, KvError> {
        let neutral = self.is_space_owned(space, doc);
        if !neutral && self.owner(space, doc) != Some(peer) {
            return Err(KvError::NotOwner);
        }
        if !self.ensure_presence(doc, space, quota) {
            return Err(KvError::QuotaExceeded);
        }
        let new_bytes = cell_bytes(&key, value.as_deref());
        if neutral {
            let inserted = {
                let presence = self.docs.get_mut(&doc).expect("presence ensured");
                match presence.neutral_kv.entry(key) {
                    Entry::Occupied(mut o) => {
                        if at < o.get().at {
                            Ok(false)
                        } else if let Ok(hold) = quota.hold(Stock::KvMemory, new_bytes) {
                            let cell = o.get_mut();
                            // The outgoing version keeps its own hold and
                            // becomes the fallback; whatever it replaces is
                            // dropped, releasing that hold. Depth stays one.
                            let displaced = NeutralCell {
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
                            v.insert(NeutralCell {
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
            return Ok(KvPlacement::Neutral);
        }
        let inserted = {
            let entry = self
                .peers
                .entry(peer)
                .or_default()
                .docs
                .entry(doc)
                .or_default();
            match entry.kv.entry(key) {
                Entry::Occupied(mut o) => {
                    if at < o.get().at {
                        Ok(false)
                    } else if o.get_mut().hold.resize(new_bytes).is_err() {
                        Err(KvError::QuotaExceeded)
                    } else {
                        let cell = o.get_mut();
                        cell.at = at;
                        cell.value = value;
                        Ok(false)
                    }
                }
                Entry::Vacant(v) => quota.hold(Stock::KvMemory, new_bytes).map_or(
                    Err(KvError::QuotaExceeded),
                    |hold| {
                        v.insert(OwnedCell { at, value, hold });
                        Ok(true)
                    },
                ),
            }
        };
        match inserted {
            Ok(true) => self.inc_ref(doc),
            Ok(false) => {}
            Err(err) => {
                self.prune_entry(peer, doc);
                self.prune_presence(doc);
                return Err(err);
            }
        }
        Ok(KvPlacement::Owned)
    }

    /// Restores each neutral cell `peer` last wrote to its prior version, or
    /// drops it when the prior version was also theirs — a peer cannot leave
    /// its own earlier write behind as the fallback.
    fn revert_neutral_writes(&mut self, peer: PeerId) -> usize {
        let mut emptied = Vec::new();
        let mut reverted = 0;

        for (&doc, presence) in &mut self.docs {
            let mut dropped = 0;
            presence.neutral_kv.retain(|_, cell| {
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

    fn remove_kv(&mut self, peer: PeerId, doc: NamespaceId, key: &str, placement: KvPlacement) {
        match placement {
            KvPlacement::Neutral => {
                if let Some(p) = self.docs.get_mut(&doc)
                    && p.neutral_kv.remove(key).is_some()
                {
                    self.dec_ref(doc);
                }
            }
            KvPlacement::Owned => {
                if let Some(entry) = self.peers.get_mut(&peer).and_then(|r| r.docs.get_mut(&doc))
                    && entry.kv.remove(key).is_some()
                {
                    self.dec_ref(doc);
                    self.prune_entry(peer, doc);
                }
            }
        }
    }

    /// Owner of `doc`: the oldest valid pin, breaking ties by peer id. Resolved
    /// per-client; ownership migrates to the next-oldest pinner when an owner
    /// leaves.
    fn resolve_peer(
        &self,
        space: NamespaceId,
        doc: NamespaceId,
        newest: bool,
        field: impl Fn(&PeerDocEntry) -> Option<u64>,
    ) -> Option<PeerId> {
        if self.docs.get(&doc).is_none_or(|p| p.space != space) {
            return None;
        }
        let candidates = self
            .peers
            .iter()
            .filter_map(|(pid, r)| Some((field(r.docs.get(&doc)?)?, *pid)));
        let tiebreak =
            |a: &(u64, PeerId), b: &(u64, PeerId)| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1));
        if newest {
            candidates.max_by(tiebreak)
        } else {
            candidates.min_by(tiebreak)
        }
        .map(|(_, pid)| pid)
    }

    fn owner(&self, space: NamespaceId, doc: NamespaceId) -> Option<PeerId> {
        self.resolve_peer(space, doc, false, |e| e.pin)
    }

    /// Transform authority for `doc`: the latest explicit claimer, or the
    /// document's owner when no one has claimed, so an owner drives its objects
    /// by default until a peer grabs them.
    fn authority(&self, space: NamespaceId, doc: NamespaceId) -> Option<PeerId> {
        self.resolve_peer(space, doc, true, |e| e.authority)
            .or_else(|| self.owner(space, doc))
    }

    fn merged_cell_ref(
        &self,
        space: NamespaceId,
        doc: NamespaceId,
        key: &str,
    ) -> Option<&Option<Vec<u8>>> {
        let presence = self.docs.get(&doc).filter(|p| p.space == space)?;
        let owned = self.peers.iter().filter_map(|(pid, r)| {
            let cell = r.docs.get(&doc)?.kv.get(key)?;
            Some((cell.at, *pid, &cell.value))
        });
        let neutral = presence
            .neutral_kv
            .get(key)
            .map(|c| (c.at, c.peer, &c.value));
        owned
            .chain(neutral)
            .max_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)))
            .map(|(_, _, value)| value)
    }

    fn merged_cell(&self, space: NamespaceId, doc: NamespaceId, key: &str) -> Option<Vec<u8>> {
        self.merged_cell_ref(space, doc, key).and_then(Clone::clone)
    }

    /// Whether `doc` is space-owned: the space's base document, or a doc with
    /// no pin-owner. Space-owned documents accept KV writes from any peer
    /// and store them neutrally; peer-owned documents only from their
    /// owner.
    fn is_space_owned(&self, space: NamespaceId, doc: NamespaceId) -> bool {
        doc == space || self.owner(space, doc).is_none()
    }

    fn key_set(&self, doc: NamespaceId) -> HashSet<String> {
        let mut keys = HashSet::new();
        for r in self.peers.values() {
            if let Some(e) = r.docs.get(&doc) {
                keys.extend(e.kv.keys().cloned());
            }
        }
        if let Some(p) = self.docs.get(&doc) {
            keys.extend(p.neutral_kv.keys().cloned());
        }
        keys
    }

    fn self_snapshot(&self, me: PeerId) -> Vec<DocSnapshot> {
        let mut by_doc: HashMap<NamespaceId, DocSnapshot> = HashMap::new();
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
                        kv: e
                            .kv
                            .iter()
                            .map(|(k, c)| KvSnapshot {
                                key:   k.clone(),
                                value: c.value.clone(),
                                at:    c.at,
                            })
                            .collect(),
                    },
                );
            }
        }
        for (doc, p) in &self.docs {
            for (key, c) in &p.neutral_kv {
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

static PEER_STATE: LazyLock<Mutex<ReplicatedPeerState>> =
    LazyLock::new(|| Mutex::new(ReplicatedPeerState::default()));
static SENDER_TOKEN: AtomicU64 = AtomicU64::new(0);

#[must_use]
pub fn current_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

/// Whether `at` is within the accepted clock skew of local time.
#[must_use]
pub fn time_valid(at: u64) -> bool {
    // TODO lower bound check or use "recieved" time only
    at <= current_millis().saturating_add(MAX_CLOCK_SKEW_MILLIS)
}

/// Runs `reassign` against the document quotas after the state lock is
/// released, since the owner resolver re-enters the store.
fn settle_reassigns(reassign: Vec<(NamespaceId, NamespaceId)>) {
    for (doc, space) in reassign {
        reassign_document_in_space(DocId(*doc.as_bytes()), DocId(*space.as_bytes()));
    }
}

/// Registers a delta stream, returning its cancel token and a receiver whose
/// first message is a full snapshot of the local peer's state.
pub fn register_stream() -> (u64, async_channel::Receiver<StateMsg>) {
    let (tx, rx) = async_channel::unbounded();
    let mut state = PEER_STATE.lock();
    let snapshot = self_peer_id()
        .map(|me| state.self_snapshot(me))
        .unwrap_or_default();
    let _ = tx.try_send(StateMsg::Snapshot(snapshot));
    let token = SENDER_TOKEN.fetch_add(1, Ordering::Relaxed);
    state.senders.insert(token, tx);
    drop(state);
    (token, rx)
}

pub fn unregister_stream(token: u64) {
    PEER_STATE.lock().senders.remove(&token);
}

pub fn broadcast(msg: &StateMsg) {
    PEER_STATE.lock().broadcast(msg);
}

/// Where a KV write landed, so the guard knows which store location to clear on
/// drop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KvPlacement {
    Owned,
    Neutral,
}

/// Adds a peer's pin on `doc`. Returns `false` if the document quota refuses
/// the presence. Idempotent: a repeat pin from the same peer is a no-op.
pub fn add_pin(peer: PeerId, doc: NamespaceId, space: NamespaceId, at: u64) -> bool {
    let quota = document_quota(DocId(*doc.as_bytes()));
    let mut reassign = Vec::new();
    let mut state = PEER_STATE.lock();
    let ok = state.add_pin(peer, doc, space, at, &quota, &mut reassign);
    drop(state);
    settle_reassigns(reassign);
    ok
}

pub fn remove_pin(peer: PeerId, doc: NamespaceId) {
    let mut reassign = Vec::new();
    let mut state = PEER_STATE.lock();
    state.remove_pin(peer, doc, &mut reassign);
    drop(state);
    settle_reassigns(reassign);
}

/// Adds or refreshes a peer's authority claim on `doc`. Returns `false` if the
/// document quota refuses the presence.
pub fn add_authority(peer: PeerId, doc: NamespaceId, space: NamespaceId, at: u64) -> bool {
    let quota = document_quota(DocId(*doc.as_bytes()));
    let mut state = PEER_STATE.lock();
    let ok = state.add_authority(peer, doc, space, at, &quota);
    drop(state);
    ok
}

pub fn remove_authority(peer: PeerId, doc: NamespaceId) {
    let mut state = PEER_STATE.lock();
    state.remove_authority(peer, doc);
    drop(state);
}

/// Applies a KV write for `peer`, placing it neutrally for space-owned
/// documents or under the peer for owner-authored ones.
///
/// Rejects writes to a peer-owned document by a non-owner, and drops writes
/// that exceed quota.
pub fn add_kv(
    peer: PeerId,
    doc: NamespaceId,
    space: NamespaceId,
    key: String,
    value: Option<Vec<u8>>,
    at: u64,
) -> Result<KvPlacement, KvError> {
    let quota = document_quota(DocId(*doc.as_bytes()));
    let mut state = PEER_STATE.lock();
    let result = state.add_kv(peer, doc, space, key, value, at, &quota);
    drop(state);
    result
}

pub fn remove_kv(peer: PeerId, doc: NamespaceId, key: &str, placement: KvPlacement) {
    let mut state = PEER_STATE.lock();
    state.remove_kv(peer, doc, key, placement);
    drop(state);
}

/// Rolls back every neutral cell whose current value came from `peer`.
///
/// Returns how many cells changed. Owner-authored KV, pins and authority claims
/// need no equivalent: they hang off the peer's `RemotePeer` entity and cascade
/// away when it despawns. Neutral cells deliberately outlive a disconnect, so
/// they are the one surface that has to be undone by hand.
pub fn revert_neutral_writes(peer: PeerId) -> usize {
    let mut state = PEER_STATE.lock();
    state.revert_neutral_writes(peer)
}

#[must_use]
pub fn owner(space: NamespaceId, doc: NamespaceId) -> Option<PeerId> {
    PEER_STATE.lock().owner(space, doc)
}

#[must_use]
pub fn is_self_owner(space: NamespaceId, doc: NamespaceId) -> bool {
    self_peer_id().is_some_and(|me| owner(space, doc) == Some(me))
}

#[must_use]
pub fn authority(space: NamespaceId, doc: NamespaceId) -> Option<PeerId> {
    PEER_STATE.lock().authority(space, doc)
}

#[must_use]
pub fn is_self_authority(space: NamespaceId, doc: NamespaceId) -> bool {
    self_peer_id().is_some_and(|me| authority(space, doc) == Some(me))
}

#[must_use]
pub fn has_doc(space: NamespaceId, doc: NamespaceId) -> bool {
    PEER_STATE
        .lock()
        .docs
        .get(&doc)
        .is_some_and(|p| p.space == space)
}

#[must_use]
pub fn doc_kv_get(space: NamespaceId, doc: NamespaceId, key: &str) -> Option<Vec<u8>> {
    PEER_STATE.lock().merged_cell(space, doc, key)
}

#[must_use]
pub fn doc_kv_keys(space: NamespaceId, doc: NamespaceId) -> Vec<String> {
    let state = PEER_STATE.lock();
    if state.docs.get(&doc).is_none_or(|p| p.space != space) {
        return Vec::new();
    }
    let out = state
        .key_set(doc)
        .into_iter()
        .filter(|k| {
            state
                .merged_cell_ref(space, doc, k)
                .is_some_and(Option::is_some)
        })
        .collect();
    drop(state);
    out
}

#[must_use]
pub fn doc_kv_total_bytes(space: NamespaceId, doc: NamespaceId) -> usize {
    let state = PEER_STATE.lock();
    if state.docs.get(&doc).is_none_or(|p| p.space != space) {
        return 0;
    }
    let total = state
        .key_set(doc)
        .into_iter()
        .filter_map(|k| {
            let len = state.merged_cell_ref(space, doc, &k)?.as_ref()?.len();
            Some(k.len() + len)
        })
        .sum();
    drop(state);
    total
}

/// Remote peers that hold `doc`, those this client can sync the record from.
/// Excludes the local peer; the owner is listed first as the freshest source.
#[must_use]
pub fn doc_holders(doc: NamespaceId) -> Vec<PeerId> {
    let me = self_peer_id();
    let state = PEER_STATE.lock();
    let Some(space) = state.docs.get(&doc).map(|p| p.space) else {
        return Vec::new();
    };
    let owner = state.owner(space, doc);
    let mut holders = state
        .peers
        .iter()
        .filter(|(pid, r)| {
            Some(**pid) != me
                && Some(**pid) != owner
                && r.docs.get(&doc).is_some_and(|e| e.pin.is_some())
        })
        .map(|(pid, _)| *pid)
        .collect::<Vec<_>>();
    if let Some(owner) = owner.filter(|o| Some(*o) != me) {
        holders.insert(0, owner);
    }
    drop(state);
    holders
}

/// The space `doc` is pinned in, per any peer's replica. Lets membership
/// resolve a synced doc's space without a local ownership claim.
#[must_use]
pub fn space_of(doc: NamespaceId) -> Option<NamespaceId> {
    PEER_STATE.lock().docs.get(&doc).map(|p| p.space)
}

/// Whether any peer currently pins `doc`. Drives the scene's fetch/despawn of
/// tracked documents.
#[must_use]
pub fn is_pinned(doc: NamespaceId) -> bool {
    PEER_STATE
        .lock()
        .peers
        .values()
        .any(|r| r.docs.get(&doc).is_some_and(|e| e.pin.is_some()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum KvError {
    #[error("kv key exceeds maximum length")]
    KeyTooLong,
    #[error("kv write to a peer-owned document by a non-owner")]
    NotOwner,
    #[error("kv write exceeds quota")]
    QuotaExceeded,
    #[error("kv write failed")]
    Other,
}

/// Read-only views of the peer store for the dev tools state inspector.
#[cfg(feature = "devtools")]
pub mod debug {
    use super::{
        HashMap,
        NamespaceId,
        OwnedCell,
        PEER_STATE,
        PeerId,
    };

    pub struct DebugKv {
        pub key:    String,
        /// The cell's value bytes; `None` is a tombstone.
        pub value:  Option<Vec<u8>>,
        pub at:     u64,
        /// The authoring peer for neutral cells; owned cells' writer is the
        /// peer they sit under.
        pub writer: Option<PeerId>,
    }

    pub struct DebugDoc {
        pub doc:       NamespaceId,
        pub space:     NamespaceId,
        pub pin:       Option<u64>,
        pub authority: Option<u64>,
        pub kv:        Vec<DebugKv>,
    }

    pub struct DebugPeer {
        pub peer: PeerId,
        pub docs: Vec<DebugDoc>,
    }

    pub struct DebugSnapshot {
        pub peers:   Vec<DebugPeer>,
        /// Space-owned (neutral) KV, held by documents rather than any peer.
        pub neutral: Vec<DebugDoc>,
    }

    fn debug_kv(kv: &HashMap<String, OwnedCell>) -> Vec<DebugKv> {
        let mut out = kv
            .iter()
            .map(|(k, c)| DebugKv {
                key:    k.clone(),
                value:  c.value.clone(),
                at:     c.at,
                writer: None,
            })
            .collect::<Vec<_>>();
        out.sort_unstable_by(|a, b| a.key.cmp(&b.key));
        out
    }

    /// A deterministically ordered snapshot, so the panel can fingerprint it
    /// and rebuild only on change.
    #[must_use]
    pub fn snapshot() -> DebugSnapshot {
        let state = PEER_STATE.lock();
        let mut peers = state
            .peers
            .iter()
            .map(|(pid, r)| {
                let mut docs = r
                    .docs
                    .iter()
                    .map(|(doc, e)| DebugDoc {
                        doc:       *doc,
                        space:     state.docs.get(doc).map_or(*doc, |p| p.space),
                        pin:       e.pin,
                        authority: e.authority,
                        kv:        debug_kv(&e.kv),
                    })
                    .collect::<Vec<_>>();
                docs.sort_unstable_by_key(|d| *d.doc.as_bytes());
                DebugPeer { peer: *pid, docs }
            })
            .collect::<Vec<_>>();
        peers.sort_unstable_by_key(|p| p.peer);
        let mut neutral = state
            .docs
            .iter()
            .filter(|(_, p)| !p.neutral_kv.is_empty())
            .map(|(doc, p)| {
                let mut kv = p
                    .neutral_kv
                    .iter()
                    .map(|(k, c)| DebugKv {
                        key:    k.clone(),
                        value:  c.value.clone(),
                        at:     c.at,
                        writer: Some(c.peer),
                    })
                    .collect::<Vec<_>>();
                kv.sort_unstable_by(|a, b| a.key.cmp(&b.key));
                DebugDoc {
                    doc: *doc,
                    space: p.space,
                    pin: None,
                    authority: None,
                    kv,
                }
            })
            .collect::<Vec<_>>();
        neutral.sort_unstable_by_key(|d| *d.doc.as_bytes());
        drop(state);
        DebugSnapshot { peers, neutral }
    }
}

#[cfg(test)]
pub fn reset() {
    *PEER_STATE.lock() = ReplicatedPeerState::default();
}

/// Serializes tests, which share the global state and self-peer identity.
#[cfg(test)]
pub static TEST_LOCK: Mutex<()> = Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;

    fn h(seed: &[u8]) -> NamespaceId {
        NamespaceId::from(blake3::hash(seed).as_bytes())
    }

    #[test]
    fn pin_owner_is_oldest_and_releases() {
        let _g = TEST_LOCK.lock();
        reset();
        let space = h(b"oldest-space");
        let doc = h(b"oldest-doc");
        let early = [1u8; 32];
        let late = [2u8; 32];

        assert!(add_pin(late, doc, space, 20));
        assert_eq!(owner(space, doc), Some(late));
        assert!(add_pin(early, doc, space, 10));
        assert_eq!(owner(space, doc), Some(early));

        remove_pin(early, doc);
        assert_eq!(owner(space, doc), Some(late));
        remove_pin(late, doc);
        assert_eq!(owner(space, doc), None);
        assert!(!has_doc(space, doc));
        reset();
    }

    #[test]
    fn authority_latest_and_defaults_to_owner() {
        let _g = TEST_LOCK.lock();
        reset();
        let space = h(b"auth-space");
        let doc = h(b"auth-doc");
        let owner_peer = [1u8; 32];
        let grabber = [2u8; 32];

        assert!(add_pin(owner_peer, doc, space, 10));
        assert!(add_pin(grabber, doc, space, 20));
        assert_eq!(authority(space, doc), Some(owner_peer));

        assert!(add_authority(grabber, doc, space, 200));
        assert_eq!(authority(space, doc), Some(grabber));
        assert_eq!(owner(space, doc), Some(owner_peer));

        remove_authority(grabber, doc);
        assert_eq!(authority(space, doc), Some(owner_peer));
        reset();
    }

    #[test]
    fn neutral_kv_persists_across_owned_kv() {
        let _g = TEST_LOCK.lock();
        reset();
        let space = h(b"kv-space");
        let alice = [2u8; 32];

        assert_eq!(
            add_kv(
                alice,
                space,
                space,
                "link".into(),
                Some(b"dest".to_vec()),
                1
            ),
            Ok(KvPlacement::Neutral)
        );
        assert_eq!(
            doc_kv_get(space, space, "link").as_deref(),
            Some(&b"dest"[..])
        );

        remove_kv(alice, space, "missing", KvPlacement::Owned);
        assert_eq!(
            doc_kv_get(space, space, "link").as_deref(),
            Some(&b"dest"[..])
        );

        remove_kv(alice, space, "link", KvPlacement::Neutral);
        assert_eq!(doc_kv_get(space, space, "link"), None);
        assert!(!has_doc(space, space));
        reset();
    }

    #[test]
    fn reverting_a_peer_restores_the_value_it_overwrote() {
        let _g = TEST_LOCK.lock();
        reset();
        let space = h(b"revert-space");
        let (alice, mallory) = ([2u8; 32], [3u8; 32]);

        add_kv(
            alice,
            space,
            space,
            "sign".into(),
            Some(b"welcome".to_vec()),
            1,
        )
        .expect("alice writes the sign");
        add_kv(
            mallory,
            space,
            space,
            "sign".into(),
            Some(b"defaced".to_vec()),
            2,
        )
        .expect("mallory defaces it");
        assert_eq!(
            doc_kv_get(space, space, "sign").as_deref(),
            Some(&b"defaced"[..])
        );

        assert_eq!(revert_neutral_writes(mallory), 1);
        assert_eq!(
            doc_kv_get(space, space, "sign").as_deref(),
            Some(&b"welcome"[..]),
            "blocking must put back what the blocked peer wrote over"
        );
        reset();
    }

    #[test]
    fn reverting_drops_a_cell_the_peer_created() {
        let _g = TEST_LOCK.lock();
        reset();
        let space = h(b"revert-new-space");
        let mallory = [3u8; 32];

        add_kv(mallory, space, space, "spam".into(), Some(b"x".to_vec()), 1)
            .expect("mallory writes a new cell");

        assert_eq!(revert_neutral_writes(mallory), 1);
        assert_eq!(
            doc_kv_get(space, space, "spam"),
            None,
            "a cell with no prior version has nothing to fall back to"
        );
        assert!(
            !has_doc(space, space),
            "dropping the last cell must release the document presence"
        );
        reset();
    }

    #[test]
    fn a_peer_cannot_leave_its_own_earlier_write_as_the_fallback() {
        let _g = TEST_LOCK.lock();
        reset();
        let space = h(b"revert-own-space");
        let mallory = [3u8; 32];

        add_kv(
            mallory,
            space,
            space,
            "sign".into(),
            Some(b"first".to_vec()),
            1,
        )
        .expect("first");
        add_kv(
            mallory,
            space,
            space,
            "sign".into(),
            Some(b"second".to_vec()),
            2,
        )
        .expect("second");

        revert_neutral_writes(mallory);
        assert_eq!(
            doc_kv_get(space, space, "sign"),
            None,
            "falling back to the blocked peer's own earlier write undoes nothing"
        );
        reset();
    }

    #[test]
    fn reverting_leaves_another_peers_cells_alone() {
        let _g = TEST_LOCK.lock();
        reset();
        let space = h(b"revert-other-space");
        let (alice, mallory) = ([2u8; 32], [3u8; 32]);

        add_kv(
            alice,
            space,
            space,
            "keep".into(),
            Some(b"mine".to_vec()),
            1,
        )
        .expect("alice");

        assert_eq!(revert_neutral_writes(mallory), 0);
        assert_eq!(
            doc_kv_get(space, space, "keep").as_deref(),
            Some(&b"mine"[..])
        );
        reset();
    }

    #[test]
    fn owned_kv_gated_by_ownership() {
        let _g = TEST_LOCK.lock();
        reset();
        let space = h(b"perm-space");
        let owner_peer = [1u8; 32];
        let other = [2u8; 32];
        let doc = h(b"perm-doc");

        assert!(add_pin(owner_peer, doc, space, 1));
        assert_eq!(
            add_kv(owner_peer, doc, space, "k".into(), Some(b"v".to_vec()), 2),
            Ok(KvPlacement::Owned)
        );
        assert!(matches!(
            add_kv(other, doc, space, "k".into(), Some(b"v".to_vec()), 3),
            Err(KvError::NotOwner)
        ));
        assert_eq!(doc_kv_get(space, doc, "k").as_deref(), Some(&b"v"[..]));
        reset();
    }

    #[test]
    fn refs_release_presence_and_quota() {
        let _g = TEST_LOCK.lock();
        reset();
        let space = h(b"refund-space");
        let doc = h(b"refund-doc");
        let peer = [11u8; 32];

        assert!(add_pin(peer, doc, space, 1));
        assert_eq!(
            add_kv(peer, doc, space, "k".into(), Some(b"value".to_vec()), 2),
            Ok(KvPlacement::Owned)
        );
        let quota = document_quota(DocId(*doc.as_bytes()));
        assert!(quota.usage(Stock::KvMemory) > 0);
        assert_eq!(quota.usage(Stock::Documents), 1);

        remove_kv(peer, doc, "k", KvPlacement::Owned);
        remove_pin(peer, doc);
        assert_eq!(quota.usage(Stock::KvMemory), 0);
        assert_eq!(quota.usage(Stock::Documents), 0);
        assert!(!has_doc(space, doc));
        reset();
    }
}
