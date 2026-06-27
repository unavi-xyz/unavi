use std::{
    collections::{
        HashMap,
        HashSet,
        hash_map::Entry,
    },
    sync::{
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
    registry::peer_quota,
};

use crate::{
    peer::self_peer_id,
    state::message::{
        DocSnapshot,
        KvSnapshot,
        StateMsg,
    },
};

pub type PeerId = [u8; 32];

/// A change to whether any peer pins a document, emitted on the 0↔1 holder
/// transition so the scene can react without scanning every pin each frame.
#[derive(Debug, Clone)]
pub enum PinChange {
    Pinned { doc: Hash, space: Hash },
    Unpinned { doc: Hash },
}

#[derive(Default)]
struct PeerState {
    docs: HashMap<Hash, DocEntry>,
}

struct DocEntry {
    space:  Hash,
    /// Whether this peer pins the doc for scene instancing. KV state syncs
    /// independently, so a doc can carry KV without being pinned.
    pinned: bool,
    claim:  Option<u64>,
    kv:     HashMap<String, KvValue>,
}

impl DocEntry {
    fn new(space: Hash) -> Self {
        Self {
            space,
            pinned: false,
            claim: None,
            kv: HashMap::new(),
        }
    }

    /// A doc no longer worth retaining: not pinned and holding no KV state.
    fn is_empty(&self) -> bool {
        !self.pinned && self.kv.is_empty()
    }
}

/// A KV cell. `value: None` is a tombstone retained so a delete keeps winning
/// over an older live write on another peer.
struct KvValue {
    at:    u64,
    value: Option<Vec<u8>>,
}

/// Holds every peer's replicated state (self included) alongside the live delta
/// senders.
#[derive(Default)]
struct Store {
    peers:   HashMap<PeerId, PeerState>,
    senders: HashMap<u64, async_channel::Sender<StateMsg>>,
    pins:    HashMap<Hash, u32>,
    pin_tx:  Option<async_channel::Sender<PinChange>>,
}

impl Store {
    fn broadcast(&mut self, msg: &StateMsg) {
        self.senders
            .retain(|_, tx| tx.try_send(msg.clone()).is_ok());
    }

    /// Records a holder for `doc`, emitting [`PinChange::Pinned`] on the first.
    fn note_pin(&mut self, doc: Hash, space: Hash) {
        let holders = self.pins.entry(doc).or_insert(0);
        *holders += 1;
        if *holders == 1
            && let Some(tx) = &self.pin_tx
        {
            let _ = tx.try_send(PinChange::Pinned { doc, space });
        }
    }

    /// Drops a holder for `doc`, emitting [`PinChange::Unpinned`] on the last.
    fn note_unpin(&mut self, doc: Hash) {
        let Entry::Occupied(mut e) = self.pins.entry(doc) else {
            return;
        };
        let holders = e.get_mut();
        *holders = holders.saturating_sub(1);
        if *holders == 0 {
            e.remove();
            if let Some(tx) = &self.pin_tx {
                let _ = tx.try_send(PinChange::Unpinned { doc });
            }
        }
    }
}

static PEER_STORE: LazyLock<Mutex<Store>> = LazyLock::new(|| Mutex::new(Store::default()));
static SENDER_TOKEN: AtomicU64 = AtomicU64::new(0);

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

fn cell_bytes(key: &str, cell: &KvValue) -> u64 {
    (key.len() + cell.value.as_ref().map_or(0, Vec::len)) as u64
}

fn entry_bytes(entry: &DocEntry) -> u64 {
    entry.kv.iter().map(|(k, c)| cell_bytes(k, c)).sum()
}

fn self_snapshot(store: &Store) -> Vec<DocSnapshot> {
    let Some(me) = self_peer_id() else {
        return Vec::new();
    };
    store
        .peers
        .get(&me)
        .map(|ps| {
            ps.docs
                .iter()
                .map(|(doc, e)| DocSnapshot {
                    doc:    *doc,
                    space:  e.space,
                    pinned: e.pinned,
                    claim:  e.claim,
                    kv:     e
                        .kv
                        .iter()
                        .map(|(k, c)| KvSnapshot {
                            key:   k.clone(),
                            value: c.value.clone(),
                            at:    c.at,
                        })
                        .collect(),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Owner of `doc` in `space`: the latest claimer, breaking ties by peer id.
fn resolve_owner(store: &Store, space: Hash, doc: Hash) -> Option<PeerId> {
    store
        .peers
        .iter()
        .filter_map(|(pid, ps)| {
            let e = ps.docs.get(&doc)?;
            (e.space == space).then_some(())?;
            Some((e.claim?, *pid))
        })
        .max_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)))
        .map(|(_, pid)| pid)
}

/// Merges `key` across every peer that holds `doc` in `space`, last-write-wins
/// by `(at, peer)`. A winning tombstone resolves to `None`.
fn merged_cell(store: &Store, space: Hash, doc: Hash, key: &str) -> Option<Vec<u8>> {
    store
        .peers
        .iter()
        .filter_map(|(pid, ps)| {
            let e = ps.docs.get(&doc)?;
            (e.space == space).then_some(())?;
            let cell = e.kv.get(key)?;
            Some((cell.at, *pid, cell.value.clone()))
        })
        .max_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)))
        .and_then(|(_, _, value)| value)
}

