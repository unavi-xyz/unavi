use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use flume::{Receiver, Sender};
use iroh::protocol::{Router, RouterBuilder};
use iroh::{Endpoint, EndpointId, SecretKey};
use xdid::core::did::Did;

use crate::db::Database;
use crate::sync::{ALPN, ConnectionPool, SyncEvent, WiredSyncProtocol};
use crate::{DataStoreView, GarbageCollectStats, gc, hash_did};

pub struct DataStoreBuilder {
    pub data_dir: PathBuf,
    pub ephemeral: bool,
    pub with_router: Option<Box<dyn Send + FnOnce(&Endpoint, RouterBuilder) -> RouterBuilder>>,
}

impl DataStoreBuilder {
    #[must_use]
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            ephemeral: true,
            with_router: None,
        }
    }

    /// # Errors
    ///
    /// Returns error if directories cannot be created or database cannot be initialized.
    pub async fn build(self) -> anyhow::Result<DataStore> {
        let data_dir = self.data_dir;

        std::fs::create_dir_all(data_dir.join("blobs")).context("create blobs directory")?;
        std::fs::create_dir_all(data_dir.join("records")).context("create records directory")?;

        let db_path = data_dir.join("index.db");
        let db = Database::new(&db_path).await?;

        // Initialize iroh endpoint.
        let endpoint = init_iroh(&db, self.ephemeral).await?;

        // Create connection pool.
        let connection_pool = Arc::new(ConnectionPool::new());

        // Create sync protocol handler.
        let protocol = WiredSyncProtocol::new(db.clone(), Arc::clone(&connection_pool));

        // Create router and register sync protocol.
        let router_builder = Router::builder(endpoint.clone()).accept(ALPN, protocol);

        let router_builder = if let Some(f) = self.with_router {
            f(&endpoint, router_builder)
        } else {
            router_builder
        };

        let router = router_builder.spawn();

        // Create sync event channel.
        let (sync_tx, sync_rx) = flume::unbounded();

        Ok(DataStore {
            connection_pool,
            data_dir,
            db,
            endpoint,
            router,
            sync_rx,
            sync_tx,
        })
    }
}

async fn init_iroh(db: &Database, ephemeral: bool) -> anyhow::Result<Endpoint> {
    // Load or create secret key.
    let secret_key = load_or_create_secret_key(db, ephemeral).await?;

    // Create iroh endpoint.
    let mut builder = Endpoint::builder().secret_key(secret_key);

    #[cfg(feature = "discovery-local-network")]
    {
        let mdns = iroh::discovery::mdns::MdnsDiscovery::builder();
        builder = builder.discovery(mdns);
    }

    let endpoint = builder.bind().await.context("bind iroh endpoint")?;

    Ok(endpoint)
}

async fn load_or_create_secret_key(db: &Database, ephemeral: bool) -> anyhow::Result<SecretKey> {
    if !ephemeral {
        // Query existing key.
        let existing: Option<Vec<u8>> =
            sqlx::query_scalar!("SELECT secret_key FROM iroh_config WHERE id = 1")
                .fetch_optional(db.pool())
                .await
                .context("query iroh secret key")?;

        if let Some(bytes) = existing {
            let arr: [u8; 32] = bytes
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid secret key length"))?;
            return Ok(SecretKey::from_bytes(&arr));
        }
    }

    // Generate new key and persist.
    let secret_key = SecretKey::generate(&mut rand::rng());
    let bytes = secret_key.to_bytes().to_vec();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_secs()
        .cast_signed();

    if !ephemeral {
        sqlx::query!(
            "INSERT INTO iroh_config (id, secret_key, created) VALUES (1, ?, ?)",
            bytes,
            now
        )
        .execute(db.pool())
        .await
        .context("insert iroh secret key")?;
    }

    Ok(secret_key)
}

