//! An open document: reads, writes under this store's author, and sync.

use std::{
    collections::HashSet,
    pin::{
        Pin,
        pin,
    },
    sync::Arc,
};

use bytes::Bytes;
use iroh_blobs::{
    Hash,
    api::blobs::{
        BlobStatus,
        Blobs,
    },
};
use iroh_docs::{
    AuthorId,
    Entry,
    NamespaceId,
    api::Doc,
    engine::LiveEvent,
    store::{
        DownloadPolicy,
        Query,
    },
};
use n0_future::{
    Stream,
    StreamExt,
};

use crate::{
    error::{
        IrohResultExt,
        Result,
    },
    fetch::{
        Fetchers,
        Scan,
    },
};

/// A document's live events: entries inserted locally or by peers, content
/// arriving, and sync progress.
///
/// For a joined document, [`LiveEvent::ContentReady`] and
/// [`LiveEvent::PendingContentReady`] report this store's bounded fetches.
///
/// `Send` on every target, unlike most `n0_future` streams on wasm, so it can
/// travel through a Bevy command.
pub type Events = Pin<Box<dyn Stream<Item = Result<LiveEvent>> + Send>>;

/// Whether a read sees keys whose latest entry is empty, which is how a
/// removed key reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tombstones {
    Include,
    Exclude,
}

/// An open document, writing as this store's author. Clones share one handle.
#[derive(Clone, Debug)]
pub struct Document {
    handle:   Arc<DocHandle>,
    blobs:    Blobs,
    author:   AuthorId,
    fetchers: Arc<Fetchers>,
}

/// Holds a replica open; closes it when dropped.
#[derive(Debug)]
struct DocHandle {
    doc:     Doc,
    /// The runtime the document was opened on. A [`Document`] may be dropped
    /// outside an async context, so the close is spawned here.
    #[cfg(not(target_family = "wasm"))]
    runtime: tokio::runtime::Handle,
}

impl Drop for DocHandle {
    fn drop(&mut self) {
        let doc = self.doc.clone();

        let close = async move {
            if let Err(err) = doc.close().await {
                tracing::debug!(?err, "failed to close document");
            }
        };

        cfg_select! {
            target_family = "wasm" => drop(n0_future::task::spawn(close)),
            _ => drop(self.runtime.spawn(close)),
        };
    }
}

impl Document {
    pub(crate) fn new(
        doc: Doc,
        blobs: Blobs,
        author: AuthorId,
        fetchers: Arc<Fetchers>,
    ) -> Result<Self> {
        let handle = DocHandle {
            doc,
            #[cfg(not(target_family = "wasm"))]
            runtime: tokio::runtime::Handle::try_current().iroh()?,
        };

        Ok(Self {
            handle: Arc::new(handle),
            blobs,
            author,
            fetchers,
        })
    }

    pub(crate) fn doc(&self) -> &Doc {
        &self.handle.doc
    }

    #[must_use]
    pub fn id(&self) -> NamespaceId {
        self.doc().id()
    }

    /// The content at `key`, or `None` if no entry holds any or its content has
    /// not been downloaded.
    pub async fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Bytes>> {
        match self.entry(key, Tombstones::Exclude).await? {
            Some(entry) => self.value(&entry).await,
            None => Ok(None),
        }
    }

    /// The latest entry at `key`.
    pub async fn entry(
        &self,
        key: impl AsRef<[u8]>,
        tombstones: Tombstones,
    ) -> Result<Option<Entry>> {
        let query = Query::single_latest_per_key().key_exact(key);
        let query = match tombstones {
            Tombstones::Include => query.include_empty(),
            Tombstones::Exclude => query,
        };
        Ok(self.doc().get_one(query).await?)
    }

    /// The latest entry per key under `prefix`, in key order.
    pub async fn entries(
        &self,
        prefix: impl AsRef<[u8]>,
        tombstones: Tombstones,
    ) -> Result<impl Stream<Item = Result<Entry>>> {
        let query = Query::single_latest_per_key().key_prefix(prefix);
        let query = match tombstones {
            Tombstones::Include => query.include_empty(),
            Tombstones::Exclude => query,
        };
        let stream = self.doc().get_many(query).await?;
        Ok(stream.map(IrohResultExt::iroh))
    }

    /// Every key's latest entry under `prefix`, removed keys left out.
    pub async fn list(&self, prefix: impl AsRef<[u8]>) -> Result<Vec<Entry>> {
        let mut stream = pin!(self.entries(prefix, Tombstones::Exclude).await?);
        let mut out = Vec::new();
        while let Some(entry) = stream.next().await {
            out.push(entry?);
        }
        Ok(out)
    }