/// Registers a delta stream, returning its cancel token and a receiver whose
/// first message is a full snapshot of the local peer's state.
pub fn register_stream() -> (u64, async_channel::Receiver<StateMsg>) {
    let (tx, rx) = async_channel::unbounded();
    let mut store = PEER_STORE.lock();
    let snapshot = self_snapshot(&store);
    let _ = tx.try_send(StateMsg::Snapshot(snapshot));
    let token = SENDER_TOKEN.fetch_add(1, Ordering::Relaxed);
    store.senders.insert(token, tx);
    drop(store);
    (token, rx)
}

pub fn unregister_stream(token: u64) {
    PEER_STORE.lock().senders.remove(&token);
}

/// Registers the scene's pin lifecycle stream, receiving a [`PinChange`] each
/// time a document gains its first or loses its last holder.
pub fn register_pin_stream() -> async_channel::Receiver<PinChange> {
    let (tx, rx) = async_channel::unbounded();
    PEER_STORE.lock().pin_tx = Some(tx);
    rx
}

/// Applies a remote peer's update to its replica.
///
/// Bytes added are charged against that peer's quota; updates that would exceed
/// its cap are dropped rather than stored, bounding what one peer can cost us.
pub fn apply_remote(peer: PeerId, msg: StateMsg) {
    let quota = peer_quota(Hash::from(peer));
    let mut store = PEER_STORE.lock();
    let mut added = Vec::<(Hash, Hash)>::new();
    let mut removed = Vec::<Hash>::new();
    {
        let docs = &mut store.peers.entry(peer).or_default().docs;
        match msg {
            StateMsg::Snapshot(snaps) => {
                apply_snapshot(docs, &quota, peer, snaps, &mut added, &mut removed);
            }
            StateMsg::Pin { doc, space } => match docs.entry(doc) {
                Entry::Vacant(v) => {
                    if quota.try_charge(Stock::Documents, 1).is_ok() {
                        let mut e = DocEntry::new(space);
                        e.pinned = true;
                        v.insert(e);
                        added.push((doc, space));
                    } else {
                        warn!(?peer, "pin dropped over peer quota");
                    }
                }
                Entry::Occupied(mut o) if !o.get().pinned => {
                    o.get_mut().pinned = true;
                    added.push((doc, o.get().space));
                }
                Entry::Occupied(_) => {}
            },
            StateMsg::Unpin { doc } => {
                if let Entry::Occupied(mut o) = docs.entry(doc) {
                    if o.get().pinned {
                        o.get_mut().pinned = false;
                        o.get_mut().claim = None;
                        removed.push(doc);
                    }
                    if o.get().is_empty() {
                        let e = o.remove();
                        quota.release(Stock::Documents, 1);
                        quota.release(Stock::KvMemory, entry_bytes(&e));
                    }
                }
            }
            StateMsg::Claim { doc, at } => {
                if let Some(e) = docs.get_mut(&doc) {
                    e.claim = Some(at);
                }
            }
            StateMsg::Kv {
                doc,
                space,
                key,
                value,
                at,
            } => match docs.entry(doc) {
                Entry::Occupied(mut o) => {
                    apply_remote_kv(&quota, o.get_mut(), key, value, at, peer);
                }
                Entry::Vacant(v) => {
                    if quota.try_charge(Stock::Documents, 1).is_ok() {
                        apply_remote_kv(
                            &quota,
                            v.insert(DocEntry::new(space)),
                            key,
                            value,
                            at,
                            peer,
                        );
                    } else {
                        warn!(?peer, "kv doc dropped over peer quota");
                    }
                }
            },
        }
    }
    for (doc, space) in added {
        store.note_pin(doc, space);
    }
    for doc in removed {
        store.note_unpin(doc);
    }
    drop(store);
}

