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

use blake3::Hash;
use parking_lot::Mutex;
use tracing::warn;
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

/// Rejects peer-supplied timestamps more than this far past local time, so a
/// forged future `at` cannot pin ownership/authority or win KV merges forever.
const MAX_CLOCK_SKEW_MILLIS: u64 = 5 * 60 * 1000;

/// A change to whether any peer pins a document, emitted on the 0↔1 holder
/// transition so the scene can react without scanning every pin each frame.
#[derive(Debug, Clone)]
pub enum PinChange {
    Pinned { doc: Hash, space: Hash },
    Unpinned { doc: Hash },
}

/// A KV cell merged last-write-wins. `value: None` is a tombstone retained so a
/// delete keeps winning over an older live write on another peer. `_hold`
/// charges the cell's bytes to the document's quota for exactly its lifetime,
/// so dropping the cell refunds with no separate bookkeeping.
struct LwwCell {
    at:    u64,
    value: Option<Vec<u8>>,
    hold:  StockHold,
}

impl LwwCell {
    fn bytes(key: &str, value: Option<&[u8]>) -> u64 {
        (key.len() + value.map_or(0, <[u8]>::len)) as u64
    }
}

/// One peer's contribution to a document: its pin (timestamped, so the oldest
/// pin owns the doc), its latest object-authority claim, and the KV it
/// authored. KV is owner-authoritative for peer-owned docs and open for
/// space-owned docs, so per-peer cells exist mainly to merge writes to open
/// documents.
#[derive(Default)]
struct PeerDocEntry {
    pin:       Option<u64>,
    authority: Option<u64>,
    kv:        HashMap<String, LwwCell>,
}

impl PeerDocEntry {
    fn is_empty(&self) -> bool {
        self.pin.is_none() && self.authority.is_none() && self.kv.is_empty()
    }
}

#[derive(Default)]
struct PeerReplica {
    docs: HashMap<Hash, PeerDocEntry>,
}

/// Per-document state shared across peers: its space, its quota (charged to the
/// owner, migrated on handoff), a `Documents` hold held while the doc is known
/// locally, and the holder/reference counts that drive scene lifecycle.
struct DocPresence {
    space:       Hash,
    quota:       Arc<Quota>,
    _doc_hold:   StockHold,
    pin_holders: u32,
    entry_refs:  u32,
}

/// Holds every peer's replicated state (self included), per-document presence,
/// and the live delta/pin senders.
#[derive(Default)]
struct ReplicatedPeerState {
    peers:   HashMap<PeerId, PeerReplica>,
    docs:    HashMap<Hash, DocPresence>,
    senders: HashMap<u64, async_channel::Sender<StateMsg>>,
    pin_tx:  Option<async_channel::Sender<PinChange>>,
}

impl ReplicatedPeerState {
    fn broadcast(&mut self, msg: &StateMsg) {
        self.senders
            .retain(|_, tx| tx.try_send(msg.clone()).is_ok());
    }

    fn emit_pin(&self, change: PinChange) {
        if let Some(tx) = &self.pin_tx {
            let _ = tx.try_send(change);
        }
    }

    /// Ensures a per-peer entry for `doc`, creating its [`DocPresence`] (and
    /// charging one `Documents` unit) on first sight. Returns `false` when that
    /// charge is refused, so the caller drops the update rather than store it
    /// untracked.
    fn ensure_entry(&mut self, peer: PeerId, doc: Hash, space: Hash, quota: &Arc<Quota>) -> bool {
        if let Entry::Vacant(v) = self.docs.entry(doc) {
            let Ok(hold) = quota.hold(Stock::Documents, 1) else {
                return false;
            };
            v.insert(DocPresence {
                space,
                quota: Arc::clone(quota),
                _doc_hold: hold,
                pin_holders: 0,
                entry_refs: 0,
            });
        }
        let replica = self.peers.entry(peer).or_default();
        if let Entry::Vacant(v) = replica.docs.entry(doc) {
            v.insert(PeerDocEntry::default());
            if let Some(p) = self.docs.get_mut(&doc) {
                p.entry_refs += 1;
            }
        }
        true
    }

