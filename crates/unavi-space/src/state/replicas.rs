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
};

use hsd::id::DocId;
use iroh::EndpointId;
use iroh_docs::NamespaceId;
use parking_lot::Mutex;
use unavi_policy::{
    quota::{
        Quota,
        Stock,
        StockHold,
    },
    registry::Policy,
};
use web_time::{
    SystemTime,
    UNIX_EPOCH,
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

pub const KV_KEY_MAX_BYTES: usize = 256;

/// Caps peer-supplied timestamps to within the clock skew of local time, so a
/// forged future `at` cannot pin ownership/authority or win KV merges forever.
const MAX_CLOCK_SKEW_MILLIS: u64 = 5 * 60 * 1000;

/// A KV cell, merged last-write-wins by `(at, peer)`. `value: None` is a
/// retained tombstone, so a delete keeps winning over an older live write.
///
/// Cells live on the document, never under the peer that wrote one, so a cell
/// survives exactly as long as the document does. Ownership of a document
/// migrates to the next-oldest pinner when its owner leaves; keeping the cells
/// under that peer's replica would drop the state while the content it belongs
/// to carried on.
struct Cell {
    at:    u64,
    peer:  EndpointId,
    value: Option<Vec<u8>>,
    hold:  StockHold,
    /// Exactly one prior version, making "revert everything peer X wrote" a
    /// scan of the cell map rather than a general undo log.
    prev:  Option<Box<Self>>,
}

fn cell_bytes(key: &str, value: Option<&[u8]>) -> u64 {
    (key.len() + value.map_or(0, <[u8]>::len)) as u64
}

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
    docs: HashMap<NamespaceId, PeerDocEntry>,
}

/// Per-document state shared across peers. `refs` counts the live pins,
/// authority claims and KV cells keeping the presence alive; `_doc_hold`
/// charges one `Documents` unit while the doc is known locally.
struct DocPresence {
    space:     NamespaceId,
    _doc_hold: StockHold,
    kv:        HashMap<String, Cell>,
    refs:      u32,
}

