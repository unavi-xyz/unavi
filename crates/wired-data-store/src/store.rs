use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use xdid::core::did::Did;

use crate::{DataStoreView, GarbageCollectStats, db::Database, gc, hash_did};

/// Data store owning shared infrastructure.
///
/// Create once per server instance. Owns the `SQLite` database and
/// data directory, which are shared across all users. Use `view_for_user()` to
/// create lightweight per-user `DataStoreView` instances.
///
/// # Future: Iroh Integration
///
/// The Iroh P2P layer will be added to this struct as `Arc<IrohNode>`.
/// Iroh responsibilities:
/// - P2P node management (single node per server)
/// - Blob transfer using iroh-blobs (content-addressed)
/// - Record syncing protocol (user-agnostic transport)
/// - Network connectivity
///
/// Integration pattern:
/// - `DataStore` owns `IrohNode` (user-agnostic)
/// - Database enforces `owner_did` isolation (user-specific)
/// - Blobs are content-addressed (naturally shared)
pub struct DataStore {
    db: Arc<Database>,
    data_dir: PathBuf,
}

impl DataStore {
    /// Initialize the data store with shared infrastructure.
    ///
    /// Creates the database, blobs directory, and records directory.
    /// This should be called once during server startup.
    ///
    /// # Errors
    ///
    /// Returns error if directories cannot be created or database cannot be initialized.
    pub async fn new(data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(data_dir.join("blobs")).context("create blobs directory")?;
        std::fs::create_dir_all(data_dir.join("records")).context("create records directory")?;

        let db_path = data_dir.join("index.db");
        let db = Arc::new(Database::new(&db_path).await?);

        Ok(Self { db, data_dir })
    }

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
            db: Arc::clone(&self.db),
            data_dir: self.data_dir.clone(),
            owner_did,
        }
    }

    /// Access to the shared database for server-level operations.
    ///
    /// Use this for operations that span multiple users, such as global
    /// garbage collection or statistics.
    #[must_use]
    pub const fn database(&self) -> &Arc<Database> {
        &self.db
    }

    /// Data directory path.
    #[must_use]
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
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
    pub async fn garbage_collect(&self) -> Result<GarbageCollectStats> {
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

        // Step 1: Remove expired pins (across all users).
        let (expired_pins, pins_removed) = gc::remove_expired_pins(self.db.pool(), now).await?;
        stats.pins_removed = pins_removed;

        // Step 2: Remove unpinned records (across all users).
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

        // Step 3: Remove orphaned user_blobs (across all users).
        gc::remove_orphaned_user_blobs(self.db.pool()).await?;

        // Step 4: Remove orphaned blobs (shared across all users).
        let blobs_dir = self.data_dir.join("blobs");
        let (blobs_removed, bytes_freed) =
            gc::remove_orphaned_blobs(self.db.pool(), &blobs_dir).await?;
        stats.blobs_removed = blobs_removed;
        stats.bytes_freed += bytes_freed;

        Ok(stats)
    }
}

// TODO: Future Iroh integration methods:
//
// pub async fn sync_user_records(&self, owner_did: &Did) -> Result<()>
//   - Fetch records via P2P for a specific user
//   - Write to DB with owner_did isolation
//   - User-agnostic at transport level
//   - User-specific at data level
//
// pub async fn provide_blob(&self, blob_id: &BlobId) -> Result<()>
//   - Announce blob availability to the network
//   - Uses iroh-blobs for content-addressed transfer
//
// pub async fn subscribe_to_updates(&self) -> Result<UpdateStream>
//   - Real-time record syncing
//   - Multiplexed across all users