    /// Drops a peer's now-empty entry, releasing its document presence (and the
    /// `Documents` hold) once no peer references the doc.
    fn prune_entry(&mut self, peer: PeerId, doc: Hash, reassign: &mut Vec<(Hash, Hash)>) {
        let Some(replica) = self.peers.get_mut(&peer) else {
            return;
        };
        let Entry::Occupied(e) = replica.docs.entry(doc) else {
            return;
        };
        if !e.get().is_empty() {
            return;
        }
        e.remove();
        if let Entry::Occupied(mut p) = self.docs.entry(doc) {
            p.get_mut().entry_refs -= 1;
            if p.get().entry_refs == 0 {
                p.remove();
            } else {
                reassign.push((doc, p.get().space));
            }
        }
    }

    fn set_pin(&mut self, peer: PeerId, doc: Hash, at: u64, reassign: &mut Vec<(Hash, Hash)>) {
        let Some(entry) = self.peers.get_mut(&peer).and_then(|r| r.docs.get_mut(&doc)) else {
            return;
        };
        if entry.pin.is_some() {
            return;
        }
        entry.pin = Some(at);
        let Some(p) = self.docs.get_mut(&doc) else {
            return;
        };
        p.pin_holders += 1;
        let space = p.space;
        let first = p.pin_holders == 1;
        if first {
            self.emit_pin(PinChange::Pinned { doc, space });
        }
        reassign.push((doc, space));
    }

    fn clear_pin(&mut self, peer: PeerId, doc: Hash, reassign: &mut Vec<(Hash, Hash)>) {
        let Some(entry) = self.peers.get_mut(&peer).and_then(|r| r.docs.get_mut(&doc)) else {
            return;
        };
        if entry.pin.take().is_none() {
            return;
        }
        if let Some(p) = self.docs.get_mut(&doc) {
            p.pin_holders = p.pin_holders.saturating_sub(1);
            let space = p.space;
            let last = p.pin_holders == 0;
            if last {
                self.emit_pin(PinChange::Unpinned { doc });
            }
            reassign.push((doc, space));
        }
    }

    fn set_authority(&mut self, peer: PeerId, doc: Hash, at: u64) {
        if let Some(entry) = self.peers.get_mut(&peer).and_then(|r| r.docs.get_mut(&doc)) {
            entry.authority = Some(at);
        }
    }

    /// Applies a KV write to a peer's entry, sizing the cell's quota hold to
    /// the net byte delta. A smaller value or tombstone always fits, even
    /// at a full cap; a growth that exceeds the cap leaves the old value
    /// and returns `false`. A stale (older `at`) write is a no-op and
    /// returns `true`.
    fn write_kv(
        &mut self,
        peer: PeerId,
        doc: Hash,
        key: String,
        value: Option<Vec<u8>>,
        at: u64,
    ) -> bool {
        let Some(p) = self.docs.get(&doc) else {
            return false;
        };
        let quota = Arc::clone(&p.quota);
        let Some(entry) = self.peers.get_mut(&peer).and_then(|r| r.docs.get_mut(&doc)) else {
            return false;
        };
        let new_bytes = LwwCell::bytes(&key, value.as_deref());
        match entry.kv.entry(key) {
            Entry::Occupied(mut o) => {
                if at < o.get().at {
                    return true;
                }
                let cell = o.get_mut();
                if cell.hold.resize(new_bytes).is_err() {
                    warn!(?peer, "kv update dropped over quota");
                    return false;
                }
                cell.at = at;
                cell.value = value;
                true
            }
            Entry::Vacant(v) => {
                let Ok(hold) = quota.hold(Stock::KvMemory, new_bytes) else {
                    warn!(?peer, "kv write dropped over quota");
                    return false;
                };
                v.insert(LwwCell { at, value, hold });
                true
            }
        }
    }

    /// Removes a peer's entire replica, dropping its holds and decrementing the
    /// per-document holder/reference counts.
    fn clear_peer(&mut self, peer: PeerId, reassign: &mut Vec<(Hash, Hash)>) {
        let Some(replica) = self.peers.remove(&peer) else {
            return;
        };
        for (doc, entry) in replica.docs {
            let Entry::Occupied(mut p) = self.docs.entry(doc) else {
                continue;
            };
            let emit_unpin = if entry.pin.is_some() {
                let h = p.get_mut();
                h.pin_holders = h.pin_holders.saturating_sub(1);
                h.pin_holders == 0
            } else {
                false
            };
            p.get_mut().entry_refs -= 1;
            let space = p.get().space;
            if p.get().entry_refs == 0 {
                p.remove();
            } else {
                reassign.push((doc, space));
            }
            if emit_unpin {
                self.emit_pin(PinChange::Unpinned { doc });
            }
        }
    }