/// Every peer's replicated state (self included), per-document presence, and
/// the live delta senders. Mutated only by the RAII guards in
/// [`crate::state::entities`].
#[derive(Default)]
struct ReplicatedPeerState {
    peers:   HashMap<EndpointId, PeerReplica>,
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
                kv: HashMap::new(),
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
    fn prune_entry(&mut self, peer: EndpointId, doc: NamespaceId) {
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
        peer: EndpointId,
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
        peer: EndpointId,
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

    fn remove_authority(&mut self, peer: EndpointId, doc: NamespaceId) {
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
        doc: NamespaceId,
        space: NamespaceId,
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

    fn remove_kv(&mut self, doc: NamespaceId, key: &str) {
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
        space: NamespaceId,
        doc: NamespaceId,
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

    fn owner(&self, space: NamespaceId, doc: NamespaceId) -> Option<EndpointId> {
        self.resolve_peer(space, doc, false, |e| e.pin)
    }

    /// Transform authority for `doc`: the latest explicit claimer, or the
    /// document's owner when no one has claimed, so an owner drives its objects
    /// by default until a peer grabs them.
    fn authority(&self, space: NamespaceId, doc: NamespaceId) -> Option<EndpointId> {
        self.resolve_peer(space, doc, true, |e| e.authority)
            .or_else(|| self.owner(space, doc))
    }

    /// The value at `key`, or `None` for a tombstone or a key nothing wrote.
    ///
    /// One cell per key, so the last-write-wins merge already happened at write
    /// time and there is nothing to resolve here.
    fn cell(&self, space: NamespaceId, doc: NamespaceId, key: &str) -> Option<Vec<u8>> {
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
    fn is_space_owned(&self, space: NamespaceId, doc: NamespaceId) -> bool {
        doc == space || self.owner(space, doc).is_none()
    }

    fn key_set(&self, doc: NamespaceId) -> HashSet<String> {
        self.docs
            .get(&doc)
            .map(|p| p.kv.keys().cloned().collect())
            .unwrap_or_default()
    }

    fn self_snapshot(&self, me: EndpointId) -> Vec<DocSnapshot> {
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
fn settle_reassigns(policy: &Policy, reassign: Vec<(NamespaceId, NamespaceId)>) {
    for (doc, space) in reassign {
        reassign_document_in_space(policy, DocId(*doc.as_bytes()), DocId(*space.as_bytes()));
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

/// Adds a peer's pin on `doc`. Returns `false` if the document quota refuses
/// the presence. Idempotent: a repeat pin from the same peer is a no-op.
pub fn add_pin(
    policy: &Policy,
    peer: EndpointId,
    doc: NamespaceId,
    space: NamespaceId,
    at: u64,
) -> bool {
    let quota = document_quota(policy, DocId(*doc.as_bytes()));
    let mut reassign = Vec::new();
    let mut state = PEER_STATE.lock();
    let ok = state.add_pin(peer, doc, space, at, &quota, &mut reassign);
    drop(state);
    settle_reassigns(policy, reassign);
    ok
}

pub fn remove_pin(policy: &Policy, peer: EndpointId, doc: NamespaceId) {
    let mut reassign = Vec::new();
    let mut state = PEER_STATE.lock();
    state.remove_pin(peer, doc, &mut reassign);
    drop(state);
    settle_reassigns(policy, reassign);
}

/// Adds or refreshes a peer's authority claim on `doc`. Returns `false` if the
/// document quota refuses the presence.
pub fn add_authority(
    policy: &Policy,
    peer: EndpointId,
    doc: NamespaceId,
    space: NamespaceId,
    at: u64,
) -> bool {
    let quota = document_quota(policy, DocId(*doc.as_bytes()));
    let mut state = PEER_STATE.lock();
    let ok = state.add_authority(peer, doc, space, at, &quota);
    drop(state);
    ok
}

pub fn remove_authority(peer: EndpointId, doc: NamespaceId) {
    let mut state = PEER_STATE.lock();
    state.remove_authority(peer, doc);
    drop(state);
}

/// Applies a KV write for `peer`.
///
/// Rejects writes to a peer-owned document by a non-owner, and drops writes
/// that exceed quota.
pub fn add_kv(
    policy: &Policy,
    peer: EndpointId,
    doc: NamespaceId,
    space: NamespaceId,
    key: String,
    value: Option<Vec<u8>>,
    at: u64,
) -> Result<(), KvError> {
    let quota = document_quota(policy, DocId(*doc.as_bytes()));
    let mut state = PEER_STATE.lock();
    let result = state.add_kv(peer, doc, space, key, value, at, &quota);
    drop(state);
    result
}

pub fn remove_kv(doc: NamespaceId, key: &str) {
    let mut state = PEER_STATE.lock();
    state.remove_kv(doc, key);
    drop(state);
}

/// Rolls back every cell whose current value came from `peer`, returning how
/// many changed.
///
/// The undo that pins and authority claims get from the peer's entity cascade.
/// Cells live on the document rather than the peer, so they need this instead.
pub fn revert_writes(peer: EndpointId) -> usize {
    let mut state = PEER_STATE.lock();
    state.revert_writes(peer)
}

#[must_use]
pub fn owner(space: NamespaceId, doc: NamespaceId) -> Option<EndpointId> {
    PEER_STATE.lock().owner(space, doc)
}

#[must_use]
pub fn is_self_owner(space: NamespaceId, doc: NamespaceId) -> bool {
    self_peer_id().is_some_and(|me| owner(space, doc) == Some(me))
}

#[must_use]
pub fn authority(space: NamespaceId, doc: NamespaceId) -> Option<EndpointId> {
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
    PEER_STATE.lock().cell(space, doc, key)
}

/// Every key holding a live value. A tombstone is stored but reads as absent,
/// so it is not listed.
#[must_use]
pub fn doc_kv_keys(space: NamespaceId, doc: NamespaceId) -> Vec<String> {
    let state = PEER_STATE.lock();
    if state.docs.get(&doc).is_none_or(|p| p.space != space) {
        return Vec::new();
    }
    let out = state
        .key_set(doc)
        .into_iter()
        .filter(|k| state.cell(space, doc, k).is_some())
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
            let len = state.cell(space, doc, &k)?.len();
            Some(k.len() + len)
        })
        .sum();
    drop(state);
    total
}

/// Remote peers that hold `doc`, those this client can sync the record from.
/// Excludes the local peer; the owner is listed first as the freshest source.
#[must_use]
pub fn doc_holders(doc: NamespaceId) -> Vec<EndpointId> {
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
        EndpointId,
        NamespaceId,
        PEER_STATE,
    };

    pub struct DebugKv {
        pub key:    String,
        /// The cell's value bytes; `None` is a tombstone.
        pub value:  Option<Vec<u8>>,
        pub at:     u64,
        pub writer: EndpointId,
    }

    /// What one peer contributes to a document.
    pub struct DebugPeerDoc {
        pub doc:       NamespaceId,
        pub space:     NamespaceId,
        pub pin:       Option<u64>,
        pub authority: Option<u64>,
    }

    /// What a document holds regardless of which peer wrote it.
    pub struct DebugDoc {
        pub doc:   NamespaceId,
        pub space: NamespaceId,
        pub kv:    Vec<DebugKv>,
    }

    pub struct DebugPeer {
        pub peer: EndpointId,
        pub docs: Vec<DebugPeerDoc>,
    }

    pub struct DebugSnapshot {
        /// Each peer's pins and authority claims.
        pub peers: Vec<DebugPeer>,
        /// KV, which is held by documents rather than by any peer.
        pub docs:  Vec<DebugDoc>,
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
                    .map(|(doc, e)| DebugPeerDoc {
                        doc:       *doc,
                        space:     state.docs.get(doc).map_or(*doc, |p| p.space),
                        pin:       e.pin,
                        authority: e.authority,
                    })
                    .collect::<Vec<_>>();
                docs.sort_unstable_by_key(|d| *d.doc.as_bytes());
                DebugPeer { peer: *pid, docs }
            })
            .collect::<Vec<_>>();
        peers.sort_unstable_by_key(|p| p.peer);

        let mut docs = state
            .docs
            .iter()
            .filter(|(_, p)| !p.kv.is_empty())
            .map(|(doc, p)| {
                let mut kv =
                    p.kv.iter()
                        .map(|(k, c)| DebugKv {
                            key:    k.clone(),
                            value:  c.value.clone(),
                            at:     c.at,
                            writer: c.peer,
                        })
                        .collect::<Vec<_>>();
                kv.sort_unstable_by(|a, b| a.key.cmp(&b.key));
                DebugDoc {
                    doc: *doc,
                    space: p.space,
                    kv,
                }
            })
            .collect::<Vec<_>>();
        docs.sort_unstable_by_key(|d| *d.doc.as_bytes());
        drop(state);
        DebugSnapshot { peers, docs }
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

    /// A distinct, valid endpoint id per seed. Arbitrary bytes are not a curve
    /// point, so a key has to be derived rather than written down.
    fn peer(seed: u8) -> EndpointId {
        iroh::SecretKey::from_bytes(&[seed; 32]).public()
    }

    #[test]
    fn pin_owner_is_oldest_and_releases() {
        let _g = TEST_LOCK.lock();
        reset();
        let policy = Policy::new();
        let space = h(b"oldest-space");
        let doc = h(b"oldest-doc");
        let early = peer(1);
        let late = peer(2);

        assert!(add_pin(&policy, late, doc, space, 20));
        assert_eq!(owner(space, doc), Some(late));
        assert!(add_pin(&policy, early, doc, space, 10));
        assert_eq!(owner(space, doc), Some(early));

        remove_pin(&policy, early, doc);
        assert_eq!(owner(space, doc), Some(late));
        remove_pin(&policy, late, doc);
        assert_eq!(owner(space, doc), None);
        assert!(!has_doc(space, doc));
        reset();
    }

    #[test]
    fn authority_latest_and_defaults_to_owner() {
        let _g = TEST_LOCK.lock();
        reset();
        let policy = Policy::new();
        let space = h(b"auth-space");
        let doc = h(b"auth-doc");
        let owner_peer = peer(1);
        let grabber = peer(2);

        assert!(add_pin(&policy, owner_peer, doc, space, 10));
        assert!(add_pin(&policy, grabber, doc, space, 20));
        assert_eq!(authority(space, doc), Some(owner_peer));

        assert!(add_authority(&policy, grabber, doc, space, 200));
        assert_eq!(authority(space, doc), Some(grabber));
        assert_eq!(owner(space, doc), Some(owner_peer));

        remove_authority(grabber, doc);
        assert_eq!(authority(space, doc), Some(owner_peer));
        reset();
    }

    /// The document outlives its first owner through the next-oldest pin, so
    /// its state has to outlive them too. Cells kept under the owner's replica
    /// went with them, leaving the content behind with an empty KV.
    #[test]
    fn kv_survives_the_owner_leaving() {
        let _g = TEST_LOCK.lock();
        reset();
        let policy = Policy::new();
        let space = h(b"handoff-space");
        let doc = h(b"handoff-doc");
        let (first, second) = (peer(1), peer(2));

        assert!(add_pin(&policy, first, doc, space, 10));
        assert!(add_pin(&policy, second, doc, space, 20));
        assert_eq!(owner(space, doc), Some(first));

        add_kv(
            &policy,
            first,
            doc,
            space,
            "colour".into(),
            Some(b"red".to_vec()),
            1,
        )
        .expect("the owner may write");

        remove_pin(&policy, first, doc);

        assert_eq!(owner(space, doc), Some(second), "ownership hands off");
        assert_eq!(
            doc_kv_get(space, doc, "colour").as_deref(),
            Some(&b"red"[..]),
            "the new owner inherits the state, not an empty document"
        );

        add_kv(
            &policy,
            second,
            doc,
            space,
            "colour".into(),
            Some(b"blue".to_vec()),
            2,
        )
        .expect("the new owner may write what it inherited");
        assert_eq!(
            doc_kv_get(space, doc, "colour").as_deref(),
            Some(&b"blue"[..])
        );
        reset();
    }

    #[test]
    fn neutral_kv_persists_across_owned_kv() {
        let _g = TEST_LOCK.lock();
        reset();
        let policy = Policy::new();
        let space = h(b"kv-space");
        let alice = peer(2);

        assert_eq!(
            add_kv(
                &policy,
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
            doc_kv_get(space, space, "link").as_deref(),
            Some(&b"dest"[..])
        );

        remove_kv(space, "missing");
        assert_eq!(
            doc_kv_get(space, space, "link").as_deref(),
            Some(&b"dest"[..])
        );

        remove_kv(space, "link");
        assert_eq!(doc_kv_get(space, space, "link"), None);
        assert!(!has_doc(space, space));
        reset();
    }

    #[test]
    fn reverting_a_peer_restores_the_value_it_overwrote() {
        let _g = TEST_LOCK.lock();
        reset();
        let policy = Policy::new();
        let space = h(b"revert-space");
        let (alice, mallory) = (peer(2), peer(3));

        add_kv(
            &policy,
            alice,
            space,
            space,
            "sign".into(),
            Some(b"welcome".to_vec()),
            1,
        )
        .expect("alice writes the sign");
        add_kv(
            &policy,
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

        assert_eq!(revert_writes(mallory), 1);
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
        let policy = Policy::new();
        let space = h(b"revert-new-space");
        let mallory = peer(3);

        add_kv(
            &policy,
            mallory,
            space,
            space,
            "spam".into(),
            Some(b"x".to_vec()),
            1,
        )
        .expect("mallory writes a new cell");

        assert_eq!(revert_writes(mallory), 1);
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
        let policy = Policy::new();
        let space = h(b"revert-own-space");
        let mallory = peer(3);

        add_kv(
            &policy,
            mallory,
            space,
            space,
            "sign".into(),
            Some(b"first".to_vec()),
            1,
        )
        .expect("first");
        add_kv(
            &policy,
            mallory,
            space,
            space,
            "sign".into(),
            Some(b"second".to_vec()),
            2,
        )
        .expect("second");

        revert_writes(mallory);
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
        let policy = Policy::new();
        let space = h(b"revert-other-space");
        let (alice, mallory) = (peer(2), peer(3));

        add_kv(
            &policy,
            alice,
            space,
            space,
            "keep".into(),
            Some(b"mine".to_vec()),
            1,
        )
        .expect("alice");

        assert_eq!(revert_writes(mallory), 0);
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
        let policy = Policy::new();
        let space = h(b"perm-space");
        let owner_peer = peer(1);
        let other = peer(2);
        let doc = h(b"perm-doc");

        assert!(add_pin(&policy, owner_peer, doc, space, 1));
        assert_eq!(
            add_kv(
                &policy,
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
            add_kv(
                &policy,
                other,
                doc,
                space,
                "k".into(),
                Some(b"v".to_vec()),
                3
            ),
            Err(KvError::NotOwner)
        ));
        assert_eq!(doc_kv_get(space, doc, "k").as_deref(), Some(&b"v"[..]));
        reset();
    }

    #[test]
    fn refs_release_presence_and_quota() {
        let _g = TEST_LOCK.lock();
        reset();
        let policy = Policy::new();
        let space = h(b"refund-space");
        let doc = h(b"refund-doc");
        let peer = peer(11);

        assert!(add_pin(&policy, peer, doc, space, 1));
        assert_eq!(
            add_kv(
                &policy,
                peer,
                doc,
                space,
                "k".into(),
                Some(b"value".to_vec()),
                2
            ),
            Ok(())
        );
        let quota = document_quota(&policy, DocId(*doc.as_bytes()));
        assert!(quota.usage(Stock::KvMemory) > 0);
        assert_eq!(quota.usage(Stock::Documents), 1);

        remove_kv(doc, "k");
        remove_pin(&policy, peer, doc);
        assert_eq!(quota.usage(Stock::KvMemory), 0);
        assert_eq!(quota.usage(Stock::Documents), 0);
        assert!(!has_doc(space, doc));
        reset();
    }
}
