//! Bounded content fetching for joined documents.
//!
//! The docs engine is told to download nothing, so an entry synced from a peer
//! arrives without its content. One fetch task per joined document fetches it
//! instead, refusing an entry over [`MAX_ENTRY_BYTES`] or one that would take
//! the document over [`MAX_DOC_BYTES`] before a byte is requested. A request
//! asks for no more than the entry's signed length, so content bigger than its
//! entry claims stops at the claim and reads as never downloaded.

use std::{
    collections::{
        HashMap,
        HashSet,
        VecDeque,
    },
    ops::ControlFlow,
    sync::{
        Arc,
        Mutex,
        MutexGuard,
        PoisonError,
    },
    time::Duration,
};

use async_channel::{
    Receiver,
    Sender,
};
use iroh::EndpointId;
use iroh_blobs::{
    Hash,
    api::{
        blobs::{
            BlobStatus,
            Blobs,
        },
        downloader::Downloader,
    },
    protocol::{
        ChunkRanges,
        ChunkRangesExt,
        GetRequest,
    },
};
use iroh_docs::{
    ContentStatus,
    NamespaceId,
    api::DocsApi,
    engine::LiveEvent,
    store::Query,
};
use n0_future::{
    FuturesUnordered,
    Stream,
    StreamExt,
    task::AbortOnDropHandle,
    time::Instant,
};

const MIB: u64 = 1024 * 1024;

/// Largest entry content this store fetches. Bigger content stays absent.
pub const MAX_ENTRY_BYTES: u64 = 32 * MIB;

/// Most content one joined document may bring onto disk.
pub const MAX_DOC_BYTES: u64 = 512 * MIB;

/// Most keys a joined document may hold. One that grows past it is left: it
/// stops syncing for the rest of the session.
pub const MAX_DOC_ENTRIES: u64 = 100_000;

const CONCURRENT_FETCHES: usize = 4;
const FETCH_TIMEOUT: Duration = Duration::from_mins(5);

/// Cap on remembered failed fetches, each retried when a peer next appears.
const MAX_MISSING: usize = 4096;

/// A cap hit by counts that only grow is re-measured against the replica, at
/// most this often, since a key written twice counts twice.
const RECOUNT_INTERVAL: Duration = Duration::from_mins(1);

/// What a document held when measured.
#[derive(Debug, Default)]
pub struct Scan {
    pub entries: u64,
    /// Content on disk, counted once per hash.
    pub bytes:   u64,
    /// Content not on disk, with the length its entry claims.
    pub missing: Vec<(Hash, u64)>,
}

/// What the fetch tasks need from the store.
#[derive(Clone)]
pub struct Context {
    pub blobs:      Blobs,
    pub downloader: Downloader,
    pub docs:       DocsApi,
}

/// The fetch task of every joined document.
#[derive(Debug, Default)]
pub struct Fetchers(Mutex<HashMap<NamespaceId, Fetcher>>);

#[derive(Debug)]
struct Fetcher {
    shared: Arc<Shared>,
    _task:  AbortOnDropHandle<()>,
}

#[derive(Debug, Default)]
struct Shared {
    subscribers: Mutex<Vec<Sender<LiveEvent>>>,
    /// Peers every fetch may ask, beyond the one that sent the entry.
    providers:   Mutex<Vec<EndpointId>>,
}

/// Poison only means another thread panicked mid-push; the lists stay valid.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Shared {
    fn add_providers(&self, providers: &[EndpointId]) {
        let mut held = lock(&self.providers);
        for provider in providers {
            if !held.contains(provider) {
                held.push(*provider);
            }
        }
    }

    fn publish(&self, event: &LiveEvent) {
        lock(&self.subscribers).retain(|tx| tx.try_send(event.clone()).is_ok());
    }
}

impl Fetchers {
    pub fn contains(&self, ns: NamespaceId) -> bool {
        lock(&self.0).contains_key(&ns)
    }

    pub fn add_providers(&self, ns: NamespaceId, providers: &[EndpointId]) {
        if let Some(fetcher) = lock(&self.0).get(&ns) {
            fetcher.shared.add_providers(providers);
        }
    }

    /// Starts fetching for `ns` from `events`, which must be subscribed before
    /// the document syncs. A no-op beyond adding `providers` if a racing join
    /// started one first.
    pub fn start(
        &self,
        ns: NamespaceId,
        events: impl Stream<Item = anyhow::Result<LiveEvent>> + Send + Unpin + 'static,
        scan: Scan,
        providers: &[EndpointId],
        cx: Context,
    ) {
        let mut fetchers = lock(&self.0);
        if let Some(fetcher) = fetchers.get(&ns) {
            fetcher.shared.add_providers(providers);
            return;
        }

        let shared = Arc::new(Shared::default());
        shared.add_providers(providers);

        let mut job = Job {
            ns,
            cx,
            shared: Arc::clone(&shared),
            entries: scan.entries,
            bytes: scan.bytes,
            queue: VecDeque::new(),
            pending: HashMap::new(),
            missing: HashMap::new(),
            awaiting_ready: false,
            last_recount: None,
            warned_full: false,
        };
        job.missing
            .extend(scan.missing.into_iter().take(MAX_MISSING));

        let task = n0_future::task::spawn(job.run(events));
        fetchers.insert(
            ns,
            Fetcher {
                shared,
                _task: AbortOnDropHandle::new(task),
            },
        );
    }