    /// Owner of `doc`: the oldest valid pin, breaking ties by peer id. Resolved
    /// per-client; ownership migrates to the next-oldest pinner when an owner
    /// leaves.
    fn owner(&self, space: Hash, doc: Hash) -> Option<PeerId> {
        if self.docs.get(&doc).is_none_or(|p| p.space != space) {
            return None;
        }
        self.peers
            .iter()
            .filter_map(|(pid, r)| Some((r.docs.get(&doc)?.pin?, *pid)))
            .min_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)))
            .map(|(_, pid)| pid)
    }

    /// Transform authority for `doc`: the latest explicit claimer (e.g. whoever
    /// last grabbed it), or the document's owner when no one has claimed, so an
    /// owner drives its objects by default until a peer grabs them.
    fn authority(&self, space: Hash, doc: Hash) -> Option<PeerId> {
        if self.docs.get(&doc).is_none_or(|p| p.space != space) {
            return None;
        }
        self.peers
            .iter()
            .filter_map(|(pid, r)| Some((r.docs.get(&doc)?.authority?, *pid)))
            .max_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)))
            .map(|(_, pid)| pid)
            .or_else(|| self.owner(space, doc))
    }

    fn merged_cell(&self, space: Hash, doc: Hash, key: &str) -> Option<Vec<u8>> {
        if self.docs.get(&doc).is_none_or(|p| p.space != space) {
            return None;
        }
        self.peers
            .iter()
            .filter_map(|(pid, r)| {
                let cell = r.docs.get(&doc)?.kv.get(key)?;
                Some((cell.at, *pid, cell.value.clone()))
            })
            .max_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)))
            .and_then(|(_, _, value)| value)
    }

    fn self_snapshot(&self, me: PeerId) -> Vec<DocSnapshot> {
        let Some(replica) = self.peers.get(&me) else {
            return Vec::new();
        };
        replica
            .docs
            .iter()
            .filter_map(|(doc, e)| {
                let space = self.docs.get(doc)?.space;
                Some(DocSnapshot {
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
                })
            })
            .collect()
    }
}

static PEER_STATE: LazyLock<Mutex<ReplicatedPeerState>> =
    LazyLock::new(|| Mutex::new(ReplicatedPeerState::default()));
static SENDER_TOKEN: AtomicU64 = AtomicU64::new(0);

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

/// Whether `at` is within the accepted clock skew of local time.
fn time_valid(at: u64) -> bool {
    at <= now_millis().saturating_add(MAX_CLOCK_SKEW_MILLIS)
}