/// Replaces a peer's replica with a fresh snapshot, recording pinned-doc
/// transitions in `added`/`removed` so holder counts and fetches reconcile.
fn apply_snapshot(
    docs: &mut HashMap<Hash, DocEntry>,
    quota: &Quota,
    peer: PeerId,
    snaps: Vec<DocSnapshot>,
    added: &mut Vec<(Hash, Hash)>,
    removed: &mut Vec<Hash>,
) {
    let old_pinned = docs
        .iter()
        .filter(|(_, e)| e.pinned)
        .map(|(d, e)| (*d, e.space))
        .collect::<HashMap<_, _>>();
    quota.release(Stock::Documents, docs.len() as u64);
    quota.release(Stock::KvMemory, docs.values().map(entry_bytes).sum());
    docs.clear();

    let mut dropped = 0u32;
    for s in snaps {
        let bytes =
            s.kv.iter()
                .map(|k| (k.key.len() + k.value.as_ref().map_or(0, Vec::len)) as u64)
                .sum();
        if quota.try_charge(Stock::Documents, 1).is_err() {
            dropped += 1;
            continue;
        }
        if quota.try_charge(Stock::KvMemory, bytes).is_err() {
            quota.release(Stock::Documents, 1);
            dropped += 1;
            continue;
        }
        docs.insert(
            s.doc,
            DocEntry {
                space:  s.space,
                pinned: s.pinned,
                claim:  s.claim,
                kv:     s
                    .kv
                    .into_iter()
                    .map(|k| {
                        (
                            k.key,
                            KvValue {
                                at:    k.at,
                                value: k.value,
                            },
                        )
                    })
                    .collect(),
            },
        );
    }
    if dropped > 0 {
        warn!(?peer, dropped, "snapshot entries dropped over peer quota");
    }
    added.extend(
        docs.iter()
            .filter(|(d, e)| e.pinned && !old_pinned.contains_key(*d))
            .map(|(d, e)| (*d, e.space)),
    );
    removed.extend(
        old_pinned
            .keys()
            .filter(|d| docs.get(*d).is_none_or(|e| !e.pinned))
            .copied(),
    );
}

fn apply_remote_kv(
    quota: &Quota,
    entry: &mut DocEntry,
    key: String,
    value: Option<Vec<u8>>,
    at: u64,
    peer: PeerId,
) {
    if entry.kv.get(&key).is_some_and(|cur| at < cur.at) {
        return;
    }
    let old = entry.kv.get(&key).map_or(0, |c| cell_bytes(&key, c));
    let new = (key.len() + value.as_ref().map_or(0, Vec::len)) as u64;
    if new > old {
        if quota.try_charge(Stock::KvMemory, new - old).is_err() {
            warn!(?peer, "kv update dropped over peer quota");
            return;
        }
    } else {
        quota.release(Stock::KvMemory, old - new);
    }
    entry.kv.insert(key, KvValue { at, value });
}

/// Removes a disconnected peer's replica and releases its quota.
pub fn remove_peer(peer: PeerId) {
    let mut store = PEER_STORE.lock();
    let Some(ps) = store.peers.remove(&peer) else {
        return;
    };
    let quota = peer_quota(Hash::from(peer));
    quota.release(Stock::Documents, ps.docs.len() as u64);
    quota.release(Stock::KvMemory, ps.docs.values().map(entry_bytes).sum());
    for doc in ps.docs.keys() {
        store.note_unpin(*doc);
    }
    drop(store);
}

pub fn self_pin(space: Hash, doc: Hash) {
    let Some(me) = self_peer_id() else {
        return;
    };
    let mut store = PEER_STORE.lock();
    let newly = {
        let entry = store
            .peers
            .entry(me)
            .or_default()
            .docs
            .entry(doc)
            .or_insert_with(|| DocEntry::new(space));
        let newly = !entry.pinned;
        entry.pinned = true;
        newly
    };
    if newly {
        store.note_pin(doc, space);
        store.broadcast(&StateMsg::Pin { doc, space });
    }
}