    /// A feed of `ns`'s events, or `None` when it is not being fetched.
    pub fn subscribe(&self, ns: NamespaceId) -> Option<Receiver<LiveEvent>> {
        let shared = Arc::clone(&lock(&self.0).get(&ns)?.shared);
        // Unbounded, since a fetch task waiting on a slow reader would stall
        // every other reader. A reader that never drains holds at most one
        // event per entry the caps let in.
        let (tx, rx) = async_channel::unbounded();
        lock(&shared.subscribers).push(tx);
        Some(rx)
    }

    pub fn stop(&self, ns: NamespaceId) {
        lock(&self.0).remove(&ns);
    }
}

struct Fetch {
    hash:      Hash,
    len:       u64,
    providers: Vec<EndpointId>,
}

enum Outcome {
    Complete,
    /// Ended short of complete: the content is longer than its entry claims,
    /// or the provider held only part of it.
    Refused,
    Failed,
}

struct Job {
    ns:             NamespaceId,
    cx:             Context,
    shared:         Arc<Shared>,
    /// Keys at the last count plus remote inserts since: an upper bound.
    entries:        u64,
    /// Content on disk plus what queued and in-flight fetches may add.
    bytes:          u64,
    queue:          VecDeque<Fetch>,
    /// Queued and in-flight fetches, with the bytes each reserved.
    pending:        HashMap<Hash, u64>,
    missing:        HashMap<Hash, u64>,
    /// Set by a finished sync, cleared once its fetches drain.
    awaiting_ready: bool,
    last_recount:   Option<Instant>,
    warned_full:    bool,
}

impl Job {
    async fn run(mut self, mut events: impl Stream<Item = anyhow::Result<LiveEvent>> + Unpin) {
        let mut inflight = FuturesUnordered::new();

        self.retry_missing(None).await;

        loop {
            while inflight.len() < CONCURRENT_FETCHES
                && let Some(fetch) = self.queue.pop_front()
            {
                inflight.push(fetch_one(self.cx.clone(), fetch));
            }

            if self.awaiting_ready && inflight.is_empty() {
                self.awaiting_ready = false;
                self.shared.publish(&LiveEvent::PendingContentReady);
            }

            tokio::select! {
                event = events.next() => match event {
                    Some(Ok(event)) => {
                        if self.on_event(event).await.is_break() {
                            return;
                        }
                    }
                    Some(Err(err)) => tracing::debug!(ns = %self.ns, ?err, "document event lost"),
                    None => return,
                },
                Some((hash, len, outcome)) = inflight.next(), if !inflight.is_empty() => {
                    self.on_fetched(hash, len, outcome);
                }
            }
        }
    }

    async fn on_event(&mut self, event: LiveEvent) -> ControlFlow<()> {
        match &event {
            LiveEvent::InsertRemote {
                from,
                entry,
                content_status,
            } => {
                self.entries += 1;
                if self.entries > MAX_DOC_ENTRIES
                    && self
                        .recount()
                        .await
                        .is_some_and(|entries| entries > MAX_DOC_ENTRIES)
                {
                    tracing::error!(
                        ns = %self.ns,
                        cap = MAX_DOC_ENTRIES,
                        "document holds too many keys; leaving it"
                    );
                    self.leave().await;
                    return ControlFlow::Break(());
                }
                if !matches!(content_status, ContentStatus::Complete) {
                    self.want(entry.content_hash(), entry.content_len(), Some(*from))
                        .await;
                }
            }
            LiveEvent::SyncFinished(sync) => {
                self.awaiting_ready = true;
                self.retry_missing(Some(sync.peer)).await;
            }
            LiveEvent::NeighborUp(peer) => self.retry_missing(Some(*peer)).await,
            // The engine's own reports, about downloads it no longer makes.
            LiveEvent::ContentReady { .. } | LiveEvent::PendingContentReady => {
                return ControlFlow::Continue(());
            }
            LiveEvent::InsertLocal { .. } | LiveEvent::NeighborDown(_) => {}
        }

        self.shared.publish(&event);
        ControlFlow::Continue(())
    }

    fn on_fetched(&mut self, hash: Hash, len: u64, outcome: Outcome) {
        self.pending.remove(&hash);
        match outcome {
            Outcome::Complete => self.shared.publish(&LiveEvent::ContentReady { hash }),
            Outcome::Refused => {
                tracing::warn!(ns = %self.ns, %hash, len, "content stopped short of its entry's length; it stays absent");
            }
            Outcome::Failed => {
                self.bytes = self.bytes.saturating_sub(len);
                if self.missing.len() < MAX_MISSING {
                    self.missing.insert(hash, len);
                }
            }
        }
    }