    /// An entry's content, or `None` if it has not been downloaded.
    pub async fn value(&self, entry: &Entry) -> Result<Option<Bytes>> {
        let hash = entry.content_hash();
        if !self.blobs.has(hash).await.iroh()? {
            return Ok(None);
        }
        Ok(Some(self.blobs.get_bytes(hash).await.iroh()?))
    }

    /// How many bytes of this document's content are on disk. Content shared by
    /// several keys counts once.
    pub async fn size(&self) -> Result<u64> {
        Ok(self.scan().await?.bytes)
    }

    /// What the document holds, measured against the blob store rather than
    /// the lengths entries claim.
    pub(crate) async fn scan(&self) -> Result<Scan> {
        let mut scan = Scan::default();
        let mut counted = HashSet::new();

        let mut stream = pin!(self.entries("", Tombstones::Exclude).await?);
        while let Some(entry) = stream.next().await {
            let entry = entry?;
            scan.entries += 1;

            let hash = entry.content_hash();
            match self.blobs.status(hash).await.iroh()? {
                BlobStatus::Complete { size } => {
                    if counted.insert(hash) {
                        scan.bytes += size;
                    }
                }
                BlobStatus::Partial { .. } | BlobStatus::NotFound => {
                    scan.missing.push((hash, entry.content_len()));
                }
            }
        }

        Ok(scan)
    }

    /// Writes `value` at `key`.
    pub async fn set(&self, key: impl Into<Bytes>, value: impl Into<Bytes>) -> Result<Hash> {
        Ok(self.doc().set_bytes(self.author, key, value).await?)
    }

    /// Points `key` at content already addressed by `hash`, uploading nothing.
    ///
    /// `size` is the content's length, which iroh-docs requires be non-zero.
    pub async fn set_hash(&self, key: impl Into<Bytes>, hash: Hash, size: u64) -> Result<()> {
        Ok(self.doc().set_hash(self.author, key, hash, size).await?)
    }

    /// Removes this store's entry at exactly `key`.
    ///
    /// iroh-docs only deletes by prefix, so the longer keys under `key` whose
    /// latest entry is this store's are written back afterwards, under a fresh
    /// timestamp. Entries other authors wrote are left alone.
    pub async fn remove_key(&self, key: impl Into<Bytes>) -> Result<()> {
        let key = key.into();

        let mut longer = Vec::new();
        let mut stream = pin!(self.entries(key.clone(), Tombstones::Exclude).await?);
        while let Some(entry) = stream.next().await {
            let entry = entry?;
            if entry.author() == self.author && entry.key() != key.as_ref() {
                longer.push(entry);
            }
        }

        self.doc().del(self.author, key).await?;

        for entry in longer {
            self.set_hash(
                Bytes::copy_from_slice(entry.key()),
                entry.content_hash(),
                entry.content_len(),
            )
            .await?;
        }

        Ok(())
    }

    /// Removes every entry under `prefix` this store authored, returning how
    /// many were removed.
    ///
    /// Entries other authors wrote are left alone; removing one of those means
    /// writing an empty value, which wins by timestamp and reads as absence.
    pub async fn remove_prefix(&self, prefix: impl Into<Bytes>) -> Result<usize> {
        Ok(self.doc().del(self.author, prefix).await?)
    }

    /// Events from here on. A joined document reports content as this store
    /// fetches it.
    pub async fn subscribe(&self) -> Result<Events> {
        if let Some(fetched) = self.fetchers.subscribe(self.id()) {
            return Ok(Box::pin(fetched.map(Ok)));
        }
        let events = self.doc().subscribe().await?;
        Ok(Box::pin(events.map(IrohResultExt::iroh)))
    }

    /// Enrols in the sync set, so peers asking for this namespace are answered.
    ///
    /// A namespace outside the sync set rejects every request with `NotFound`.
    /// Enrolment dials nobody, and the sync engine holds its own handle, so it
    /// outlives the document that asked. Content peers write is not fetched;
    /// see [`crate::Store::join`].
    pub async fn serve(&self) -> Result<()> {
        self.refuse_downloads().await?;
        self.doc().start_sync(Vec::new()).await?;
        Ok(())
    }

    /// Whether the namespace is in the sync set.
    pub async fn is_served(&self) -> Result<bool> {
        Ok(self.doc().status().await?.sync)
    }

    /// Stops the docs engine fetching content for this namespace, which it does
    /// unbounded by default. The policy is stored with the replica.
    pub(crate) async fn refuse_downloads(&self) -> Result<()> {
        self.doc()
            .set_download_policy(DownloadPolicy::NothingExcept(Vec::new()))
            .await?;
        Ok(())
    }
}