pub fn self_claim(space: Hash, doc: Hash) {
    let Some(me) = self_peer_id() else {
        return;
    };
    let at = now_millis();
    let mut store = PEER_STORE.lock();
    let newly = {
        let entry = store
            .peers
            .entry(me)
            .or_default()
            .docs
            .entry(doc)
            .or_insert_with(|| DocEntry::new(space));
        entry.claim = Some(at);
        let newly = !entry.pinned;
        entry.pinned = true;
        newly
    };
    if newly {
        store.note_pin(doc, space);
        store.broadcast(&StateMsg::Pin { doc, space });
    }
    store.broadcast(&StateMsg::Claim { doc, at });
}

pub fn self_unpin(doc: Hash) {
    let Some(me) = self_peer_id() else {
        return;
    };
    let mut store = PEER_STORE.lock();
    let unpinned = store.peers.get_mut(&me).is_some_and(|p| {
        let Entry::Occupied(mut o) = p.docs.entry(doc) else {
            return false;
        };
        let was_pinned = o.get().pinned;
        o.get_mut().pinned = false;
        o.get_mut().claim = None;
        if o.get().is_empty() {
            o.remove();
        }
        was_pinned
    });
    if unpinned {
        store.note_unpin(doc);
        store.broadcast(&StateMsg::Unpin { doc });
    }
}

pub fn self_kv_set(space: Hash, doc: Hash, key: &str, value: &[u8]) {
    let Some(me) = self_peer_id() else {
        return;
    };
    let at = now_millis();
    let mut store = PEER_STORE.lock();
    store
        .peers
        .entry(me)
        .or_default()
        .docs
        .entry(doc)
        .or_insert_with(|| DocEntry::new(space))
        .kv
        .insert(
            key.to_string(),
            KvValue {
                at,
                value: Some(value.to_vec()),
            },
        );
    store.broadcast(&StateMsg::Kv {
        doc,
        space,
        key: key.to_string(),
        value: Some(value.to_vec()),
        at,
    });
}

/// Writes a tombstone for `key`, so the delete merges as last-write-wins.
pub fn self_kv_delete(space: Hash, doc: Hash, key: &str) {
    let Some(me) = self_peer_id() else {
        return;
    };
    let at = now_millis();
    let mut store = PEER_STORE.lock();
    store
        .peers
        .entry(me)
        .or_default()
        .docs
        .entry(doc)
        .or_insert_with(|| DocEntry::new(space))
        .kv
        .insert(key.to_string(), KvValue { at, value: None });
    store.broadcast(&StateMsg::Kv {
        doc,
        space,
        key: key.to_string(),
        value: None,
        at,
    });
}

/// `(live total bytes, byte length of `key`'s current value, key live)` for the
/// local peer's entry, sized as `doc_kv_set` charges quota.
pub fn self_kv_accounting(doc: Hash, key: &str) -> (usize, usize, bool) {
    let Some(me) = self_peer_id() else {
        return (0, 0, false);
    };
    let store = PEER_STORE.lock();
    let mut current = 0;
    let mut old_value_len = 0;
    let mut key_present = false;
    if let Some(entry) = store.peers.get(&me).and_then(|p| p.docs.get(&doc)) {
        for (k, c) in &entry.kv {
            let Some(v) = &c.value else {
                continue;
            };
            current += k.len() + v.len();
            if k == key {
                old_value_len = v.len();
                key_present = true;
            }
        }
    }
    drop(store);
    (current, old_value_len, key_present)
}

#[must_use]
pub fn owner(space: Hash, doc: Hash) -> Option<PeerId> {
    resolve_owner(&PEER_STORE.lock(), space, doc)
}

#[must_use]
pub fn is_self_owner(space: Hash, doc: Hash) -> bool {
    self_peer_id().is_some_and(|me| owner(space, doc) == Some(me))
}

#[must_use]
pub fn has_doc(space: Hash, doc: Hash) -> bool {
    PEER_STORE
        .lock()
        .peers
        .values()
        .any(|p| p.docs.get(&doc).is_some_and(|e| e.space == space))
}

#[must_use]
pub fn kv_get(space: Hash, doc: Hash, key: &str) -> Option<Vec<u8>> {
    merged_cell(&PEER_STORE.lock(), space, doc, key)
}

#[must_use]
pub fn kv_keys(space: Hash, doc: Hash) -> Vec<String> {
    let store = PEER_STORE.lock();
    let mut keys = HashSet::new();
    for ps in store.peers.values() {
        if let Some(e) = ps.docs.get(&doc).filter(|e| e.space == space) {
            keys.extend(e.kv.keys().cloned());
        }
    }
    let out = keys
        .into_iter()
        .filter(|k| merged_cell(&store, space, doc, k).is_some())
        .collect();
    drop(store);
    out
}

