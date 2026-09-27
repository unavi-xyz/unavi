use std::collections::{
    HashMap,
    HashSet,
};

use anyhow::anyhow;
use async_channel::Sender;
use bevy::log::warn;
use bytes::Bytes;
use hsd::{
    bounds::MAX_ENTRY_BYTES,
    key,
    state::entry::Entry,
};
use iroh_blobs::Hash;
use iroh_docs::{
    engine::LiveEvent,
    store::Query,
};
use n0_future::StreamExt;
use wds::document::Document;

use crate::feed::{
    Delta,
    Ready,
};

/// Reads each key's winner out of a document's store and sends it on.
pub(super) struct Reader {
    doc:         Document,
    tx:          Sender<Delta>,
    /// Keys whose winner's content has not downloaded yet, by that content's
    /// hash.
    parked:      HashMap<Hash, HashSet<String>>,
    /// Each parked key's current hash, so a key re-parked under a new hash is
    /// dropped from its old one instead of leaking there forever.
    parked_hash: HashMap<String, Hash>,
}

impl Reader {
    pub(super) fn new(doc: Document, tx: Sender<Delta>) -> Self {
        Self {
            doc,
            tx,
            parked: HashMap::new(),
            parked_hash: HashMap::new(),
        }
    }

    /// Subscribes before reading the snapshot, so no insert lands between
    /// the two unseen. A key may then arrive twice, which a projection
    /// absorbs.
    pub(super) async fn run(mut self, ready: Ready) -> anyhow::Result<()> {
        let mut events = self.doc.subscribe().await?;

        let holds_prim = self.snapshot().await?;
        let mut waiting = match ready {
            Ready::Snapshot => false,
            Ready::RemoteSync => !holds_prim,
        };
        if !waiting {
            self.send(Delta::Synced).await?;
        }

        let mut synced_remote = false;
        while let Some(event) = events.next().await {
            match event? {
                LiveEvent::InsertLocal { entry } | LiveEvent::InsertRemote { entry, .. } => {
                    self.reread(entry.key()).await?;
                }
                LiveEvent::ContentReady { hash } => {
                    for key in self.parked.remove(&hash).unwrap_or_default() {
                        self.parked_hash.remove(&key);
                        self.reread(key.as_bytes()).await?;
                    }
                }
                LiveEvent::SyncFinished(_) => synced_remote = true,
                LiveEvent::PendingContentReady if waiting && synced_remote => {
                    waiting = false;
                    self.send(Delta::Synced).await?;
                }
                LiveEvent::PendingContentReady
                | LiveEvent::NeighborUp(_)
                | LiveEvent::NeighborDown(_) => {}
            }
        }
        Ok(())
    }

    /// Answers whether the store already holds a prim.
    async fn snapshot(&mut self) -> anyhow::Result<bool> {
        let mut holds_prim = false;
        for prefix in key::PREFIXES {
            let query = Query::single_latest_per_key()
                .key_prefix(prefix)
                .include_empty();
            let entries = self.doc.get_many(query).await?.collect::<Vec<_>>().await;
            for entry in entries {
                let entry = entry?;
                holds_prim |= prefix == key::PRIM_PREFIX && entry.content_len() > 0;
                self.forward(&entry).await?;
            }
        }
        Ok(holds_prim)
    }

    /// The inserted entry is not necessarily the winner, so every insert
    /// re-reads its key through the store's own order.
    async fn reread(&mut self, key: &[u8]) -> anyhow::Result<()> {
        let query = Query::single_latest_per_key()
            .key_exact(key)
            .include_empty();
        if let Some(entry) = self.doc.get_one(query).await? {
            return self.forward(&entry).await;
        }
        let Ok(key) = str::from_utf8(key) else {
            return Ok(());
        };
        self.send(Delta::Entry(Entry::new(key, Vec::new(), 0)))
            .await
    }

    /// An entry past [`MAX_ENTRY_BYTES`] is sent empty without fetching it.
    /// One whose content has not downloaded is parked until it has.
    async fn forward(&mut self, entry: &iroh_docs::Entry) -> anyhow::Result<()> {
        let Ok(key) = String::from_utf8(entry.key().to_vec()) else {
            return Ok(());
        };
        let len = entry.content_len();

        let value = if len == 0 {
            self.unpark(&key);
            Bytes::new()
        } else if usize::try_from(len).map_or(true, |len| len > MAX_ENTRY_BYTES) {
            warn!(%key, len, "entry is over the size cap and reads as empty");
            self.unpark(&key);
            Bytes::new()
        } else {
            let Some(bytes) = self.doc.value(entry).await? else {
                self.park(key, entry.content_hash());
                return Ok(());
            };
            self.unpark(&key);
            bytes
        };

        let timestamp = entry.timestamp();
        self.send(Delta::Entry(Entry::new(key, value, timestamp)))
            .await
    }

    /// Parks `key` under `hash`, dropping it from any hash it was parked
    /// under before.
    fn park(&mut self, key: String, hash: Hash) {
        self.unpark(&key);
        self.parked.entry(hash).or_default().insert(key.clone());
        self.parked_hash.insert(key, hash);
    }

    /// Drops `key` from the hash it is parked under, if any.
    fn unpark(&mut self, key: &str) {
        let Some(hash) = self.parked_hash.remove(key) else {
            return;
        };
        if let Some(keys) = self.parked.get_mut(&hash) {
            keys.remove(key);
            if keys.is_empty() {
                self.parked.remove(&hash);
            }
        }
    }

    async fn send(&self, delta: Delta) -> anyhow::Result<()> {
        self.tx
            .send(delta)
            .await
            .map_err(|_| anyhow!("document feed dropped"))
    }
}