    async fn want(&mut self, hash: Hash, len: u64, from: Option<EndpointId>) {
        if len == 0 || self.pending.contains_key(&hash) {
            return;
        }
        if len > MAX_ENTRY_BYTES {
            tracing::debug!(ns = %self.ns, %hash, len, "entry is over the size cap; not fetching it");
            return;
        }
        if self.bytes + len > MAX_DOC_BYTES {
            self.recount().await;
            if self.bytes + len > MAX_DOC_BYTES {
                if !self.warned_full {
                    self.warned_full = true;
                    tracing::warn!(ns = %self.ns, cap = MAX_DOC_BYTES, "document is over its byte budget; fetching no more of it");
                }
                return;
            }
        }

        let mut providers = from.into_iter().collect::<Vec<_>>();
        for provider in lock(&self.shared.providers).iter() {
            if !providers.contains(provider) {
                providers.push(*provider);
            }
        }
        if providers.is_empty() {
            if self.missing.len() < MAX_MISSING {
                self.missing.insert(hash, len);
            }
            return;
        }

        self.bytes += len;
        self.pending.insert(hash, len);
        self.queue.push_back(Fetch {
            hash,
            len,
            providers,
        });
    }

    /// Asks `peer`, and the shared providers, for content an earlier fetch
    /// failed to get.
    async fn retry_missing(&mut self, peer: Option<EndpointId>) {
        for (hash, len) in std::mem::take(&mut self.missing) {
            self.want(hash, len, peer).await;
        }
    }

    /// Re-measures the counts against the replica, returning the key count.
    /// `None` when rate-limited, so a document genuinely over a cap is not
    /// re-scanned per entry, or when the replica would not measure.
    async fn recount(&mut self) -> Option<u64> {
        if self
            .last_recount
            .is_some_and(|last| last.elapsed() < RECOUNT_INTERVAL)
        {
            return None;
        }
        self.last_recount = Some(Instant::now());

        match self.measure().await {
            Ok((entries, bytes)) => {
                let reserved = self.pending.values().sum::<u64>();
                self.entries = entries;
                self.bytes = bytes + reserved;
                Some(entries)
            }
            Err(err) => {
                tracing::debug!(ns = %self.ns, ?err, "could not re-measure document");
                None
            }
        }
    }

    async fn measure(&self) -> anyhow::Result<(u64, u64)> {
        let Some(doc) = self.cx.docs.open(self.ns).await? else {
            anyhow::bail!("document is gone");
        };

        let measured = async {
            let mut entries = 0;
            let mut bytes = 0;
            let mut counted = HashSet::new();
            let mut stream = std::pin::pin!(doc.get_many(Query::single_latest_per_key()).await?);
            while let Some(entry) = stream.next().await {
                let entry = entry?;
                entries += 1;
                let hash = entry.content_hash();
                if let BlobStatus::Complete { size } = self.cx.blobs.status(hash).await?
                    && counted.insert(hash)
                {
                    bytes += size;
                }
            }
            anyhow::Ok((entries, bytes))
        }
        .await;

        doc.close().await?;
        measured
    }

    async fn leave(&self) {
        let left = async {
            if let Some(doc) = self.cx.docs.open(self.ns).await? {
                doc.leave().await?;
                doc.close().await?;
            }
            anyhow::Ok(())
        };
        if let Err(err) = left.await {
            tracing::warn!(ns = %self.ns, ?err, "failed to leave document");
        }
    }
}

async fn fetch_one(cx: Context, fetch: Fetch) -> (Hash, u64, Outcome) {
    let request = GetRequest::blob_ranges(fetch.hash, ChunkRanges::bytes(..fetch.len));
    let downloaded = n0_future::time::timeout(
        FETCH_TIMEOUT,
        cx.downloader.download(request, fetch.providers),
    )
    .await;

    let outcome = match downloaded {
        Ok(Ok(())) => match cx.blobs.status(fetch.hash).await {
            Ok(BlobStatus::Complete { .. }) => Outcome::Complete,
            Ok(BlobStatus::Partial { .. } | BlobStatus::NotFound) => Outcome::Refused,
            Err(err) => {
                tracing::debug!(hash = %fetch.hash, ?err, "could not read fetched content's status");
                Outcome::Failed
            }
        },
        Ok(Err(err)) => {
            tracing::debug!(hash = %fetch.hash, ?err, "content fetch failed");
            Outcome::Failed
        }
        Err(_) => {
            tracing::debug!(hash = %fetch.hash, "content fetch timed out");
            Outcome::Failed
        }
    };

    (fetch.hash, fetch.len, outcome)
}