#[must_use]
pub fn kv_total_bytes(space: Hash, doc: Hash) -> usize {
    let store = PEER_STORE.lock();
    let mut keys = HashSet::new();
    for ps in store.peers.values() {
        if let Some(e) = ps.docs.get(&doc).filter(|e| e.space == space) {
            keys.extend(e.kv.keys().cloned());
        }
    }
    let total = keys
        .into_iter()
        .filter_map(|k| merged_cell(&store, space, doc, &k).map(|v| k.len() + v.len()))
        .sum();
    drop(store);
    total
}

/// Remote peers that hold `doc`, i.e. those we can sync the record from.
/// Excludes the local peer, since we read our own store first anyway. The
/// current owner is listed first, as the freshest source.
#[must_use]
pub fn doc_holders(doc: Hash) -> Vec<PeerId> {
    let me = self_peer_id();
    let store = PEER_STORE.lock();
    let space = store
        .peers
        .values()
        .find_map(|ps| ps.docs.get(&doc).map(|e| e.space));
    let owner = space.and_then(|space| resolve_owner(&store, space, doc));
    let mut holders = store
        .peers
        .iter()
        .filter(|(pid, ps)| {
            Some(**pid) != me && Some(**pid) != owner && ps.docs.get(&doc).is_some_and(|e| e.pinned)
        })
        .map(|(pid, _)| *pid)
        .collect::<Vec<_>>();
    if let Some(owner) = owner.filter(|o| Some(*o) != me) {
        holders.insert(0, owner);
    }
    drop(store);
    holders
}

/// The space `doc` is pinned in, per any peer's replica. Lets membership
/// resolve a synced doc's space without a local ownership claim.
#[must_use]
pub fn space_of(doc: Hash) -> Option<Hash> {
    PEER_STORE
        .lock()
        .peers
        .values()
        .find_map(|ps| ps.docs.get(&doc).map(|e| e.space))
}

#[cfg(test)]
pub fn reset() {
    let mut store = PEER_STORE.lock();
    store.peers.clear();
    store.senders.clear();
    store.pins.clear();
    store.pin_tx = None;
}