/// Data store owning shared infrastructure.
///
/// Create once per server instance. Owns the database and
/// data directory, which are shared across all users. Use `view_for_user()` to
/// create lightweight per-user `DataStoreView` instances.
pub struct DataStore {
    connection_pool: Arc<ConnectionPool>,
    data_dir: PathBuf,
    db: Database,
    endpoint: Endpoint,
    router: Router,
    sync_rx: Receiver<SyncEvent>,
    sync_tx: Sender<SyncEvent>,
}

impl DataStore {
    /// Create a lightweight per-user `DataStoreView`.
    ///
    /// Intended for per-user session (reuse across requests). The returned
    /// `DataStoreView` is cheap to clone and can be used concurrently.
    ///
    /// Ensures the user's record directory exists before returning.
    #[must_use]
    pub fn view_for_user(&self, owner_did: Did) -> DataStoreView {
        let did_hash = hash_did(&owner_did);
        let user_dir = self.data_dir.join("records").join(did_hash);
        let _ = std::fs::create_dir_all(&user_dir);

        DataStoreView {
            db: self.db.clone(),
            data_dir: self.data_dir.clone(),
            owner_did,
            sync_tx: self.sync_tx.clone(),
        }
    }

    /// Access to the shared database for server-level operations.
    ///
    /// Use this for operations that span multiple users, such as global
    /// garbage collection or statistics.
    #[must_use]
    pub const fn database(&self) -> &Database {
        &self.db
    }

    /// Data directory path.
    #[must_use]
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    #[must_use]
    pub const fn router(&self) -> &Router {
        &self.router
    }

    /// Access to the iroh endpoint for peer connections.
    #[must_use]
    pub const fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    /// This node's iroh [`EndpointId`].
    #[must_use]
    pub fn endpoint_id(&self) -> EndpointId {
        self.endpoint.id()
    }

    /// Access to the connection pool for sync operations.
    #[must_use]
    pub const fn connection_pool(&self) -> &Arc<ConnectionPool> {
        &self.connection_pool
    }

    /// Subscribe to sync events.
    ///
    /// Returns a cloneable receiver for sync events emitted by record operations.
    /// Use this to implement sync protocol handlers.
    #[must_use]
    pub fn sync_events(&self) -> Receiver<SyncEvent> {
        self.sync_rx.clone()
    }

    /// Runs garbage collection across all users to remove expired pins, orphaned records, and orphaned blobs.
    ///
    /// This is a server-level operation that should be run periodically.
    ///
    /// # Errors
    ///
    /// Returns error if GC operations fail.
    ///
    /// # Panics
    ///
    /// Panics if system time is before UNIX epoch.
    pub async fn garbage_collect(&self) -> anyhow::Result<GarbageCollectStats> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_secs()
            .cast_signed();

        let mut stats = GarbageCollectStats {
            pins_removed: 0,
            records_removed: 0,
            blobs_removed: 0,
            bytes_freed: 0,
        };

        // Remove expired pins.
        let (expired_pins, pins_removed) = gc::remove_expired_pins(self.db.pool(), now).await?;
        stats.pins_removed = pins_removed;

        // Remove unpinned records.
        for (record_id, owner_did) in expired_pins {
            if !gc::has_remaining_pins(self.db.pool(), &record_id, &owner_did).await?
                && let Some(size) = gc::remove_unpinned_record(
                    self.db.pool(),
                    &self.data_dir,
                    &record_id,
                    &owner_did,
                )
                .await?
            {
                stats.records_removed += 1;
                stats.bytes_freed += size;
            }
        }

        // Remove orphaned user_blobs.
        gc::remove_orphaned_user_blobs(self.db.pool()).await?;

        // Remove orphaned blobs.
        let blobs_dir = self.data_dir.join("blobs");
        let (blobs_removed, bytes_freed) =
            gc::remove_orphaned_blobs(self.db.pool(), &blobs_dir).await?;
        stats.blobs_removed = blobs_removed;
        stats.bytes_freed += bytes_freed;

        Ok(stats)
    }
}