/// Runs `reassign` against the document quotas after the state lock is
/// released, since the owner resolver re-enters the store.
fn settle_reassigns(reassign: Vec<(Hash, Hash)>) {
    for (doc, space) in reassign {
        reassign_document_in_space(doc, space);
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

/// Registers the scene's pin lifecycle stream, receiving a [`PinChange`] each
/// time a document gains its first or loses its last holder.
pub fn register_pin_stream() -> async_channel::Receiver<PinChange> {
    let (tx, rx) = async_channel::unbounded();
    PEER_STATE.lock().pin_tx = Some(tx);
    rx
}

/// Applies a remote peer's update to its replica.
///
/// Forged-future timestamps are rejected, and bytes are charged to the
/// document's owner/space; updates that exceed that quota are dropped rather
/// than stored. Document quotas are resolved before locking, since the owner
/// resolver re-enters the store.
pub fn apply_remote(peer: PeerId, msg: StateMsg) {
    let mut reassign = Vec::new();
    match msg {
        StateMsg::Snapshot(snaps) => {
            let resolved = snaps
                .into_iter()
                .map(|s| (document_quota(s.doc), s))
                .collect::<Vec<_>>();
            let mut state = PEER_STATE.lock();
            state.clear_peer(peer, &mut reassign);
            for (quota, s) in resolved {
                apply_snapshot_doc(&mut state, peer, s, &quota, &mut reassign);
            }
            drop(state);
        }
        StateMsg::Pin { doc, space, at } if time_valid(at) => {
            let quota = document_quota(doc);
            let mut state = PEER_STATE.lock();
            if state.ensure_entry(peer, doc, space, &quota) {
                state.set_pin(peer, doc, at, &mut reassign);
            } else {
                warn!(?peer, "pin dropped over quota");
            }
            drop(state);
        }
        StateMsg::Unpin { doc } => {
            let mut state = PEER_STATE.lock();
            state.clear_pin(peer, doc, &mut reassign);
            state.prune_entry(peer, doc, &mut reassign);
            drop(state);
        }
        StateMsg::Authority { doc, at } if time_valid(at) => {
            let quota = document_quota(doc);
            let mut state = PEER_STATE.lock();
            if let Some(space) = state.docs.get(&doc).map(|p| p.space)
                && state.ensure_entry(peer, doc, space, &quota)
            {
                state.set_authority(peer, doc, at);
            }
            drop(state);
        }
        StateMsg::Kv {
            doc,
            space,
            key,
            value,
            at,
        } if time_valid(at) => {
            let quota = document_quota(doc);
            let mut state = PEER_STATE.lock();
            if writer_permitted(&state, space, doc, peer)
                && state.ensure_entry(peer, doc, space, &quota)
            {
                state.write_kv(peer, doc, key, value, at);
            }
            drop(state);
        }
        _ => {}
    }
    settle_reassigns(reassign);
}

fn apply_snapshot_doc(
    state: &mut ReplicatedPeerState,
    peer: PeerId,
    s: DocSnapshot,
    quota: &Arc<Quota>,
    reassign: &mut Vec<(Hash, Hash)>,
) {
    if !state.ensure_entry(peer, s.doc, s.space, quota) {
        warn!(?peer, "snapshot doc dropped over quota");
        return;
    }
    if let Some(at) = s.pin.filter(|at| time_valid(*at)) {
        state.set_pin(peer, s.doc, at, reassign);
    }
    if let Some(at) = s.authority.filter(|at| time_valid(*at)) {
        state.set_authority(peer, s.doc, at);
    }
    let open = is_space_owned(state, s.space, s.doc);
    for kv in s.kv {
        if time_valid(kv.at) && (open || state.owner(s.space, s.doc) == Some(peer)) {
            state.write_kv(peer, s.doc, kv.key, kv.value, kv.at);
        }
    }
    state.prune_entry(peer, s.doc, reassign);
}

/// Removes a disconnected peer's replica, releasing its quota and handing off
/// ownership of any docs it owned to the next-oldest pinner.
pub fn remove_peer(peer: PeerId) {
    let mut reassign = Vec::new();
    PEER_STATE.lock().clear_peer(peer, &mut reassign);
    settle_reassigns(reassign);
}

/// Whether `doc` is space-owned: the space's base document, or a doc with no
/// pin-owner. Space-owned documents accept KV writes from any peer; peer-owned
/// documents only from their owner.
fn is_space_owned(state: &ReplicatedPeerState, space: Hash, doc: Hash) -> bool {
    doc == space || state.owner(space, doc).is_none()
}

fn writer_permitted(state: &ReplicatedPeerState, space: Hash, doc: Hash, writer: PeerId) -> bool {
    is_space_owned(state, space, doc) || state.owner(space, doc) == Some(writer)
}

pub fn self_pin(space: Hash, doc: Hash) -> bool {
    let Some(me) = self_peer_id() else {
        return false;
    };
    let at = now_millis();
    let quota = document_quota(doc);
    let mut reassign = Vec::new();
    let mut state = PEER_STATE.lock();
    let pinned = if state.ensure_entry(me, doc, space, &quota) {
        state.set_pin(me, doc, at, &mut reassign);
        state.broadcast(&StateMsg::Pin { doc, space, at });
        true
    } else {
        false
    };
    drop(state);
    settle_reassigns(reassign);
    pinned
}

pub fn self_unpin(doc: Hash) {
    let Some(me) = self_peer_id() else {
        return;
    };
    let mut reassign = Vec::new();
    let mut state = PEER_STATE.lock();
    let was_pinned = state
        .peers
        .get(&me)
        .and_then(|r| r.docs.get(&doc))
        .is_some_and(|e| e.pin.is_some());
    if was_pinned {
        state.clear_pin(me, doc, &mut reassign);
        state.prune_entry(me, doc, &mut reassign);
        state.broadcast(&StateMsg::Unpin { doc });
    }
    drop(state);
    settle_reassigns(reassign);
}

/// Claims transient transform authority over `doc` for the local peer, e.g. on
/// grabbing its rigid body. Distinct from ownership: latest claim wins.
pub fn claim_authority(space: Hash, doc: Hash) {
    let Some(me) = self_peer_id() else {
        return;
    };
    let at = now_millis();
    let quota = document_quota(doc);
    let mut state = PEER_STATE.lock();
    if state.ensure_entry(me, doc, space, &quota) {
        state.set_authority(me, doc, at);
        state.broadcast(&StateMsg::Authority { doc, at });
    }
}

pub fn doc_kv_set(space: Hash, doc: Hash, key: &str, value: &[u8]) -> Result<(), KvError> {
    if key.len() > KV_KEY_MAX_BYTES {
        return Err(KvError::KeyTooLong);
    }
    let Some(me) = self_peer_id() else {
        return Err(KvError::Other);
    };
    let at = now_millis();
    let quota = document_quota(doc);
    let mut reassign = Vec::new();
    let mut state = PEER_STATE.lock();
    let result = if !writer_permitted(&state, space, doc, me) {
        Err(KvError::NotOwner)
    } else if !state.ensure_entry(me, doc, space, &quota) {
        Err(KvError::QuotaExceeded)
    } else if state.write_kv(me, doc, key.to_string(), Some(value.to_vec()), at) {
        state.broadcast(&StateMsg::Kv {
            doc,
            space,
            key: key.to_string(),
            value: Some(value.to_vec()),
            at,
        });
        Ok(())
    } else {
        state.prune_entry(me, doc, &mut reassign);
        Err(KvError::QuotaExceeded)
    };
    drop(state);
    settle_reassigns(reassign);
    result
}

pub fn doc_kv_delete(space: Hash, doc: Hash, key: &str) {
    let Some(me) = self_peer_id() else {
        return;
    };
    let at = now_millis();
    let quota = document_quota(doc);
    let mut state = PEER_STATE.lock();
    if writer_permitted(&state, space, doc, me) && state.ensure_entry(me, doc, space, &quota) {
        state.write_kv(me, doc, key.to_string(), None, at);
        state.broadcast(&StateMsg::Kv {
            doc,
            space,
            key: key.to_string(),
            value: None,
            at,
        });
    }
}

#[must_use]
pub fn owner(space: Hash, doc: Hash) -> Option<PeerId> {
    PEER_STATE.lock().owner(space, doc)
}

#[must_use]
pub fn is_self_owner(space: Hash, doc: Hash) -> bool {
    self_peer_id().is_some_and(|me| owner(space, doc) == Some(me))
}

#[must_use]
pub fn authority(space: Hash, doc: Hash) -> Option<PeerId> {
    PEER_STATE.lock().authority(space, doc)
}

#[must_use]
pub fn is_self_authority(space: Hash, doc: Hash) -> bool {
    self_peer_id().is_some_and(|me| authority(space, doc) == Some(me))
}

#[must_use]
pub fn has_doc(space: Hash, doc: Hash) -> bool {
    PEER_STATE
        .lock()
        .docs
        .get(&doc)
        .is_some_and(|p| p.space == space)
}

#[must_use]
pub fn doc_kv_get(space: Hash, doc: Hash, key: &str) -> Option<Vec<u8>> {
    PEER_STATE.lock().merged_cell(space, doc, key)
}

#[must_use]
pub fn doc_kv_keys(space: Hash, doc: Hash) -> Vec<String> {
    let state = PEER_STATE.lock();
    if state.docs.get(&doc).is_none_or(|p| p.space != space) {
        return Vec::new();
    }
    let mut keys = HashSet::new();
    for r in state.peers.values() {
        if let Some(e) = r.docs.get(&doc) {
            keys.extend(e.kv.keys().cloned());
        }
    }
    let out = keys
        .into_iter()
        .filter(|k| state.merged_cell(space, doc, k).is_some())
        .collect();
    drop(state);
    out
}

#[must_use]
pub fn doc_kv_total_bytes(space: Hash, doc: Hash) -> usize {
    let state = PEER_STATE.lock();
    if state.docs.get(&doc).is_none_or(|p| p.space != space) {
        return 0;
    }
    let mut keys = HashSet::new();
    for r in state.peers.values() {
        if let Some(e) = r.docs.get(&doc) {
            keys.extend(e.kv.keys().cloned());
        }
    }
    let total = keys
        .into_iter()
        .filter_map(|k| state.merged_cell(space, doc, &k).map(|v| k.len() + v.len()))
        .sum();
    drop(state);
    total
}

/// Remote peers that hold `doc`, i.e. those we can sync the record from.
/// Excludes the local peer; the owner is listed first as the freshest source.
#[must_use]
pub fn doc_holders(doc: Hash) -> Vec<PeerId> {
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
pub fn space_of(doc: Hash) -> Option<Hash> {
    PEER_STATE.lock().docs.get(&doc).map(|p| p.space)
}

#[derive(Debug, Clone, Copy)]
pub enum KvError {
    KeyTooLong,
    NotOwner,
    QuotaExceeded,
    Other,
}

/// Read-only views of the peer store for the dev tools state inspector.
#[cfg(feature = "devtools")]
pub mod debug {
    use super::{
        Hash,
        PEER_STATE,
        PeerId,
        self_peer_id,
    };

    pub struct DebugKv {
        pub key:   String,
        pub bytes: Option<usize>,
    }

    pub struct DebugDoc {
        pub doc:       Hash,
        pub space:     Hash,
        pub pin:       Option<u64>,
        pub authority: Option<u64>,
        pub kv:        Vec<DebugKv>,
    }

    pub struct DebugPeer {
        pub peer:    PeerId,
        pub is_self: bool,
        pub docs:    Vec<DebugDoc>,
    }

    #[must_use]
    pub fn snapshot() -> Vec<DebugPeer> {
        let me = self_peer_id();
        let state = PEER_STATE.lock();
        state
            .peers
            .iter()
            .map(|(pid, r)| DebugPeer {
                peer:    *pid,
                is_self: Some(*pid) == me,
                docs:    r
                    .docs
                    .iter()
                    .map(|(doc, e)| DebugDoc {
                        doc:       *doc,
                        space:     state.docs.get(doc).map_or(*doc, |p| p.space),
                        pin:       e.pin,
                        authority: e.authority,
                        kv:        e
                            .kv
                            .iter()
                            .map(|(k, c)| DebugKv {
                                key:   k.clone(),
                                bytes: c.value.as_ref().map(Vec::len),
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect()
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
    use unavi_quota::limits::Limits;

    use super::*;
    use crate::peer::set_self_peer_id;

    fn h(seed: &[u8]) -> Hash {
        blake3::hash(seed)
    }

    fn kv(key: &str, value: Option<&[u8]>, at: u64) -> KvSnapshot {
        KvSnapshot {
            key: key.into(),
            value: value.map(<[u8]>::to_vec),
            at,
        }
    }

    #[test]
    fn snapshot_then_deltas_converge() {
        let _g = TEST_LOCK.lock();
        reset();
        let peer = [9u8; 32];
        let space = h(b"converge-space");
        let doc = h(b"converge-doc");

        apply_remote(
            peer,
            StateMsg::Snapshot(vec![DocSnapshot {
                doc,
                space,
                pin: Some(1),
                authority: None,
                kv: vec![kv("k", Some(b"v1"), 1)],
            }]),
        );
        assert_eq!(doc_kv_get(space, doc, "k").as_deref(), Some(&b"v1"[..]));
        assert_eq!(owner(space, doc), Some(peer));

        apply_remote(
            peer,
            StateMsg::Kv {
                doc,
                space,
                key: "k".into(),
                value: Some(b"v2".to_vec()),
                at: 2,
            },
        );
        assert_eq!(doc_kv_get(space, doc, "k").as_deref(), Some(&b"v2"[..]));

        apply_remote(
            peer,
            StateMsg::Kv {
                doc,
                space,
                key: "k".into(),
                value: None,
                at: 3,
            },
        );
        assert_eq!(doc_kv_get(space, doc, "k"), None);

        remove_peer(peer);
        reset();
    }

    #[test]
    fn owner_is_oldest_pin_and_hands_off() {
        let _g = TEST_LOCK.lock();
        reset();
        let space = h(b"oldest-space");
        let doc = h(b"oldest-doc");
        let early = [1u8; 32];
        let late = [2u8; 32];

        apply_remote(late, StateMsg::Pin { doc, space, at: 20 });
        assert_eq!(owner(space, doc), Some(late));

        // An older pin takes ownership regardless of arrival order.
        apply_remote(early, StateMsg::Pin { doc, space, at: 10 });
        assert_eq!(owner(space, doc), Some(early));

        // The owner leaving hands ownership to the next-oldest pinner.
        remove_peer(early);
        assert_eq!(owner(space, doc), Some(late));

        remove_peer(late);
        assert_eq!(owner(space, doc), None);
        reset();
    }

    #[test]
    fn authority_is_latest_and_independent_of_owner() {
        let _g = TEST_LOCK.lock();
        reset();
        let space = h(b"auth-space");
        let doc = h(b"auth-doc");
        let owner_peer = [1u8; 32];
        let grabber = [2u8; 32];

        apply_remote(owner_peer, StateMsg::Pin { doc, space, at: 10 });
        apply_remote(grabber, StateMsg::Pin { doc, space, at: 20 });
        assert_eq!(owner(space, doc), Some(owner_peer));

        // With no explicit claim, authority defaults to the owner.
        assert_eq!(authority(space, doc), Some(owner_peer));

        // A later authority claim by a non-owner wins authority but not ownership.
        apply_remote(grabber, StateMsg::Authority { doc, at: 200 });
        assert_eq!(authority(space, doc), Some(grabber));
        assert_eq!(owner(space, doc), Some(owner_peer));

        remove_peer(owner_peer);
        remove_peer(grabber);
        reset();
    }

    #[test]
    fn rejects_far_future_timestamps() {
        let _g = TEST_LOCK.lock();
        reset();
        let peer = [3u8; 32];
        let space = h(b"skew-space");
        let doc = h(b"skew-doc");

        // A forged-future pin and authority are ignored.
        apply_remote(
            peer,
            StateMsg::Pin {
                doc,
                space,
                at: u64::MAX,
            },
        );
        assert!(!has_doc(space, doc));

        // Establish the doc with a valid pin, then a forged-future KV is dropped.
        apply_remote(peer, StateMsg::Pin { doc, space, at: 1 });
        apply_remote(
            peer,
            StateMsg::Kv {
                doc,
                space,
                key: "k".into(),
                value: Some(b"future".to_vec()),
                at: u64::MAX,
            },
        );
        assert_eq!(doc_kv_get(space, doc, "k"), None);

        remove_peer(peer);
        reset();
    }

    #[test]
    fn far_future_snapshot_cell_dropped_siblings_kept() {
        let _g = TEST_LOCK.lock();
        reset();
        let peer = [4u8; 32];
        let space = h(b"snap-skew-space");
        let doc = h(b"snap-skew-doc");

        apply_remote(
            peer,
            StateMsg::Snapshot(vec![DocSnapshot {
                doc,
                space,
                pin: Some(1),
                authority: None,
                kv: vec![kv("ok", Some(b"v"), 1), kv("bad", Some(b"v"), u64::MAX)],
            }]),
        );
        assert_eq!(doc_kv_get(space, doc, "ok").as_deref(), Some(&b"v"[..]));
        assert_eq!(doc_kv_get(space, doc, "bad"), None);

        remove_peer(peer);
        reset();
    }

    #[test]
    fn pin_changes_emit_on_holder_transitions() {
        let _g = TEST_LOCK.lock();
        reset();
        let rx = register_pin_stream();
        let space = h(b"pinchange-space");
        let doc = h(b"pinchange-doc");
        let a = [1u8; 32];
        let b = [2u8; 32];

        apply_remote(a, StateMsg::Pin { doc, space, at: 1 });
        assert!(matches!(
            rx.try_recv(),
            Ok(PinChange::Pinned { doc: d, space: s }) if d == doc && s == space
        ));

        apply_remote(b, StateMsg::Pin { doc, space, at: 2 });
        assert!(rx.try_recv().is_err());

        apply_remote(a, StateMsg::Unpin { doc });
        assert!(rx.try_recv().is_err());

        remove_peer(b);
        assert!(matches!(rx.try_recv(), Ok(PinChange::Unpinned { doc: d }) if d == doc));
        reset();
    }

    #[test]
    fn kv_write_gated_by_ownership() {
        let _g = TEST_LOCK.lock();
        reset();
        set_self_peer_id([1u8; 32]);
        let space = h(b"perm-space");

        // A peer doc the local peer owns (only pinner) is writable.
        let owned = h(b"perm-owned-doc");
        assert!(self_pin(space, owned));
        assert!(doc_kv_set(space, owned, "k", b"v").is_ok());

        // A peer doc owned by an older remote pinner is not writable locally.
        let foreign = h(b"perm-foreign-doc");
        apply_remote(
            [2u8; 32],
            StateMsg::Pin {
                doc: foreign,
                space,
                at: 1,
            },
        );
        assert!(self_pin(space, foreign));
        assert!(matches!(
            doc_kv_set(space, foreign, "k", b"v"),
            Err(KvError::NotOwner)
        ));

        // The space's base document is open to all writers.
        assert!(doc_kv_set(space, space, "k", b"v").is_ok());

        reset();
    }

    #[test]
    fn small_overwrite_succeeds_at_full_cap() {
        let _g = TEST_LOCK.lock();
        reset();
        set_self_peer_id([8u8; 32]);
        let space = h(b"full-space");
        let doc = h(b"full-doc");
        assert!(self_pin(space, doc));

        let cap = *Limits::document()
            .stock
            .get(&Stock::KvMemory)
            .expect("document caps kv memory") as usize;
        let big = vec![0u8; cap - 64];
        doc_kv_set(space, doc, "a", &big).expect("fills near the cap");

        // A new key that would grow past the cap is refused.
        assert!(matches!(
            doc_kv_set(space, doc, "b", &[0u8; 128]),
            Err(KvError::QuotaExceeded)
        ));

        // Overwriting the large value with a tiny one succeeds despite the cap.
        doc_kv_set(space, doc, "a", b"x").expect("shrinking overwrite at a full cap");
        assert_eq!(doc_kv_get(space, doc, "a").as_deref(), Some(&b"x"[..]));

        reset();
    }

    #[test]
    fn dropping_replica_refunds_quota() {
        let _g = TEST_LOCK.lock();
        reset();
        let peer = [11u8; 32];
        let space = h(b"refund-space");
        let doc = h(b"refund-doc");

        apply_remote(peer, StateMsg::Pin { doc, space, at: 1 });
        apply_remote(
            peer,
            StateMsg::Kv {
                doc,
                space,
                key: "k".into(),
                value: Some(b"value".to_vec()),
                at: 2,
            },
        );
        let quota = crate::quota::document_quota(doc);
        assert!(quota.usage(Stock::KvMemory) > 0);
        assert_eq!(quota.usage(Stock::Documents), 1);

        // Dropping the peer's replica refunds every hold it owned, with no
        // recompute.
        remove_peer(peer);
        assert_eq!(quota.usage(Stock::KvMemory), 0);
        assert_eq!(quota.usage(Stock::Documents), 0);

        reset();
    }

    #[test]
    fn kv_syncs_without_pinning_on_open_doc() {
        let _g = TEST_LOCK.lock();
        reset();
        let rx = register_pin_stream();
        let peer = [5u8; 32];
        // A space-owned (open) doc: doc == space.
        let space = h(b"open-doc");
        let doc = space;

        apply_remote(
            peer,
            StateMsg::Kv {
                doc,
                space,
                key: "k".into(),
                value: Some(b"v".to_vec()),
                at: 1,
            },
        );
        assert_eq!(doc_kv_get(space, doc, "k").as_deref(), Some(&b"v"[..]));
        assert!(has_doc(space, doc));
        assert!(doc_holders(doc).is_empty());
        assert!(rx.try_recv().is_err());

        remove_peer(peer);
        reset();
    }
}