/// Serializes tests, which share the global store and self-peer identity.
#[cfg(test)]
pub static TEST_LOCK: Mutex<()> = Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;

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
                pinned: true,
                claim: Some(1),
                kv: vec![kv("k", Some(b"v1"), 1)],
            }]),
        );
        assert_eq!(kv_get(space, doc, "k").as_deref(), Some(&b"v1"[..]));
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
        assert_eq!(kv_get(space, doc, "k").as_deref(), Some(&b"v2"[..]));

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
        assert_eq!(kv_get(space, doc, "k"), None);

        remove_peer(peer);
    }

    #[test]
    fn kv_merges_across_peers_last_write_wins() {
        let _g = TEST_LOCK.lock();
        reset();
        let a = [1u8; 32];
        let b = [2u8; 32];
        let space = h(b"merge-space");
        let doc = h(b"merge-doc");

        apply_remote(a, StateMsg::Pin { doc, space });
        apply_remote(b, StateMsg::Pin { doc, space });

        // Distinct keys union together.
        apply_remote(
            a,
            StateMsg::Kv {
                doc,
                space,
                key: "a".into(),
                value: Some(b"x".to_vec()),
                at: 1,
            },
        );
        apply_remote(
            b,
            StateMsg::Kv {
                doc,
                space,
                key: "b".into(),
                value: Some(b"y".to_vec()),
                at: 1,
            },
        );
        let mut keys = kv_keys(space, doc);
        keys.sort();
        assert_eq!(keys, vec!["a".to_string(), "b".to_string()]);

        // Same key resolves to the later write regardless of source peer.
        apply_remote(
            a,
            StateMsg::Kv {
                doc,
                space,
                key: "k".into(),
                value: Some(b"old".to_vec()),
                at: 10,
            },
        );
        apply_remote(
            b,
            StateMsg::Kv {
                doc,
                space,
                key: "k".into(),
                value: Some(b"new".to_vec()),
                at: 20,
            },
        );
        assert_eq!(kv_get(space, doc, "k").as_deref(), Some(&b"new"[..]));

        // A newer tombstone wins over an older live write on the other peer.
        apply_remote(
            a,
            StateMsg::Kv {
                doc,
                space,
                key: "k".into(),
                value: None,
                at: 30,
            },
        );
        assert_eq!(kv_get(space, doc, "k"), None);

        remove_peer(a);
        remove_peer(b);
    }

    #[test]
    fn kv_syncs_without_pinning() {
        let _g = TEST_LOCK.lock();
        reset();
        let rx = register_pin_stream();
        let peer = [5u8; 32];
        let space = h(b"kvnopin-space");
        let doc = h(b"kvnopin-doc");

        // A bare Kv delta self-creates the doc's state with no prior pin.
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

        // Readable by scripts, yet never announced as a pin to instance/fetch.
        assert_eq!(kv_get(space, doc, "k").as_deref(), Some(&b"v"[..]));
        assert!(has_doc(space, doc));
        assert!(doc_holders(doc).is_empty());
        assert!(rx.try_recv().is_err());

        remove_peer(peer);
        reset();
    }

    #[test]
    fn owner_is_latest_claimer_and_hands_off() {
        let _g = TEST_LOCK.lock();
        reset();
        let space = h(b"handoff-space");
        let doc = h(b"handoff-doc");
        let early = [1u8; 32];
        let late = [2u8; 32];

        apply_remote(early, StateMsg::Pin { doc, space });
        apply_remote(early, StateMsg::Claim { doc, at: 10 });
        assert_eq!(owner(space, doc), Some(early));

        apply_remote(late, StateMsg::Pin { doc, space });
        apply_remote(late, StateMsg::Claim { doc, at: 20 });
        assert_eq!(owner(space, doc), Some(late));

        // The latest claimer leaving hands ownership back to the prior claimer.
        remove_peer(late);
        assert_eq!(owner(space, doc), Some(early));

        remove_peer(early);
        assert_eq!(owner(space, doc), None);
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

        apply_remote(a, StateMsg::Pin { doc, space });
        assert!(matches!(
            rx.try_recv(),
            Ok(PinChange::Pinned { doc: d, space: s }) if d == doc && s == space
        ));

        // A second holder does not re-emit.
        apply_remote(b, StateMsg::Pin { doc, space });
        assert!(rx.try_recv().is_err());

        // Losing one of two holders keeps the doc pinned.
        apply_remote(a, StateMsg::Unpin { doc });
        assert!(rx.try_recv().is_err());

        // The last holder leaving emits Unpinned.
        remove_peer(b);
        assert!(matches!(rx.try_recv(), Ok(PinChange::Unpinned { doc: d }) if d == doc));

        reset();
    }

    #[test]
    fn snapshot_resend_does_not_churn_pins() {
        let _g = TEST_LOCK.lock();
        reset();
        let rx = register_pin_stream();
        let space = h(b"churn-space");
        let doc = h(b"churn-doc");
        let peer = [7u8; 32];

        apply_remote(
            peer,
            StateMsg::Snapshot(vec![DocSnapshot {
                doc,
                space,
                pinned: true,
                claim: None,
                kv: vec![],
            }]),
        );
        assert!(matches!(rx.try_recv(), Ok(PinChange::Pinned { .. })));

        // Re-snapshotting the same doc must not emit unpin/pin churn.
        apply_remote(
            peer,
            StateMsg::Snapshot(vec![DocSnapshot {
                doc,
                space,
                pinned: true,
                claim: Some(1),
                kv: vec![],
            }]),
        );
        assert!(rx.try_recv().is_err());

        // Dropping the doc from a later snapshot emits Unpinned.
        apply_remote(peer, StateMsg::Snapshot(vec![]));
        assert!(matches!(rx.try_recv(), Ok(PinChange::Unpinned { doc: d }) if d == doc));

        reset();
    }

    #[test]
    fn unpin_removes_pin_and_owner() {
        let _g = TEST_LOCK.lock();
        reset();
        let space = h(b"unpin-space");
        let doc = h(b"unpin-doc");
        let peer = [3u8; 32];

        apply_remote(peer, StateMsg::Pin { doc, space });
        apply_remote(peer, StateMsg::Claim { doc, at: 5 });
        assert!(has_doc(space, doc));
        assert_eq!(owner(space, doc), Some(peer));

        apply_remote(peer, StateMsg::Unpin { doc });
        assert!(!has_doc(space, doc));
        assert_eq!(owner(space, doc), None);

        remove_peer(peer);
    }
}
