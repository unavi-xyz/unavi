use std::{
    str::FromStr,
    time::Duration,
};

use iroh::{
    Endpoint,
    protocol::RouterBuilder,
};
use iroh_blobs::{
    BlobsProtocol,
    api::{
        Store as BlobStore,
        blobs::Blobs,
    },
    store::{
        GcConfig,
        mem::MemStore,
    },
};
use iroh_docs::{
    Author,
    AuthorId,
    Capability,
    NamespaceId,
    api::Doc,
    engine::ProtectCallbackHandler,
    protocol::Docs,
};
use iroh_gossip::net::Gossip;
use n0_future::task::AbortOnDropHandle;

use crate::{
    cache::Cache,
    local::Storage,
    namespace::Namespace,
};

#[cfg(not(target_family = "wasm"))] mod fs;
#[cfg(target_family = "wasm")] mod web;

pub type BoxedRouterBuilder = Box<dyn FnOnce(RouterBuilder) -> RouterBuilder + Send + Sync>;
type BoxedBlobs = Box<dyn AsRef<BlobStore> + Send + Sync>;

/// Where this node's root document id is recorded, beside the identity that
/// authored it.
const ROOT_KEY: &str = "root-doc";

/// Every handle this node's data plane runs on. Each is `Arc`-backed.
#[derive(Clone, Debug)]
pub struct Store {
    blobs:   BlobStore,
    docs:    Docs,
    gossip:  Gossip,
    author:  AuthorId,
    storage: Storage,
    root:    NamespaceId,
}

pub struct Spawned {
    pub store:  Store,
    /// Registers the blob, gossip and docs protocols on a router.
    pub router: BoxedRouterBuilder,
    /// Dropping this shuts the blob store down and stops garbage collection, so
    /// it outlives every use of [`Self::store`].
    pub guard:  Guard,
}

pub struct Guard {
    _blobs: BoxedBlobs,
    _gc:    Option<AbortOnDropHandle<()>>,
}

impl Store {
    /// Makes `ns` available locally, importing it read-only if this node does
    /// not already hold it.
    ///
    /// `Docs::open` errors on an unknown namespace rather than returning
    /// `Ok(None)`, so it cannot serve as the import path. Merging a read
    /// capability into a write capability already held is a no-op, not a
    /// downgrade.
    pub async fn open(&self, ns: NamespaceId) -> anyhow::Result<Namespace> {
        let doc = self
            .docs
            .api()
            .import_namespace(Capability::Read(ns))
            .await?;
        Ok(self.wrap(doc))
    }

    /// Mints a namespace this node holds the write capability for.
    pub async fn create(&self) -> anyhow::Result<Namespace> {
        Ok(self.wrap(self.docs.api().create().await?))
    }

    /// Opens the namespace this store's [`Storage`] records at `key`, minting
    /// and recording one on first use.
    pub async fn open_or_mint(&self, key: &str) -> anyhow::Result<Namespace> {
        Ok(self.wrap(open_or_mint_doc(&self.docs, &self.storage, key).await?))
    }

    /// This node's root document: the one namespace everything else hangs off.
    #[must_use]
    pub const fn root(&self) -> NamespaceId {
        self.root
    }

    #[must_use]
    pub const fn author(&self) -> AuthorId {
        self.author
    }

    #[must_use]
    pub fn cache(&self) -> Cache {
        Cache::new(self.blobs.clone())
    }

    #[must_use]
    pub const fn blob_store(&self) -> &BlobStore {
        &self.blobs
    }

    #[must_use]
    pub fn blobs(&self) -> &Blobs {
        self.blobs.blobs()
    }

    #[must_use]
    pub const fn docs(&self) -> &Docs {
        &self.docs
    }

    /// The one gossip instance for this endpoint.
    ///
    /// `iroh_gossip::ALPN` can be accepted only once per router, so a second
    /// instance registering it takes every inbound connection from the first,
    /// leaving that one able to dial out and never to receive.
    #[must_use]
    pub const fn gossip(&self) -> &Gossip {
        &self.gossip
    }

    fn wrap(&self, doc: Doc) -> Namespace {
        Namespace::new(doc, self.blobs.blobs().clone(), self.author)
    }
}

pub struct Builder {
    author:   Author,
    endpoint: Endpoint,
    gc_timer: Option<Duration>,
    storage:  Storage,
}

impl Builder {
    #[must_use]
    pub const fn new(endpoint: Endpoint, author: Author) -> Self {
        Self {
            author,
            endpoint,
            gc_timer: None,
            storage: Storage::Ephemeral,
        }
    }

    /// Spawns a task to sweep expired cache tags at a set frequency. Disabled
    /// by default.
    #[must_use]
    pub const fn gc_timer(mut self, frequency: Duration) -> Self {
        self.gc_timer = Some(frequency);
        self
    }

    /// Where blobs and documents are kept. Defaults to
    /// [`Storage::Ephemeral`], which holds both in memory.
    ///
    /// [`Storage::Path`] is not supported on wasm — [`Self::build`] errors if
    /// it is set there, since there is no filesystem to fall back to.
    #[must_use]
    pub fn storage(mut self, storage: Storage) -> Self {
        self.storage = storage;
        self
    }

    pub async fn build(self) -> anyhow::Result<Spawned> {
        // Blob GC reclaims anything no tag covers; document content carries no
        // tag of its own and is reported through this callback instead. Both
        // halves are wired or neither — a GC run without the callback would
        // reclaim every open document's content.
        let (protect_handler, protect_cb) = ProtectCallbackHandler::new();
        let gc = self.gc_timer.map(|interval| GcConfig {
            interval,
            add_protected: Some(protect_cb),
        });

        let (owned, docs_builder) = cfg_select! {
            target_family = "wasm" => web::init(&self.storage, gc)?,
            _ => fs::init(&self.storage, gc).await?,
        };
        let blobs = owned.as_ref().as_ref().clone();

        sweep_auto_tags(&blobs).await?;

        let gossip = Gossip::builder().spawn(self.endpoint.clone());

        let docs = docs_builder
            .protect_handler(protect_handler)
            .spawn(self.endpoint, blobs.clone(), gossip.clone())
            .await?;

        // Derived rather than minted, so this node writes under the same author
        // every session, and that author names its endpoint.
        let author = self.author.id();
        docs.api().author_import(self.author).await?;
        docs.api().author_set_default(author).await?;

        let root = open_or_mint_doc(&docs, &self.storage, ROOT_KEY).await?.id();

        let blob_protocol = BlobsProtocol::new(&blobs, None);

        let router = {
            let docs = docs.clone();
            let gossip = gossip.clone();
            Box::new(move |builder: RouterBuilder| {
                builder
                    .accept(iroh_blobs::ALPN, blob_protocol)
                    .accept(iroh_gossip::ALPN, gossip)
                    .accept(iroh_docs::ALPN, docs)
            })
        };

        let gc = self.gc_timer.map(|duration| {
            let cache = Cache::new(blobs.clone());
            let handle = n0_future::task::spawn(async move {
                loop {
                    // Content itself is reclaimed by the blob store's own
                    // sweep, which reads the tags this pass leaves behind.
                    match cache.sweep().await {
                        Ok(deleted) if deleted > 0 => {
                            tracing::debug!(deleted, "expired cache tags");
                        }
                        Ok(_) => {}
                        Err(err) => tracing::warn!(?err, "failed to sweep cache tags"),
                    }
                    n0_future::time::sleep(duration).await;
                }
            });
            AbortOnDropHandle::new(handle)
        });

        Ok(Spawned {
            store: Store {
                blobs,
                docs,
                gossip,
                author,
                storage: self.storage,
                root,
            },
            router,
            guard: Guard {
                _blobs: owned,
                _gc:    gc,
            },
        })
    }
}

/// Opens the namespace `storage` records at `key`, minting and recording one on
/// first use.
///
/// The id outlives the process, so a restart reopens the document peers already
/// reference. A recorded id whose capability the docs store no longer holds
/// means that store was lost; the document is unrecoverable either way, so a
/// fresh one is minted and the record replaced.
///
/// [`Storage::Ephemeral`] mints fresh every run and leaves nothing behind.
async fn open_or_mint_doc(docs: &Docs, storage: &Storage, key: &str) -> anyhow::Result<Doc> {
    if let Some(ns) = recorded(storage, key)
        && let Some(doc) = held(docs, ns).await
    {
        return Ok(doc);
    }

    let doc = docs.api().create().await?;
    storage.write(key, &doc.id().to_string())?;
    Ok(doc)
}

fn recorded(storage: &Storage, key: &str) -> Option<NamespaceId> {
    match storage.read(key) {
        Ok(Some(text)) => NamespaceId::from_str(text.trim()).ok(),
        Ok(None) => None,
        Err(err) => {
            tracing::warn!(%key, ?err, "recorded namespace is unreadable; minting a replacement");
            None
        }
    }
}

/// `Docs::open` errors on a namespace this node does not hold rather than
/// returning `Ok(None)`, so both shapes of absence have to read alike; a
/// propagated error would leave the node with no document at all.
async fn held(docs: &Docs, ns: NamespaceId) -> Option<Doc> {
    match docs.api().open(ns).await {
        Ok(doc) => doc,
        Err(err) => {
            tracing::warn!(%ns, ?err, "recorded namespace is unreadable; minting a replacement");
            None
        }
    }
}

/// Deletes leftover `auto-<rfc3339>` tags minted by bare `add_bytes(..)` calls,
/// which nothing else sweeps; content still referenced by a document survives
/// through the protect callback.
async fn sweep_auto_tags(blobs: &BlobStore) -> anyhow::Result<()> {
    let deleted = blobs.tags().delete_prefix("auto-").await?;
    if deleted > 0 {
        tracing::info!(deleted, "swept orphaned auto tags");
    }
    Ok(())
}

fn mem_store(gc: Option<GcConfig>) -> MemStore {
    MemStore::new_with_opts(iroh_blobs::store::mem::Options { gc_config: gc })
}
